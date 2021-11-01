use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use std::collections::{BTreeSet, HashMap};
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

    let mut dur: Option<Parameter> = Some(Parameter::with_value(200.0));

    let mut ev_vec = Vec::new();
    let mut cur_key: String = "".to_string();

    let mut autosilence = true;

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
			sample.retain(|c| !c.is_whitespace());
                    }
                }
                "events" => {
                    collect_events = true;
                    continue;
                }
                "dur" => match tail_drain.next() {
                    Some(Expr::Constant(Atom::Float(n))) => {
                        dur = Some(Parameter::with_value(n));
                    }
                    Some(Expr::Constant(Atom::Parameter(p))) => {
                        dur = Some(p);
                    }
                    _ => {}
                },
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
            name,
            generator: pfa,
            event_mapping,
            duration_mapping: HashMap::new(),
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur.unwrap().static_val as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}
