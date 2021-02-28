use std::collections::{BTreeSet, HashMap};
use std::sync;

use crate::builtin_types::*;
use crate::cyc_parser;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::{MarkovSequenceGenerator, Rule};
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use crate::sample_set::SampleSet;
use crate::session::OutputMode;
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use vom_rs::pfa::Pfa;

pub fn construct_learn(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    let name = if let Some(n) = get_string_from_expr(&tail_drain.next().unwrap()) {
        n
    } else {
        "".to_string()
    };

    let mut sample: String = "".to_string();
    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();

    let mut collect_events = false;
    let mut bound = 3;
    let mut epsilon = 0.01;
    let mut pfa_size = 30;

    let mut dur = 200;

    let mut ev_vec = Vec::new();
    let mut cur_key: String = "".to_string();

    let mut autosilence = false;

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        if collect_events {
            match c {
                Atom::Symbol(ref s) => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        println!("found event {}", cur_key);
                        event_mapping.insert(cur_key.chars().next().unwrap(), ev_vec.clone());
                        ev_vec.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                Atom::SoundEvent(e) => {
                    ev_vec.push(SourceEvent::Sound(e));
                    continue;
                }
                Atom::ControlEvent(e) => {
                    ev_vec.push(SourceEvent::Control(e));
                    continue;
                }
                _ => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        println!("found event {}", cur_key);
                        event_mapping.insert(cur_key.chars().next().unwrap(), ev_vec.clone());
                    }
                    collect_events = false;
                }
            }
        }

        match c {
            Atom::Keyword(k) => match k.as_str() {
                "sample" => {
                    if let Expr::Constant(Atom::Description(desc)) = tail_drain.next().unwrap() {
                        sample = desc.to_string();
                    }
                }
                "events" => {
                    collect_events = true;
                    continue;
                }
                "dur" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        dur = n as i32;
                    }
                }
                "bound" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        bound = n as usize;
                    }
                }
                "epsilon" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        epsilon = n;
                    }
                }
                "size" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        pfa_size = n as usize;
                    }
                }
                "autosilence" => {
                    if let Expr::Constant(Atom::Boolean(b)) = tail_drain.next().unwrap() {
                        autosilence = b;
                    }
                }
                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    if autosilence {
        event_mapping.insert(
            '~',
            vec![SourceEvent::Sound(Event::with_name("silence".to_string()))],
        );
    }

    let s_v: std::vec::Vec<char> = sample.chars().collect();
    let pfa = Pfa::<char>::learn(&s_v, bound, epsilon, pfa_size);
    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Atom::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name: name,
            generator: pfa,
            event_mapping,
            duration_mapping: HashMap::new(),
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}

pub fn construct_infer(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    let name = if let Some(n) = get_string_from_expr(&tail_drain.next().unwrap()) {
        n
    } else {
        "".to_string()
    };

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();
    let mut rules = Vec::new();

    let mut collect_events = false;
    let mut collect_rules = false;
    let mut dur: f32 = 200.0;

    let mut ev_vec = Vec::new();
    let mut cur_key: String = "".to_string();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        if collect_events {
            match c {
                Atom::Symbol(ref s) => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        println!("found event {}", cur_key);
                        event_mapping.insert(cur_key.chars().next().unwrap(), ev_vec.clone());
                        ev_vec.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                Atom::SoundEvent(e) => {
                    ev_vec.push(SourceEvent::Sound(e));
                    continue;
                }
                Atom::ControlEvent(e) => {
                    ev_vec.push(SourceEvent::Control(e));
                    continue;
                }
                _ => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        println!("found event {}", cur_key);
                        event_mapping.insert(cur_key.chars().next().unwrap(), ev_vec.clone());
                    }
                    collect_events = false;
                }
            }
        }

        if collect_rules {
            if let Atom::Rule(s) = c {
                let mut dur_ev = Event::with_name("transition".to_string());
                dur_ev.params.insert(
                    SynthParameter::Duration,
                    Box::new(Parameter::with_value(s.duration as f32)),
                );
                duration_mapping.insert((*s.source.last().unwrap(), s.symbol), dur_ev);
                rules.push(s.to_pfa_rule());
                continue;
            } else {
                collect_rules = false;
            }
        }

        match c {
            Atom::Keyword(k) => match k.as_str() {
                "rules" => {
                    collect_rules = true;
                    continue;
                }
                "events" => {
                    collect_events = true;
                    continue;
                }
                "dur" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        dur = n;
                    }
                }
                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    let pfa = Pfa::<char>::infer_from_rules(&mut rules);
    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Atom::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name: name,
            generator: pfa,
            event_mapping: event_mapping,
            duration_mapping: duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}

