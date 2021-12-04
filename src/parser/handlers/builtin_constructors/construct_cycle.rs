use crate::builtin_types::*;
use crate::cyc_parser;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use crate::sample_set::SampleSet;
use crate::session::OutputMode;
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

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

    let mut dur: Option<Parameter> = Some(Parameter::with_value(200.0));
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

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        if collect_template {
            match c {
                Atom::Symbol(s) => {
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
                Atom::Symbol(ref s) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        //println!("found event {}", cur_key);
                        collected_mapping.insert(cur_key.clone(), collected_evs.clone());
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
                        //println!("found event {}", cur_key);
                        collected_mapping.insert(cur_key.clone(), collected_evs.clone());
                    }
                    collect_events = false;
                }
            }
        }

        match c {
            Atom::Keyword(k) => match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(Expr::Constant(Atom::Float(n))) => {
                        dur = Some(Parameter::with_value(n));
                    }
                    Some(Expr::Constant(Atom::Parameter(p))) => {
                        dur = Some(p);
                    }
                    _ => {}
                },
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
                cycle_string = d.clone();
            }
            _ => println! {"ignored"},
        }
    }

    let mut parsed_cycle = cyc_parser::eval_cyc_from_str(
        &cycle_string,
        sample_set,
        out_mode,
        &template_evs,
        &collected_mapping,
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
                dur_vec.push(dur.clone().unwrap());
                let mut cyc_evs_drain = cyc_evs.drain(..);
                match cyc_evs_drain.next() {
                    Some(cyc_parser::CycleResult::SoundEvent(s)) => {
                        pos_vec.push(SourceEvent::Sound(s))
                    }
                    Some(cyc_parser::CycleResult::ControlEvent(c)) => {
                        pos_vec.push(SourceEvent::Control(c))
                    }
                    _ => {}
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
            } else {
                if count == num_events - 1 {
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
            event_mapping,
            duration_mapping,
            modified: false,
            symbol_ages: HashMap::new(),
            default_duration: dur.unwrap().static_val as u64,
            last_transition: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
    })
}
