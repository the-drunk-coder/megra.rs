use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;

use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;

pub fn friendship(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    var_store: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    // ignore function name in this case
    let mut tail_drain = tail.drain(..).skip(1);

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
    let center_label: char = '1'; // label chars
    let mut last_char: char = '1'; // label chars
    let mut friends_labels = Vec::new();

    let mut dur: DynVal = if let TypedVariable::ConfigParameter(ConfigParameter::Numeric(d)) =
        var_store
            .entry(VariableId::DefaultDuration)
            .or_insert(TypedVariable::ConfigParameter(ConfigParameter::Numeric(
                200.0,
            )))
            .value()
    {
        DynVal::with_value(*d)
    } else {
        unreachable!()
    };

    let mut keep_root = false;
    let mut repetition_chance: f32 = 0.0;
    let mut randomize_chance: f32 = 0.0;
    let mut max_repetitions: f32 = 0.0;

    while let Some(c) = tail_drain.next() {
        if collect_labeled {
            match c {
                EvaluatedExpr::Symbol(ref s) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
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
            friends_labels.push(next_char);
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
                        dur = DynVal::with_value(n);
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
                "rep" => {
                    if let Some(EvaluatedExpr::Float(n)) = tail_drain.next() {
                        repetition_chance = n;
                    }
                }
                "rnd" => {
                    if let Some(EvaluatedExpr::Float(n)) = tail_drain.next() {
                        randomize_chance = n;
                    }
                }
                "max-rep" => {
                    if let Some(EvaluatedExpr::Float(n)) = tail_drain.next() {
                        max_repetitions = n;
                    }
                }
                "center" => {
                    if let Some(c) = tail_drain.next() {
                        //let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
                        //last_char = next_char;
                        //center_label = next_char;
                        let mut final_vec = Vec::new();

                        match c {
                            EvaluatedExpr::Symbol(ref s) => {
                                let label = s.chars().next().unwrap();
                                if collected_mapping.contains_key(&label) {
                                    final_vec.append(
                                        &mut collected_mapping.get(&label).unwrap().clone(),
                                    );
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

                        final_mapping.insert(center_label, final_vec);
                    }
                    continue;
                }
                "friends" => {
                    collect_final = true;
                    continue;
                }
                "keep" => {
                    if let Some(EvaluatedExpr::Boolean(b)) = tail_drain.next() {
                        keep_root = b;
                    }
                }
                _ => println!("{k}"),
            }
        }
    }

    let mut duration_mapping = HashMap::new();

    let pfa = if !keep_root {
        // first of all check if we have enough friends events
        let needed_friends = friends_labels.len() % 2;
        if needed_friends != 0 {
            let last_found = last_char;
            for _ in 0..needed_friends {
                let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
                last_char = next_char;
                friends_labels.push(next_char);
                let repetition = final_mapping.get(&last_found).unwrap().clone();
                final_mapping.insert(next_char, repetition);
            }
        }

        // convert repetition chance
        if repetition_chance > 0.0 {
            repetition_chance /= 100.0;
        }

        let center_exit_prob = (1.0 - repetition_chance) / (friends_labels.len() as f32 / 2.0);
        let norep_center_exit_prob = 1.0 / (friends_labels.len() as f32 / 2.0);

        // rules to collect ...
        let mut rules = Vec::new();
        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev.params.insert(
            SynthParameterLabel::Duration,
            ParameterValue::Scalar(dur.clone()),
        );

        let mut friends_iter = friends_labels.iter();

        // center repetition
        if repetition_chance > 0.0 {
            rules.push(Rule {
                source: vec![center_label],
                symbol: center_label,
                probability: repetition_chance,
            });
        }

        for _ in 0..(friends_labels.len() / 2) {
            if let Some(label_a) = friends_iter.next() {
                if let Some(label_b) = friends_iter.next() {
                    rules.push(Rule {
                        source: vec![center_label],
                        symbol: *label_a,
                        probability: center_exit_prob,
                    });

                    // center max repetition
                    if repetition_chance > 0.0 && max_repetitions >= 2.0 {
                        let max_rep_source = vec![center_label; max_repetitions as usize];

                        // max repetition rule
                        rules.push(Rule {
                            source: max_rep_source,
                            symbol: *label_a,
                            probability: norep_center_exit_prob,
                        });
                    }

                    //println!("push rule {} -> {}", center_label, label_a);

                    rules.push(Rule {
                        source: vec![*label_a],
                        symbol: *label_b,
                        probability: 1.0 - repetition_chance,
                    });

                    if repetition_chance > 0.0 {
                        rules.push(Rule {
                            source: vec![*label_a],
                            symbol: *label_a,
                            probability: repetition_chance,
                        });

                        if max_repetitions >= 2.0 {
                            let mut max_rep_source = Vec::new();
                            for _ in 0..max_repetitions as usize {
                                max_rep_source.push(*label_a);
                            }

                            // max repetition rule
                            rules.push(Rule {
                                source: max_rep_source,
                                symbol: *label_b,
                                probability: 1.0,
                            });
                        }
                    }

                    //println!("push rule {} -> {}", label_a, label_b);

                    rules.push(Rule {
                        source: vec![*label_b],
                        symbol: center_label,
                        probability: 1.0 - repetition_chance,
                    });

                    if repetition_chance > 0.0 {
                        rules.push(Rule {
                            source: vec![*label_b],
                            symbol: *label_b,
                            probability: repetition_chance,
                        });

                        if max_repetitions >= 2.0 {
                            let mut max_rep_source = Vec::new();
                            for _ in 0..max_repetitions as usize {
                                max_rep_source.push(*label_b);
                            }

                            // max repetition rule
                            rules.push(Rule {
                                source: max_rep_source,
                                symbol: center_label,
                                probability: 1.0,
                            });
                        }
                    }

                    //println!("push rule {} -> {}", label_b, center_label);

                    duration_mapping.insert((center_label, *label_a), dur_ev.clone());
                    duration_mapping.insert((*label_a, *label_b), dur_ev.clone());
                    duration_mapping.insert((*label_b, center_label), dur_ev.clone());
                }
            }
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
        keep_root,
    })))
}
