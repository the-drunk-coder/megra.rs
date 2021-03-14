use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::{MarkovSequenceGenerator, Rule};
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use vom_rs::pfa::Pfa;

pub fn construct_flower(tail: &mut Vec<Expr>) -> Atom {
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
    let mut pistil_label: char = '!'; // label chars
    let mut last_char: char = '!'; // label chars
    let mut petal_labels = Vec::new();
    let mut dur = 200.0;
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
                "dur" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        dur = n;
                    }
                }
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
                        let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
                        last_char = next_char;
                        pistil_label = next_char;
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

                        final_mapping.insert(next_char, final_vec);
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
    dur_ev.params.insert(
        SynthParameter::Duration,
        Box::new(Parameter::with_value(dur)),
    );

    let mut duration_mapping = HashMap::new();

    // convert repetition chance
    if repetition_chance > 0.0 {
	repetition_chance = repetition_chance / 100.0;
    }

    let pistil_exit_prob = (1.0 - repetition_chance) / ((petal_labels.len()) as f32 / num_layers as f32);

    let mut petal_iter = petal_labels.iter();
    let mut petal_repetition_handled = false;
    
    for _ in 0..(petal_labels.len() / num_layers) {
        let mut label_a = pistil_label;
        let mut last_label_a = pistil_label;
        let mut cur_prob = pistil_exit_prob;

        for l in 0..num_layers {            
            if let Some(label_b) = petal_iter.next() {
		//////////////////////
                // event repetition //
		//////////////////////
		if repetition_chance > 0.0 {
		    if l == 0 && !petal_repetition_handled {
			rules.push(
			    Rule {
				source: vec![label_a],
				symbol: label_a,
				probability: repetition_chance,
				duration: dur as u64,
			    }
			    .to_pfa_rule(),
			);
			petal_repetition_handled = true;
		    } else if l > 0 {
			rules.push(
			    Rule {
				source: vec![label_a],
				symbol: label_a,
				probability: repetition_chance,
				duration: dur as u64,
			    }
			    .to_pfa_rule(),
			);
		    }
                    
		    if max_repetitions >= 2.0 {
			let mut max_rep_source = Vec::new();
			for _ in 0..max_repetitions as usize {
                            max_rep_source.push(label_a);
			}
			// max repetition rule
			rules.push(
                            Rule {
				source: max_rep_source,
				symbol: *label_b,
				probability: 1.0,
				duration: dur as u64,
                            }
                            .to_pfa_rule(),
			);
                    }		    
		}
		//////////////////////////
                // END event repetition //
		//////////////////////////
                
                rules.push(
                    Rule {
                        source: vec![label_a],
                        symbol: *label_b,
                        probability: cur_prob,
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );

		if l == num_layers - 1 {
		    rules.push(
			Rule {
                            source: vec![*label_b],
                            symbol: label_a,
                            probability: 1.0 - repetition_chance,
                            duration: dur as u64,
			}
			.to_pfa_rule(),
                    );
		} else {
		    rules.push(
			Rule {
                            source: vec![*label_b],
                            symbol: label_a,
                            probability: 0.5 - repetition_chance,
                            duration: dur as u64,
			}
			.to_pfa_rule(),
                    );
		}
                
                duration_mapping.insert((label_a, *label_b), dur_ev.clone());
                duration_mapping.insert((*label_b, label_a), dur_ev.clone());

                // "label delay to handle max repetitions for outer layers ..."
                last_label_a = label_a;
                label_a = *label_b;

                cur_prob = 0.5;
            }
	    // event repetition, special case
	    // (outermost layer ...)
            if repetition_chance > 0.0 {
                rules.push(
		    Rule {
                        source: vec![label_a],
                        symbol: label_a,
                        probability: repetition_chance,
                        duration: dur as u64,
		    }
		    .to_pfa_rule(),
                );
		if  max_repetitions >= 2.0 {
		    let mut max_rep_source = Vec::new();
		    for _ in 0..max_repetitions as usize {
                        max_rep_source.push(label_a);
		    }
		    // max repetition rule
		    rules.push(
                        Rule {
			    source: max_rep_source,
			    symbol: last_label_a,
			    probability: 1.0,
			    duration: dur as u64,
                        }
                        .to_pfa_rule(),
		    );
                }
	    }
        }
    }

    let mut pfa = Pfa::<char>::infer_from_rules(&mut rules);

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
            default_duration: dur as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}
