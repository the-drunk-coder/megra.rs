use crate::builtin_types::*;
use crate::cyc_parser;
use crate::eval::resolver::resolve_globals;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::OutputMode;
use anyhow::bail;
use anyhow::Result;
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::parser::{EvaluatedExpr, FunctionMap};

pub fn a_loop(
    functions: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    sample_set: SampleAndWavematrixSet,
    out_mode: OutputMode,
) -> Result<EvaluatedExpr> {
    // eval-time resolve
    // ignore function name
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..).peekable();

    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(n)))) =
        tail_drain.peek()
    {
        n.clone()
    } else {
        bail!("loop - missing name");
    };

    tail_drain.next();

    // get the default global duration ...
    let mut dur: DynVal = if let TypedEntity::ConfigParameter(ConfigParameter::Numeric(d)) = globals
        .entry(VariableId::DefaultDuration)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Numeric(
            200.0,
        )))
        .value()
    {
        DynVal::with_value(*d)
    } else {
        bail!("loop - global default duration not present");
    };

    // chances for modifiers ...
    let mut repetition_chance: f32 = 0.0;
    let mut randomize_chance: f32 = 0.0;
    let mut max_repetitions: f32 = 0.0;

    // collect event abbreviations ...
    let mut collect_events = false;
    // get a template ...
    let mut collect_template = false;
    let mut template_evs = Vec::new();

    // collect mapped events, i.e. :events 'a (saw 200) ...
    let mut collected_evs = Vec::new();
    let mut collected_mapping = HashMap::<String, (Vec<SourceEvent>, Event)>::new();
    let mut cur_key: String = "".to_string();

    // collect final events in their position in the cycle
    let mut ev_vecs = Vec::new();

    let mut keep_root = false;

    let mut time_shift = 0;

    while let Some(c) = tail_drain.next() {
        if collect_template {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                    template_evs.push(s);
                    continue;
                }
                _ => {
                    collect_template = false;
                }
            }
        }

        if collect_events {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(ref s))) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        //println!("found event {}", cur_key);
                        collected_mapping.insert(
                            cur_key.clone(),
                            (collected_evs.clone(), Event::transition(dur.clone())),
                        );
                        collected_evs.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                    collected_evs.push(SourceEvent::Sound(e));
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::ControlEvent(e)) => {
                    collected_evs.push(SourceEvent::Control(e));
                    continue;
                }
                _ => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        //println!("found event {}", cur_key);
                        collected_mapping.insert(
                            cur_key.clone(),
                            (collected_evs.clone(), Event::transition(dur.clone())),
                        );
                    }
                    collect_events = false;
                }
            }
        }

        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(n)))) => {
                        dur = DynVal::with_value(n);
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "rep" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.peek()
                    {
                        repetition_chance = *n;
                        tail_drain.next();
                    }
                }
                "rnd" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.peek()
                    {
                        randomize_chance = *n;
                        tail_drain.next();
                    }
                }
                "max-rep" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.peek()
                    {
                        max_repetitions = *n;
                        tail_drain.next();
                    }
                }
                "shift" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.peek()
                    {
                        time_shift = *n as i32;
                        tail_drain.next();
                    }
                }
                "events" => {
                    collect_events = true;
                    continue;
                }
                "map" => {
                    collect_template = true;
                    continue;
                }
                "keep" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.peek()
                    {
                        keep_root = *b;
                        tail_drain.next();
                    }
                }
                _ => println!("{k}"),
            },
            EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                ev_vecs.push((vec![SourceEvent::Sound(e)], dur.clone()));
            }
            EvaluatedExpr::Typed(TypedEntity::ControlEvent(e)) => {
                ev_vecs.push((vec![SourceEvent::Control(e)], dur.clone()));
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                if !ev_vecs.is_empty() {
                    ev_vecs.last_mut().unwrap().1 = DynVal::with_value(f);
                }
                //else {
                //    dur_vec.push(DynVal::with_value(f));
                //}
            }
            // resolve two levels of vecs (for now ...)
            EvaluatedExpr::Typed(TypedEntity::Vec(v)) => {
                for x in v {
                    match *x {
                        TypedEntity::SoundEvent(e) => {
                            ev_vecs.push((vec![SourceEvent::Sound(e)], dur.clone()));
                        }
                        TypedEntity::ControlEvent(e) => {
                            ev_vecs.push((vec![SourceEvent::Control(e)], dur.clone()));
                        }
                        TypedEntity::Vec(mut v2) => {
                            v2.retain(|x| {
                                matches!(**x, TypedEntity::SoundEvent(_))
                                    || matches!(**x, TypedEntity::ControlEvent(_))
                            });
                            let mut chord = Vec::new();
                            for y in v2 {
                                match *y {
                                    TypedEntity::SoundEvent(s) => chord.push(SourceEvent::Sound(s)),
                                    TypedEntity::ControlEvent(s) => {
                                        chord.push(SourceEvent::Control(s))
                                    }
                                    _ => {}
                                }
                            }
                            ev_vecs.push((chord, dur.clone()));
                        }
                        _ => {}
                    }
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(d))) => {
                let mut parsed_cycle = cyc_parser::eval_cyc_from_str(
                    &d,
                    functions,
                    sample_set.clone(),
                    out_mode,
                    &template_evs,
                    &collected_mapping,
                    globals,
                );
                if parsed_cycle.is_empty() {
                    bail!("couldn't parse loop notation");
                }
                for mut cyc_evs in parsed_cycle.drain(..) {
                    match cyc_evs.as_slice() {
                        [cyc_parser::CycleResult::Duration(dur)] => {
                            // the loop, on the other hand, has variable
                            // durations between events ...
                            ev_vecs.last_mut().unwrap().1 = DynVal::with_value(*dur);
                        }
                        _ => {
                            let mut pos_vec = Vec::new();

                            for ev in cyc_evs.drain(..) {
                                match ev {
                                    cyc_parser::CycleResult::SoundEvent(s) => {
                                        pos_vec.push(SourceEvent::Sound(s))
                                    }
                                    cyc_parser::CycleResult::ControlEvent(c) => {
                                        pos_vec.push(SourceEvent::Control(c))
                                    }
                                    _ => {}
                                }
                            }

                            ev_vecs.push((pos_vec, dur.clone()));
                        }
                    }
                }
            }
            _ => println! {"ignored"},
        }
    }

    let mut event_mapping = BTreeMap::<char, (Vec<SourceEvent>, Event)>::new();

    // re-generate pfa if necessary, now that we have collected all the info ...
    let pfa = if !keep_root {
        // generated ids
        let mut last_char: char = '1';
        let first_char = last_char;

        // collect cycle rules
        let mut rules = Vec::new();

        let mut count = 0;
        let num_events = ev_vecs.len();
        for (ev, dur) in ev_vecs.drain(..) {
            let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();

            let mut dur_ev = Event::with_name("transition".to_string());
            dur_ev.params.insert(
                SynthParameterLabel::Duration.into(),
                ParameterValue::Scalar(dur),
            );

            event_mapping.insert(last_char, (ev, dur_ev));

            if count < num_events {
                if repetition_chance > 0.0 {
                    //println!("add rep chance");
                    // repetition rule
                    rules.push(Rule {
                        source: vec![last_char],
                        symbol: last_char,
                        probability: repetition_chance / 100.0,
                    });

                    // next rule
                    if count == num_events - 1 {
                        rules.push(Rule {
                            source: vec![last_char],
                            symbol: first_char,
                            probability: 1.0 - (repetition_chance / 100.0),
                        });
                    } else {
                        rules.push(Rule {
                            source: vec![last_char],
                            symbol: next_char,
                            probability: 1.0 - (repetition_chance / 100.0),
                        });
                    }

                    // endless repetition allowed per default ...
                    if max_repetitions >= 2.0 {
                        let mut max_rep_source = Vec::new();
                        for _ in 0..max_repetitions as usize {
                            max_rep_source.push(last_char);
                        }
                        // max repetition rule
                        if count == num_events - 1 {
                            rules.push(Rule {
                                source: max_rep_source,
                                symbol: first_char,
                                probability: 1.0,
                            });
                        } else {
                            rules.push(Rule {
                                source: max_rep_source,
                                symbol: next_char,
                                probability: 1.0,
                            });
                        }
                    }
                } else if count == num_events - 1 {
                    rules.push(Rule {
                        source: vec![last_char],
                        symbol: first_char,
                        probability: 1.0,
                    });
                } else {
                    rules.push(Rule {
                        source: vec![last_char],
                        symbol: next_char,
                        probability: 1.0,
                    });
                }

                if count < num_events - 1 {
                    last_char = next_char;
                }
            }

            count += 1;
        }

        let mut tmp = Pfa::<char>::infer_from_rules(&mut rules, true);

        // this seems to be heavy ...
        // what's so heavy here ??
        if randomize_chance > 0.0 {
            //println!("add rnd chance");
            tmp.randomize_edges(randomize_chance, randomize_chance);
            tmp.rebalance();
        }
        tmp
    } else {
        Pfa::<char>::new()
    };

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Ok(EvaluatedExpr::Typed(TypedEntity::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            label_mapping: None,
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
