use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::eval::resolver::resolve_globals;

use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::Pfa;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;

pub fn learn(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    // eval-time resolve
    // ignore function name
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);
    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(n)))) =
        tail_drain.next()
    {
        n
    } else {
        "".to_string()
    };

    let mut keep_root = false;
    let mut sample: String = "".to_string();
    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();

    let mut collect_events = false;
    let mut bound = 3;
    let mut epsilon = 0.01;
    let mut pfa_size = 30;
    let mut dur: DynVal = if let TypedEntity::ConfigParameter(ConfigParameter::Numeric(d)) = globals
        .entry(VariableId::DefaultDuration)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Numeric(
            200.0,
        )))
        .value()
    {
        DynVal::with_value(*d)
    } else {
        unreachable!()
    };

    let mut ev_vec = Vec::new();
    let mut cur_key: String = "".to_string();

    let mut autosilence = true;

    while let Some(c) = tail_drain.next() {
        if collect_events {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(ref s))) => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        //println!("found event {}", cur_key);
                        event_mapping.insert(cur_key.chars().next().unwrap(), ev_vec.clone());
                        ev_vec.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::Vec(ref v)) => {
                    for x in v.clone() {
                        if let TypedEntity::Pair(key, events) = *x {
                            if let TypedEntity::Comparable(Comparable::Symbol(key_sym)) = *key {
                                let mut unpack_evs = Vec::new();

                                match *events {
                                    TypedEntity::SoundEvent(ev) => {
                                        unpack_evs.push(SourceEvent::Sound(ev));
                                    }
                                    TypedEntity::ControlEvent(ev) => {
                                        unpack_evs.push(SourceEvent::Control(ev));
                                    }
                                    TypedEntity::Vec(pot_ev_vec) => {
                                        for pot_ev in pot_ev_vec {
                                            match *pot_ev {
                                                TypedEntity::SoundEvent(ev) => {
                                                    unpack_evs.push(SourceEvent::Sound(ev));
                                                }
                                                TypedEntity::ControlEvent(ev) => {
                                                    unpack_evs.push(SourceEvent::Control(ev));
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }

                                if !unpack_evs.is_empty() {
                                    event_mapping
                                        .insert(key_sym.chars().next().unwrap(), unpack_evs);
                                }
                            }
                        }
                    }
                    continue;
                }

                EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                    ev_vec.push(SourceEvent::Sound(e));
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::ControlEvent(e)) => {
                    ev_vec.push(SourceEvent::Control(e));
                    continue;
                }
                _ => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        //println!("found event {}", cur_key);
                        event_mapping.insert(cur_key.chars().next().unwrap(), ev_vec.clone());
                    }
                    collect_events = false;
                    // move on below
                }
            }
        }

        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "sample" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(
                        desc,
                    )))) => {
                        sample = desc.to_string();
                        sample.retain(|c| !c.is_whitespace());
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Vec(args))) => {
                        for arg in args {
                            match *arg {
                                TypedEntity::Comparable(Comparable::String(s)) => {
                                    sample.push_str(&s);
                                }
                                TypedEntity::Comparable(Comparable::Character(s)) => {
                                    sample.push(s);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                "events" => {
                    collect_events = true;
                    continue;
                }
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(n)))) => {
                        dur = DynVal::with_value(n);
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "bound" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        bound = n as usize;
                    }
                }
                "epsilon" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        epsilon = n;
                    }
                }
                "size" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        pfa_size = n as usize;
                    }
                }
                "autosilence" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        autosilence = b;
                    }
                }
                "keep" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        keep_root = b;
                    }
                }
                _ => println!("{k}"),
            },
            _ => println! {"ignored {c:?}"},
        }
    }

    if autosilence {
        event_mapping.insert(
            '~',
            vec![SourceEvent::Sound(Event::with_name("silence".to_string()))],
        );
    }

    let s_v: std::vec::Vec<char> = sample.chars().collect();
    let pfa = if !keep_root {
        // only regenerate if necessary
        Pfa::<char>::learn(s_v, bound, epsilon, pfa_size)
    } else {
        Pfa::<char>::new()
    };
    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Some(EvaluatedExpr::Typed(TypedEntity::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            duration_mapping: HashMap::new(),
            modified: true,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
        keep_root,
    })))
}