pub fn construct_nucleus(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    // name is the first symbol
    let name = if let Some(n) = get_string_from_expr(&tail_drain.next().unwrap()) {
        n
    } else {
        "".to_string()
    };

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();
    let mut rules = Vec::new();

    let mut dur: f32 = 200.0;
    let mut ev_vec = Vec::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::SoundEvent(e) => ev_vec.push(SourceEvent::Sound(e)),
            Atom::ControlEvent(c) => ev_vec.push(SourceEvent::Control(c)),
            Atom::Keyword(k) => match k.as_str() {
                "dur" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        dur = n;
                    }
                }
                _ => println!("{}", k),
            },
            _ => println! {"ignored"},
        }
    }

    event_mapping.insert('a', ev_vec);

    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev.params.insert(
        SynthParameter::Duration,
        Box::new(Parameter::with_value(dur)),
    );
    duration_mapping.insert(('a', 'a'), dur_ev);
    // one rule to rule them all
    rules.push(
        Rule {
            source: vec!['a'],
            symbol: 'a',
            probability: 1.0,
            duration: dur as u64,
        }
        .to_pfa_rule(),
    );

    let pfa = Pfa::<char>::infer_from_rules(&mut rules);
    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Atom::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name: name,
            generator: pfa,
            event_mapping: event_mapping,
            duration_mapping: duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}

