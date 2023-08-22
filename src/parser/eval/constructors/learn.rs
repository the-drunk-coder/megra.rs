use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::eval::resolver::resolve_globals;

use std::collections::{BTreeMap, BTreeSet, HashMap};
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
    let mut sample: Vec<String> = Vec::new();
    let mut event_mapping = BTreeMap::<String, Vec<SourceEvent>>::new();

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
                        event_mapping.insert(cur_key, ev_vec.clone());
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
                                    event_mapping.insert(key_sym, unpack_evs);
                                }
                            }
                        }
                    }
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::Map(ref m)) => {
                    for (k, v) in m {
                        let key = match k {
                            VariableId::Custom(s) => s.clone(),
                            VariableId::Symbol(s) => s.clone(),
                            _ => {
                                continue;
                            }
                        };
                        let mut ev_vec = Vec::new();
                        match v {
                            TypedEntity::Vec(v) => {
                                for elem in v {
                                    match *elem.clone() {
                                        TypedEntity::SoundEvent(s) => {
                                            ev_vec.push(SourceEvent::Sound(s.clone()))
                                        }
                                        TypedEntity::ControlEvent(c) => {
                                            ev_vec.push(SourceEvent::Control(c.clone()))
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            TypedEntity::SoundEvent(s) => {
                                ev_vec.push(SourceEvent::Sound(s.clone()))
                            }
                            TypedEntity::ControlEvent(c) => {
                                ev_vec.push(SourceEvent::Control(c.clone()))
                            }
                            _ => {}
                        }
                        event_mapping.insert(key, ev_vec);
                    }
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
                        event_mapping.insert(cur_key.clone(), ev_vec.clone());
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
                        // if the string contains whitespace, assume it's the "new" behavior,
                        // where longer strings are allowed ...
                        if desc.contains(' ') {
                            // remove symbol markers ...
                            for token in desc.split_whitespace() {
                                if let Some(stripped) = token.strip_prefix('\'') {
                                    sample.push(stripped.to_string());
                                } else {
                                    sample.push(token.to_string());
                                }
                            }
                        } else {
                            // otherwise, assume the "old", single-character
                            // behaviour ...
                            sample.push(desc.to_string());
                        }
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Vec(args))) => {
                        for arg in args {
                            match *arg {
                                TypedEntity::Comparable(Comparable::String(s)) => {
                                    sample.push(s);
                                }
                                TypedEntity::Comparable(Comparable::Symbol(s)) => {
                                    sample.push(s);
                                }
                                TypedEntity::Comparable(Comparable::Character(s)) => {
                                    sample.push(format!("{s}"));
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
            _ => {
                collect_events = false;
                println!("ignored {c:#?}");
                continue;
            }
        }
    }

    if autosilence {
        event_mapping.insert(
            "~".to_string(),
            vec![SourceEvent::Sound(Event::with_name("silence".to_string()))],
        );
    }

    // create internal mapping from
    // string labels ...
    let mut char_event_mapping = BTreeMap::new();
    let mut label_mapping = BTreeMap::new();
    let mut reverse_label_mapping = BTreeMap::new();
    let mut next_char: char = '1';

    for (k, v) in event_mapping.into_iter() {
        char_event_mapping.insert(next_char, v);
        label_mapping.insert(next_char, k.clone());
        reverse_label_mapping.insert(k, next_char);
        next_char = std::char::from_u32(next_char as u32 + 1).unwrap();
    }

    // assemble the sample ...
    let mut s_v: std::vec::Vec<char> = Vec::new();

    //println!("raw sample {sample:?}");

    // if the sample has no whitespace, replicate
    // the previous behaviour ...
    if sample.len() == 1 && !sample[0].contains(' ') {
        for c in sample[0].chars() {
            let key = format!("{c}");
            if reverse_label_mapping.contains_key(&key) {
                s_v.push(*reverse_label_mapping.get(&key).unwrap());
            }
        }
    } else {
        // otherwise, tokenize by whitespace
        for token in sample {
            if reverse_label_mapping.contains_key(&token) {
                s_v.push(*reverse_label_mapping.get(&token).unwrap());
            }
        }
    }

    //println!("cooked sample {s_v:?}");
    //println!("map {label_mapping:#?}");
    //println!("rev map {reverse_label_mapping:#?}");

    let pfa = if !keep_root && !s_v.is_empty() && !char_event_mapping.is_empty() {
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
            event_mapping: char_event_mapping,
            label_mapping: Some(label_mapping),
            duration_mapping: HashMap::new(), // unsolved ...
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
