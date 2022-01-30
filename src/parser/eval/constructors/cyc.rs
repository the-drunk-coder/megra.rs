use crate::builtin_types::*;
use crate::cyc_parser;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::sample_set::SampleSet;
use crate::session::OutputMode;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};

use parking_lot::Mutex;

pub fn cyc(
    functions: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    global_parameters: &sync::Arc<GlobalParameters>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);

    // ignore function name in this case
    tail_drain.next();

    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Symbol(n)) = tail_drain.next() {
        n
    } else {
        "".to_string()
    };

    let mut dur: Parameter = if let ConfigParameter::Numeric(d) = global_parameters
        .entry(BuiltinGlobalParameters::DefaultDuration)
        .or_insert(ConfigParameter::Numeric(200.0))
        .value()
    {
        Parameter::with_value(*d)
    } else {
        unreachable!()
    };

    let mut repetition_chance: f32 = 0.0;
    let mut randomize_chance: f32 = 0.0;
    let mut max_repetitions: f32 = 0.0;

    let mut dur_vec: Vec<Parameter> = Vec::new();

    let mut collect_events = false;
    let mut collect_template = false;
    let mut template_evs = Vec::new();

    // collect mapped events, i.e. :events 'a (saw 200) ...
    let mut collected_evs = Vec::new();
    let mut collected_mapping = HashMap::<String, Vec<SourceEvent>>::new();
    let mut cur_key: String = "".to_string();

    // collect final events in their position in the cycle
    let mut ev_vecs = Vec::new();
    let mut cycle_string: String = "".to_string();

    while let Some(c) = tail_drain.next() {
        if collect_template {
            match c {
                EvaluatedExpr::Symbol(s) => {
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
                EvaluatedExpr::Symbol(ref s) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        //println!("found event {}", cur_key);
                        collected_mapping.insert(cur_key.clone(), collected_evs.clone());
                        collected_evs.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e)) => {
                    collected_evs.push(SourceEvent::Sound(e));
                    continue;
                }
                EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(e)) => {
                    collected_evs.push(SourceEvent::Control(e));
                    continue;
                }
                _ => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        //println!("found event {}", cur_key);
                        collected_mapping.insert(cur_key.clone(), collected_evs.clone());
                    }
                    collect_events = false;
                }
            }
        }

        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Float(n)) => {
                        dur = Parameter::with_value(n);
                    }
                    Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "rep" => {
                    if let EvaluatedExpr::Float(n) = tail_drain.next().unwrap() {
                        repetition_chance = n;
                    }
                }
                "rnd" => {
                    if let EvaluatedExpr::Float(n) = tail_drain.next().unwrap() {
                        randomize_chance = n;
                    }
                }
                "max-rep" => {
                    if let EvaluatedExpr::Float(n) = tail_drain.next().unwrap() {
                        max_repetitions = n;
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
                _ => println!("{}", k),
            },
            EvaluatedExpr::String(d) => {
                cycle_string = d.clone();
            }
            _ => println! {"ignored"},
        }
    }

    let mut parsed_cycle = cyc_parser::eval_cyc_from_str(
        &cycle_string,
        functions,
        sample_set,
        out_mode,
        &template_evs,
        &collected_mapping,
        global_parameters,
    );

    if parsed_cycle.is_empty() {
        println!("couldn't parse cycle");
    }

    for mut cyc_evs in parsed_cycle.drain(..) {
        match cyc_evs.as_slice() {
            [cyc_parser::CycleResult::Duration(d)] => {
                // only single durations count
                // slice pattern are awesome !
                *dur_vec.last_mut().unwrap() = Parameter::with_value(*d);
            }
            _ => {
                let mut pos_vec = Vec::new();
                dur_vec.push(dur.clone());

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

                ev_vecs.push(pos_vec);
            }
        }
    }

    // generated ids
    let mut last_char: char = '1';
    let first_char = last_char;

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();

    // collect cycle rules
    let mut rules = Vec::new();

    let mut count = 0;
    let num_events = ev_vecs.len();
    for ev in ev_vecs.drain(..) {
        let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();

        event_mapping.insert(last_char, ev);

        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev
            .params
            .insert(SynthParameter::Duration, Box::new(dur_vec[count].clone()));
        duration_mapping.insert((last_char, next_char), dur_ev);

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

            last_char = next_char;
        }

        count += 1;
    }

    // if our cycle isn't empty ...
    if count != 0 {
        // create duration event (otherwise not needed ...)
        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev.params.insert(
            SynthParameter::Duration,
            Box::new(dur_vec.last().unwrap().clone()),
        );
        duration_mapping.insert((last_char, first_char), dur_ev);
    }

    let mut pfa = Pfa::<char>::infer_from_rules(&mut rules, true);

    // this seems to be heavy ...
    // what's so heavy here ??
    if randomize_chance > 0.0 {
        //println!("add rnd chance");
        pfa.randomize_edges(randomize_chance, randomize_chance);
        pfa.rebalance();
    }

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })))
}
