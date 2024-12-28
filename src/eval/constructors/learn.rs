use crate::builtin_types::*;
use crate::eval::resolver::resolve_globals;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;

use anyhow::bail;
use anyhow::Result;
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::Pfa;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn learn(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
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
        bail!("learn - missing name");
    };

    let mut keep_root = false;
    let mut sample: Vec<String> = Vec::new();
    let mut event_mapping = BTreeMap::<String, (Vec<SourceEvent>, Event)>::new();

    let mut collect_events = false;
    let mut bound = 3;
    let mut tie = true;
    let mut epsilon = 0.01;
    let mut pfa_size = 30;
    let mut time_shift = 0;

    // flag to see whether we allow long names in sample
    let mut longnames = false;

    let mut dur: DynVal = if let TypedEntity::ConfigParameter(ConfigParameter::Numeric(d)) = globals
        .entry(VariableId::DefaultDuration)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Numeric(
            200.0,
        )))
        .value()
    {
        DynVal::with_value(*d)
    } else {
        bail!("learn - global default duration not present");
    };

    let mut ev_vec = Vec::new();
    let mut cur_key: String = "".to_string();

    let mut autosilence = true;

    let mut last_dur = None;

    while let Some(c) = tail_drain.next() {
        if collect_events {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(ref s))) => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        // switch to long-name mode if a long name has been found ...
                        if s.len() > 1 {
                            longnames = true;
                        }

                        event_mapping.insert(
                            cur_key,
                            (
                                ev_vec.clone(),
                                Event::transition(if let Some(d) = last_dur.clone() {
                                    d
                                } else {
                                    dur.clone()
                                }),
                            ),
                        );
                        last_dur = None;
                        ev_vec.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::Vec(ref v)) => {
                    for x in v.clone() {
                        if let TypedEntity::Pair(key, events) = *x {
                            if let TypedEntity::Comparable(Comparable::Symbol(key_sym)) = *key {
                                let mut unpacked_evs = Vec::new();
                                let mut found_dur = None;

                                match *events {
                                    // single events
                                    TypedEntity::SoundEvent(ev) => {
                                        unpacked_evs.push(SourceEvent::Sound(ev));
                                    }
                                    TypedEntity::ControlEvent(ev) => {
                                        unpacked_evs.push(SourceEvent::Control(ev));
                                    }
                                    // chords
                                    TypedEntity::Vec(pot_ev_vec) => {
                                        for pot_ev in pot_ev_vec {
                                            match *pot_ev {
                                                TypedEntity::SoundEvent(ev) => {
                                                    unpacked_evs.push(SourceEvent::Sound(ev));
                                                }
                                                TypedEntity::ControlEvent(ev) => {
                                                    unpacked_evs.push(SourceEvent::Control(ev));
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    // events plus duration
                                    TypedEntity::Pair(pot_ev_vec, pot_dur) => {
                                        // again, either single events or chords
                                        match *pot_ev_vec {
                                            TypedEntity::SoundEvent(ev) => {
                                                unpacked_evs.push(SourceEvent::Sound(ev));
                                            }
                                            TypedEntity::ControlEvent(ev) => {
                                                unpacked_evs.push(SourceEvent::Control(ev));
                                            }
                                            TypedEntity::Vec(inner_ev_vec) => {
                                                for pot_ev in inner_ev_vec {
                                                    match *pot_ev {
                                                        TypedEntity::SoundEvent(ev) => {
                                                            unpacked_evs
                                                                .push(SourceEvent::Sound(ev));
                                                        }
                                                        TypedEntity::ControlEvent(ev) => {
                                                            unpacked_evs
                                                                .push(SourceEvent::Control(ev));
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                        match *pot_dur {
                                            TypedEntity::Comparable(Comparable::Float(f)) => {
                                                found_dur = Some(f);
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                                if !unpacked_evs.is_empty() {
                                    event_mapping.insert(
                                        key_sym,
                                        (
                                            unpacked_evs,
                                            Event::transition(if let Some(d) = found_dur {
                                                DynVal::with_value(d)
                                            } else {
                                                dur.clone()
                                            }),
                                        ),
                                    );
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

                        let mut unpacked_evs = Vec::new();
                        let mut found_dur = None;

                        match v.clone() {
                            TypedEntity::Vec(v) => {
                                for elem in v {
                                    match *elem.clone() {
                                        TypedEntity::SoundEvent(s) => {
                                            unpacked_evs.push(SourceEvent::Sound(s.clone()))
                                        }
                                        TypedEntity::ControlEvent(c) => {
                                            unpacked_evs.push(SourceEvent::Control(c.clone()))
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            // events plus duration
                            TypedEntity::Pair(pot_ev_vec, pot_dur) => {
                                // again, either single events or chords
                                match *pot_ev_vec {
                                    TypedEntity::SoundEvent(ev) => {
                                        unpacked_evs.push(SourceEvent::Sound(ev));
                                    }
                                    TypedEntity::ControlEvent(ev) => {
                                        unpacked_evs.push(SourceEvent::Control(ev));
                                    }
                                    TypedEntity::Vec(inner_ev_vec) => {
                                        for pot_ev in inner_ev_vec {
                                            match *pot_ev {
                                                TypedEntity::SoundEvent(ev) => {
                                                    unpacked_evs.push(SourceEvent::Sound(ev));
                                                }
                                                TypedEntity::ControlEvent(ev) => {
                                                    unpacked_evs.push(SourceEvent::Control(ev));
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                match *pot_dur {
                                    TypedEntity::Comparable(Comparable::Float(f)) => {
                                        found_dur = Some(f);
                                    }
                                    _ => {}
                                }
                            }
                            TypedEntity::SoundEvent(s) => {
                                unpacked_evs.push(SourceEvent::Sound(s.clone()))
                            }
                            TypedEntity::ControlEvent(c) => {
                                unpacked_evs.push(SourceEvent::Control(c.clone()))
                            }
                            _ => {}
                        }
                        event_mapping.insert(
                            key,
                            (
                                unpacked_evs,
                                Event::transition(if let Some(d) = found_dur {
                                    DynVal::with_value(d)
                                } else {
                                    dur.clone()
                                }),
                            ),
                        );
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
                // last duration wins
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                    last_dur = Some(DynVal::with_value(f));
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::Parameter(d)) => {
                    last_dur = Some(d.clone());
                    continue;
                }
                _ => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        //println!("found event {}", cur_key);
                        event_mapping.insert(
                            cur_key.clone(),
                            (
                                ev_vec.clone(),
                                Event::transition(if let Some(d) = last_dur.clone() {
                                    d
                                } else {
                                    dur.clone()
                                }),
                            ),
                        );
                        last_dur = None
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
                        // long id behaviour now has to be explicilty specified
                        if longnames {
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
                                    println!("found string {s:?}");
                                    if s.len() > 1 {
                                        // switch longnames mode on of a long string has been found
                                        longnames = true;
                                    }
                                    sample.push(s);
                                }
                                TypedEntity::Comparable(Comparable::Symbol(s)) => {
                                    if s.len() > 1 {
                                        // switch longnames mode on of a long string has been found
                                        longnames = true;
                                    }
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
                "tie" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(n),
                    ))) = tail_drain.next()
                    {
                        tie = n;
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
                "shift" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        time_shift = n as i32;
                        tail_drain.next();
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
            (
                vec![SourceEvent::Sound(Event::with_name("silence".to_string()))],
                Event::transition(dur.clone()),
            ),
        );
    }

    //println!("raw sample {sample:?}");
    //println!("longnames? {longnames}");
    // create internal mapping from
    // string labels ...
    let mut char_event_mapping = BTreeMap::new();

    // assemble the sample ...
    let mut s_v: std::vec::Vec<char> = Vec::new();

    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev.params.insert(
        SynthParameterLabel::Duration.into(),
        ParameterValue::Scalar(dur.clone()),
    );

    let label_mapping = if longnames {
        let mut label_mapping = BTreeMap::new();
        let mut reverse_label_mapping = BTreeMap::new();
        let mut next_char: char = '1';
        for (k, v) in event_mapping.into_iter() {
            char_event_mapping.insert(next_char, v);
            label_mapping.insert(next_char, k.clone());
            reverse_label_mapping.insert(k, next_char);
            next_char = std::char::from_u32(next_char as u32 + 1).unwrap();
        }
        // otherwise, tokenize by whitespace
        for token in sample {
            if reverse_label_mapping.contains_key(&token) {
                s_v.push(*reverse_label_mapping.get(&token).unwrap());
            }
        }
        Some(label_mapping)
    } else {
        for (k, v) in event_mapping.into_iter() {
            // assume the label is not empty ...
            if let Some(c) = k.chars().next() {
                char_event_mapping.insert(c, v);
            }
        }

        // single-char mapping
        for token in sample {
            for c in token.chars() {
                // filter out whitespace
                if !(c.is_whitespace() || c.is_ascii_whitespace()) {
                    s_v.push(c);
                }
            }
        }
        None
    };

    //println!("baked sample {s_v:?}");

    let mut pfa = if !keep_root && !s_v.is_empty() && !char_event_mapping.is_empty() {
        // only regenerate if necessary
        Pfa::<char>::learn(s_v, bound, epsilon, pfa_size)
    } else {
        Pfa::<char>::new()
    };

    pfa.restart_when_stuck = tie;

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Ok(EvaluatedExpr::Typed(TypedEntity::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping: char_event_mapping,
            label_mapping,
            override_durations: None,
            modified: true,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
        time_shift,
        keep_root,
    })))
}
