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
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::Pfa;

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
                //println!("add rep chance");
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
