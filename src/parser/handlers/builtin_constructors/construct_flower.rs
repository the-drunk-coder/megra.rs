use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

pub fn construct_flower(
    tail: &mut Vec<Expr>,
    global_parameters: &sync::Arc<GlobalParameters>,
) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    let name = if let Some(n) = get_string_from_expr(&tail_drain.next().unwrap()) {
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
    let pistil_label: char = 'a'; // label chars
    let mut last_char: char = 'a'; // label chars
    let mut petal_labels = Vec::new();

    let mut dur: Parameter = if let ConfigParameter::Numeric(d) = global_parameters
        .entry(BuiltinGlobalParameters::DefaultDuration)
        .or_insert(ConfigParameter::Numeric(200.0))
        .value()
    {
        Parameter::with_value(*d)
    } else {
        unreachable!()
    };

    let mut num_layers = 1;
    let mut repetition_chance: f32 = 0.0;
    let mut randomize_chance: f32 = 0.0;
    let mut max_repetitions: f32 = 0.0;

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        if collect_labeled {
            match c {
                Atom::Symbol(ref s) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        collected_mapping
                            .insert(cur_key.chars().next().unwrap(), collected_evs.clone());
                        collected_evs.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                Atom::SoundEvent(e) => {
                    collected_evs.push(SourceEvent::Sound(e));
                    continue;
                }
                Atom::ControlEvent(e) => {
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
            petal_labels.push(next_char);
            let mut final_vec = Vec::new();

            match c {
                Atom::Symbol(ref s) => {
                    let label = s.chars().next().unwrap();
                    if collected_mapping.contains_key(&label) {
                        final_vec.append(&mut collected_mapping.get(&label).unwrap().clone());
                    }
                }
                Atom::SoundEvent(e) => {
                    final_vec.push(SourceEvent::Sound(e));
                }
                Atom::ControlEvent(e) => {
                    final_vec.push(SourceEvent::Control(e));
                }
                _ => {}
            }

            final_mapping.insert(next_char, final_vec);
            continue;
        }

        if let Atom::Keyword(k) = c {
            match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(Expr::Constant(Atom::Float(n))) => {
                        dur = Parameter::with_value(n);
                    }
                    Some(Expr::Constant(Atom::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "events" => {
                    collect_labeled = true;
                    continue;
                }
                "layers" => {
                    if let Some(Expr::Constant(Atom::Float(n))) = tail_drain.next() {
                        num_layers = n as usize;
                    }
                }
                "pistil" => {
                    if let Some(Expr::Constant(c)) = tail_drain.next() {
                        let mut final_vec = Vec::new();

                        match c {
                            Atom::Symbol(ref s) => {
                                let label = s.chars().next().unwrap();
                                if collected_mapping.contains_key(&label) {
                                    final_vec.append(
                                        &mut collected_mapping.get(&label).unwrap().clone(),
                                    );
                                }
                            }
                            Atom::SoundEvent(e) => {
                                final_vec.push(SourceEvent::Sound(e));
                            }
                            Atom::ControlEvent(e) => {
                                final_vec.push(SourceEvent::Control(e));
                            }
                            _ => {}
                        }

                        final_mapping.insert(pistil_label, final_vec);
                    }
                    continue;
                }
                "petals" => {
                    collect_final = true;
                    continue;
                }
                "rep" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        repetition_chance = n;
                    }
                }
                "rnd" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        randomize_chance = n;
                    }
                }
                "max-rep" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        max_repetitions = n;
                    }
                }
                _ => println!("{}", k),
            }
        }
    }

    // first of all check if we have enough petal events
    // if not, repeat the last event until we do ...
    let needed_petals = petal_labels.len() % num_layers;
    if needed_petals != 0 {
        let last_found = last_char;
        for _ in 0..needed_petals {
            let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
            last_char = next_char;
            petal_labels.push(next_char);
            let repetition = final_mapping.get(&last_found).unwrap().clone();
            final_mapping.insert(next_char, repetition);
        }
    }

    ////////////////////
    // assemble rules //
    ////////////////////

    // rules to collect ...
    let mut rules = Vec::new();
    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev
        .params
        .insert(SynthParameter::Duration, Box::new(dur.clone()));

    let mut duration_mapping = HashMap::new();

    // convert repetition chance
    if repetition_chance > 0.0 {
        repetition_chance /= 100.0;
    }

    let pistil_exit_prob =
        (1.0 - repetition_chance) / ((petal_labels.len()) as f32 / num_layers as f32);
    let norep_pistil_exit_prob = 1.0 / ((petal_labels.len()) as f32 / num_layers as f32);

    let mut petal_iter = petal_labels.iter();

    // pistil repetition
    if repetition_chance > 0.0 {
        rules.push(Rule {
            source: vec![pistil_label],
            symbol: pistil_label,
            probability: repetition_chance,
        });
    }

    for _ in 0..(petal_labels.len() / num_layers) {
        let mut label_a = pistil_label;
        let mut cur_prob = pistil_exit_prob;

        for l in 0..num_layers {
            if let Some(label_b) = petal_iter.next() {
                //////////////////////
                // event repetition //
                //////////////////////
                // pistil repetition exit
                if label_a == pistil_label && repetition_chance > 0.0 && max_repetitions >= 2.0 {
                    let max_rep_source = vec![pistil_label; max_repetitions as usize];

                    // max repetition rule
                    rules.push(Rule {
                        source: max_rep_source,
                        symbol: *label_b,
                        probability: norep_pistil_exit_prob,
                    });
                }

                rules.push(Rule {
                    source: vec![label_a],
                    symbol: *label_b,
                    probability: cur_prob,
                });

                // we already repeated the pistil label ...
                if repetition_chance > 0.0 {
                    rules.push(Rule {
                        source: vec![*label_b],
                        symbol: *label_b,
                        probability: repetition_chance,
                    });

                    if label_a != pistil_label && max_repetitions >= 2.0 {
                        let mut max_rep_source = Vec::new();
                        for _ in 0..max_repetitions as usize {
                            max_rep_source.push(label_a);
                        }
                        // max repetition rule
                        rules.push(Rule {
                            source: max_rep_source,
                            symbol: *label_b,
                            probability: 1.0,
                        });
                    }
                }

                if l == num_layers - 1 {
                    rules.push(Rule {
                        source: vec![*label_b],
                        symbol: label_a,
                        probability: 1.0 - repetition_chance,
                    });

                    if repetition_chance > 0.0 && max_repetitions >= 2.0 {
                        // event repetition, special case
                        // (outermost layer ...)
                        let mut max_rep_source = Vec::new();
                        for _ in 0..max_repetitions as usize {
                            max_rep_source.push(*label_b);
                        }
                        // max repetition rule
                        rules.push(Rule {
                            source: max_rep_source,
                            symbol: label_a,
                            probability: 1.0,
                        });
                    }
                } else {
                    rules.push(Rule {
                        source: vec![*label_b],
                        symbol: label_a,
                        probability: 0.5 - repetition_chance,
                    });
                }

                duration_mapping.insert((label_a, *label_b), dur_ev.clone());
                duration_mapping.insert((*label_b, label_a), dur_ev.clone());

                label_a = *label_b;

                cur_prob = 0.5;
            }
        }
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

    Atom::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping: final_mapping,
            duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}
