use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::{MarkovSequenceGenerator, Rule};
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use vom_rs::pfa::Pfa;

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
            name,
            generator: pfa,
            event_mapping,
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