pub fn construct_fully(tail: &mut Vec<Expr>) -> Atom {
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
    let mut last_char: char = '!'; // label chars
    let mut labels = Vec::new();
    let mut dur = 200.0;

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        if collect_labeled {
            match c {
                Atom::Symbol(ref s) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        println!("found event {}", cur_key);
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
                        println!("found event {}", cur_key);
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
            rules.push(
                Rule {
                    source: vec![*label_a],
                    symbol: *label_b,
                    probability: prob,
                    duration: dur as u64,
                }
                .to_pfa_rule(),
            );

            let mut dur_ev = Event::with_name("transition".to_string());
            dur_ev.params.insert(
                SynthParameter::Duration,
                Box::new(Parameter::with_value(dur)),
            );
            duration_mapping.insert((*label_a, *label_b), dur_ev);
        }
    }

    let pfa = Pfa::<char>::infer_from_rules(&mut rules);

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Atom::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name: name,
            generator: pfa,
            event_mapping: final_mapping,
            duration_mapping: duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}

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
                _ => println!("{}", k),
            }
        }
    }

    let mut duration_mapping = HashMap::new();
    let pistil_exit_prob = 1.0 / (petal_labels.len() - 1) as f32;

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

    // rules to collect ...
    let mut rules = Vec::new();
    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev.params.insert(
        SynthParameter::Duration,
        Box::new(Parameter::with_value(dur)),
    );

    let mut petal_iter = petal_labels.iter();
    for _ in 0..(petal_labels.len() / num_layers) {
        let mut label_a = pistil_label;
        let mut cur_prob = pistil_exit_prob;

        for _ in 0..num_layers {
            if let Some(label_b) = petal_iter.next() {
                rules.push(
                    Rule {
                        source: vec![label_a],
                        symbol: *label_b,
                        probability: cur_prob,
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );

                rules.push(
                    Rule {
                        source: vec![*label_b],
                        symbol: label_a,
                        probability: 0.5,
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );

                duration_mapping.insert((label_a, *label_b), dur_ev.clone());
                duration_mapping.insert((*label_b, label_a), dur_ev.clone());

                label_a = *label_b;

                cur_prob = 0.5;
            }
        }
    }

    let pfa = Pfa::<char>::infer_from_rules(&mut rules);

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Atom::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name: name,
            generator: pfa,
            event_mapping: final_mapping,
            duration_mapping: duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}

pub fn construct_friendship(tail: &mut Vec<Expr>) -> Atom {
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
    let mut center_label: char = '!'; // label chars
    let mut last_char: char = '!'; // label chars
    let mut friends_labels = Vec::new();
    let mut dur = 200.0;
    //let mut num_layers = 1;

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
            friends_labels.push(next_char);
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
                //"layers" => {
                //	if let Some(Expr::Constant(Atom::Float(n))) = tail_drain.next() {
                //	    num_layers = n as usize;
                //	}
                // }
                "center" => {
                    if let Some(Expr::Constant(c)) = tail_drain.next() {
                        let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
                        last_char = next_char;
                        center_label = next_char;
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
                "friends" => {
                    collect_final = true;
                    continue;
                }
                _ => println!("{}", k),
            }
        }
    }

    // first of all check if we have enough friends events
    let needed_friendss = friends_labels.len() % 2;
    if needed_friendss != 0 {
        let last_found = last_char;
        for _ in 0..needed_friendss {
            let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
            last_char = next_char;
            friends_labels.push(next_char);
            let repetition = final_mapping.get(&last_found).unwrap().clone();
            final_mapping.insert(next_char, repetition);
        }
    }

    let mut duration_mapping = HashMap::new();
    let center_exit_prob = 1.0 / (friends_labels.len() / 2) as f32;

    // rules to collect ...
    let mut rules = Vec::new();
    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev.params.insert(
        SynthParameter::Duration,
        Box::new(Parameter::with_value(dur)),
    );

    let mut friends_iter = friends_labels.iter();
    for _ in 0..(friends_labels.len() / 2) {
        if let Some(label_a) = friends_iter.next() {
            if let Some(label_b) = friends_iter.next() {
                rules.push(
                    Rule {
                        source: vec![center_label],
                        symbol: *label_a,
                        probability: center_exit_prob,
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );

                //println!("push rule {} -> {}", center_label, label_a);

                rules.push(
                    Rule {
                        source: vec![*label_a],
                        symbol: *label_b,
                        probability: 1.0,
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );

                //println!("push rule {} -> {}", label_a, label_b);

                rules.push(
                    Rule {
                        source: vec![*label_b],
                        symbol: center_label,
                        probability: 1.0,
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );

                //println!("push rule {} -> {}", label_b, center_label);

                duration_mapping.insert((center_label, *label_a), dur_ev.clone());
                duration_mapping.insert((*label_a, *label_b), dur_ev.clone());
                duration_mapping.insert((*label_b, center_label), dur_ev.clone());
            }
        }
    }

    let pfa = Pfa::<char>::infer_from_rules(&mut rules);

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Atom::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name: name,
            generator: pfa,
            event_mapping: final_mapping,
            duration_mapping: duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}

pub fn construct_rule(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let source_vec: Vec<char> = get_string_from_expr(&tail_drain.next().unwrap())
        .unwrap()
        .chars()
        .collect();
    let sym_vec: Vec<char> = get_string_from_expr(&tail_drain.next().unwrap())
        .unwrap()
        .chars()
        .collect();

    Atom::Rule(Rule {
        source: source_vec,
        symbol: sym_vec[0],
        probability: get_float_from_expr(&tail_drain.next().unwrap()).unwrap() as f32 / 100.0,
        duration: get_float_from_expr(&tail_drain.next().unwrap()).unwrap() as u64,
    })
}

pub fn construct_cycle(
    tail: &mut Vec<Expr>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    let name = if let Some(n) = get_string_from_expr(&tail_drain.next().unwrap()) {
        n
    } else {
        "".to_string()
    };

    let mut dur: f32 = 200.0;
    let mut repetition_chance: f32 = 0.0;
    let mut randomize_chance: f32 = 0.0;
    let mut max_repetitions: f32 = 0.0;

    let mut dur_vec: Vec<f32> = Vec::new();

    let mut collect_events = false;
    let mut collect_template = false;
    let mut template_evs = Vec::new();

    // collect mapped events, i.e. :events 'a (saw 200) ...
    let mut collected_evs = Vec::new();
    let mut collected_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut cur_key: String = "".to_string();

    // collect final events in their position in the cycle
    let mut ev_vecs = Vec::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        if collect_template {
            match c {
                Atom::SoundEvent(e) => {
                    template_evs.push(SourceEvent::Sound(e));
                    continue;
                }
                Atom::ControlEvent(e) => {
                    template_evs.push(SourceEvent::Control(e));
                    continue;
                }
                _ => {
                    collect_template = false;
                }
            }
        }

        if collect_events {
            match c {
                Atom::Symbol(ref s) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        println!("found event {}", cur_key);
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
                        println!("found event {}", cur_key);
                        collected_mapping
                            .insert(cur_key.chars().next().unwrap(), collected_evs.clone());
                    }
                    collect_events = false;
                }
            }
        }

        match c {
            Atom::Keyword(k) => match k.as_str() {
                "dur" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        dur = n;
                    }
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
            Atom::Description(d) => {
                let parsed_cycle = cyc_parser::eval_cyc_from_str(&d, sample_set, out_mode);
                match parsed_cycle {
                    Ok(mut c) => {
                        for mut cyc_evs in c.drain(..) {
                            match *cyc_evs.as_slice() {
                                [Some(Expr::Constant(Atom::Float(f)))] => {
                                    // slice pattern are awesome !
                                    if !dur_vec.is_empty() {
                                        // replace last value, but vec can't start with duration !
                                        *dur_vec.last_mut().unwrap() = f
                                    }
                                }
                                _ => {
                                    ev_vecs.push(Vec::new());
                                    dur_vec.push(dur);
                                    let mut cyc_evs_drain = cyc_evs.drain(..);
                                    while let Some(Some(Expr::Constant(cc))) = cyc_evs_drain.next()
                                    {
                                        match cc {
                                            Atom::Symbol(s) => {
                                                if collected_mapping
                                                    .contains_key(&s.chars().next().unwrap())
                                                {
                                                    ev_vecs.last_mut().unwrap().append(
                                                        collected_mapping
                                                            .get_mut(&s.chars().next().unwrap())
                                                            .unwrap(),
                                                    );
                                                }
                                            }
                                            Atom::SoundEvent(e) => {
                                                ev_vecs
                                                    .last_mut()
                                                    .unwrap()
                                                    .push(SourceEvent::Sound(e));
                                            }
                                            _ => { /* ignore */ }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        println!("couldn't parse cycle: {}", d);
                    }
                }
            }
            _ => println! {"ignored"},
        }
    }

    // generated ids
    let mut last_char: char = '!';
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
        dur_ev.params.insert(
            SynthParameter::Duration,
            Box::new(Parameter::with_value(dur_vec[count])),
        );
        duration_mapping.insert((last_char, next_char), dur_ev);

        if count < num_events - 1 {
            if repetition_chance > 0.0 {
                println!("add rep chance");
                // repetition rule
                rules.push(
                    Rule {
                        source: vec![last_char],
                        symbol: last_char,
                        probability: repetition_chance / 100.0,
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );

                // next rule
                rules.push(
                    Rule {
                        source: vec![last_char],
                        symbol: next_char,
                        probability: 1.0 - (repetition_chance / 100.0),
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );

                // endless repetition allowed per default ...
                if max_repetitions >= 2.0 {
                    let mut max_rep_source = Vec::new();
                    for _ in 0..max_repetitions as usize {
                        max_rep_source.push(last_char);
                    }
                    // max repetition rule
                    rules.push(
                        Rule {
                            source: max_rep_source,
                            symbol: next_char,
                            probability: 1.0,
                            duration: dur as u64,
                        }
                        .to_pfa_rule(),
                    );
                }
            } else {
                rules.push(
                    Rule {
                        source: vec![last_char],
                        symbol: next_char,
                        probability: 1.0,
                        duration: dur as u64,
                    }
                    .to_pfa_rule(),
                );
            }

            last_char = next_char;
        }

        count += 1;
    }

    // if our cycle isn't empty ...
    if count != 0 {
        // close the cycle
        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev.params.insert(
            SynthParameter::Duration,
            Box::new(Parameter::with_value(*dur_vec.last().unwrap())),
        );
        duration_mapping.insert((last_char, first_char), dur_ev);

        rules.push(
            Rule {
                source: vec![last_char],
                symbol: first_char,
                probability: 1.0,
                duration: dur as u64,
            }
            .to_pfa_rule(),
        );
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
            name: name,
            generator: pfa,
            event_mapping: event_mapping,
            duration_mapping: duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}

pub fn handle(
    constructor_type: &BuiltInConstructor,
    tail: &mut Vec<Expr>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
) -> Atom {
    match constructor_type {
        BuiltInConstructor::Infer => construct_infer(tail),
        BuiltInConstructor::Learn => construct_learn(tail),
        BuiltInConstructor::Rule => construct_rule(tail),
        BuiltInConstructor::Nucleus => construct_nucleus(tail),
        BuiltInConstructor::Flower => construct_flower(tail),
        BuiltInConstructor::Friendship => construct_friendship(tail),
        BuiltInConstructor::Fully => construct_fully(tail),
        BuiltInConstructor::Cycle => construct_cycle(tail, sample_set, out_mode),
    }
}
