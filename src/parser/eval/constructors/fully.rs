use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;

pub fn fully(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    global_parameters: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
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

    let mut collect_labeled = false;
    let mut collect_final = false;

    let mut collected_evs = Vec::new();
    let mut collected_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut cur_key: String = "".to_string();

    let mut final_mapping = HashMap::new();
    let mut last_char: char = 'a'; // label chars
    let mut labels = Vec::new();

    let mut dur: Parameter = if let ConfigParameter::Numeric(d) = global_parameters
        .entry(BuiltinGlobalParameters::DefaultDuration)
        .or_insert(ConfigParameter::Numeric(200.0))
        .value()
    {
        Parameter::with_value(*d)
    } else {
        unreachable!()
    };

    while let Some(c) = tail_drain.next() {
        if collect_labeled {
            match c {
                EvaluatedExpr::Symbol(ref s) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        //println!("found event {}", cur_key);
                        collected_mapping
                            .insert(cur_key.chars().next().unwrap(), collected_evs.clone());
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
                        collected_mapping
                            .insert(cur_key.chars().next().unwrap(), collected_evs.clone());
                    }
                    collect_labeled = false;
                }
            }
        }

        if collect_final {
            let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
            last_char = next_char;
            labels.push(next_char);
            let mut final_vec = Vec::new();

            match c {
                EvaluatedExpr::Symbol(ref s) => {
                    let label = s.chars().next().unwrap();
                    if collected_mapping.contains_key(&label) {
                        final_vec.append(&mut collected_mapping.get(&label).unwrap().clone());
                    }
                }
                EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e)) => {
                    final_vec.push(SourceEvent::Sound(e));
                }
                EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(e)) => {
                    final_vec.push(SourceEvent::Control(e));
                }
                _ => {}
            }

            final_mapping.insert(next_char, final_vec);
            continue;
        }

        if let EvaluatedExpr::Keyword(k) = c {
            match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Float(n)) => {
                        dur = Parameter::with_value(n);
                    }
                    Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "events" => {
                    collect_labeled = true;
                    continue;
                }
                "rest" => {
                    collect_final = true;
                    continue;
                }
                _ => println!("{}", k),
            }
        }
    }

    let mut duration_mapping = HashMap::new();
    let prob = 1.0 / (labels.len() - 1) as f32;
    // rules to collect ...
    let mut rules = Vec::new();
    for label_a in labels.iter() {
        for label_b in labels.iter() {
            rules.push(Rule {
                source: vec![*label_a],
                symbol: *label_b,
                probability: prob,
            });

            let mut dur_ev = Event::with_name("transition".to_string());
            dur_ev
                .params
                .insert(SynthParameter::Duration, Box::new(dur.clone()));
            duration_mapping.insert((*label_a, *label_b), dur_ev);
        }
    }

    let pfa = Pfa::<char>::infer_from_rules(&mut rules, true);

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping: final_mapping,
            duration_mapping,
            modified: true,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
        keep_root: false,
    })))
}
