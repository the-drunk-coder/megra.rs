use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::{BTreeSet, HashMap};
use vom_rs::pfa::{Pfa, Rule};

pub fn construct_stages(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    let name = if let Some(n) = get_string_from_expr(&tail_drain.next().unwrap()) {
        n
    } else {
        "".to_string()
    };

    let mut collected_evs = Vec::new();

    let mut dur: Option<Parameter> = Some(Parameter::with_value(200.0));

    let mut randomize_chance: f32 = 0.0;
    let mut pnext: f32 = 0.0;
    let mut pprev: f32 = 0.0;
    let mut cyclical = false;

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::Keyword(k) => match k.as_str() {
                "cyc" => {
                    if let Some(b) = get_bool_from_expr_opt(&tail_drain.next()) {
                        cyclical = b;
                    }
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
                "pnext" => {
                    if let Some(Expr::Constant(Atom::Float(n))) = tail_drain.next() {
                        pnext = n / 100.0;
                    }
                }
                "pprev" => {
                    if let Some(Expr::Constant(Atom::Float(n))) = tail_drain.next() {
                        pprev = n / 100.0;
                    }
                }
                "rnd" => {
                    if let Expr::Constant(Atom::Float(n)) = tail_drain.next().unwrap() {
                        randomize_chance = n;
                    }
                }
                _ => println!("{}", k),
            },
            Atom::SoundEvent(e) => {
                collected_evs.push(SourceEvent::Sound(e));
                continue;
            }
            Atom::ControlEvent(e) => {
                collected_evs.push(SourceEvent::Control(e));
                continue;
            }
            _ => {}
        }
    }

    /////////////////////////////////
    // assemble rules and mappings //
    /////////////////////////////////

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut last_char: char = '1'; // label chars
    let mut labels = Vec::new();
    for ev in collected_evs.drain(..) {
        event_mapping.insert(last_char, vec![ev]);
        labels.push(vec![last_char]);
        last_char = std::char::from_u32(last_char as u32 + 1).unwrap();
    }

    // rules to collect ...
    let mut rules = Vec::new();
    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev
        .params
        .insert(SynthParameter::Duration, Box::new(dur.clone().unwrap()));

    let mut duration_mapping = HashMap::new();

    if labels.len() == 1 {
        rules.push(Rule {
            source: labels[0].clone(),
            symbol: labels[0][0],
            probability: 1.0,
        });
        duration_mapping.insert((labels[0][0], labels[0][0]), dur_ev.clone());
    } else if labels.len() == 2 {
        rules.push(Rule {
            source: labels[0].clone(),
            symbol: labels[0][0],
            probability: 1.0 - pnext,
        });
        rules.push(Rule {
            source: labels[1].clone(),
            symbol: labels[1][0],
            probability: 1.0 - pnext,
        });
        rules.push(Rule {
            source: labels[0].clone(),
            symbol: labels[1][0],
            probability: pnext,
        });
        rules.push(Rule {
            source: labels[1].clone(),
            symbol: labels[0][0],
            probability: pnext,
        });
        duration_mapping.insert((labels[0][0], labels[0][0]), dur_ev.clone());
        duration_mapping.insert((labels[0][0], labels[1][0]), dur_ev.clone());
        duration_mapping.insert((labels[1][0], labels[0][0]), dur_ev.clone());
        duration_mapping.insert((labels[1][0], labels[1][0]), dur_ev.clone());
    } else {
        for (i, _) in labels.iter().enumerate() {
            if i == 0 {
                rules.push(Rule {
                    source: labels[i].clone(),
                    symbol: labels[i][0],
                    probability: if cyclical {
                        1.0 - pnext - pprev
                    } else {
                        1.0 - pnext
                    },
                });

                rules.push(Rule {
                    source: labels[i].clone(),
                    symbol: labels[i + 1][0],
                    probability: pnext,
                });

                if cyclical {
                    rules.push(Rule {
                        source: labels[i].clone(),
                        symbol: labels.last().unwrap()[0], // if labels are empty this shouldn't be reached
                        probability: pnext,
                    });

                    duration_mapping
                        .insert((labels[i][0], labels.last().unwrap()[0]), dur_ev.clone());
                }

                duration_mapping.insert((labels[i][0], labels[i][0]), dur_ev.clone());
                duration_mapping.insert((labels[i][0], labels[i + 1][0]), dur_ev.clone());
            } else if i == labels.len() - 1 {
                rules.push(Rule {
                    source: labels[i].clone(),
                    symbol: labels[i][0],
                    probability: if cyclical {
                        1.0 - pnext - pprev
                    } else {
                        1.0 - pprev
                    },
                });

                rules.push(Rule {
                    source: labels[i].clone(),
                    symbol: labels[i - 1][0],
                    probability: pprev,
                });

                if cyclical {
                    rules.push(Rule {
                        source: labels[i].clone(),
                        symbol: labels.first().unwrap()[0], // if labels are empty this shouldn't be reached
                        probability: pnext,
                    });
                    duration_mapping
                        .insert((labels[i][0], labels.first().unwrap()[0]), dur_ev.clone());
                }

                duration_mapping.insert((labels[i][0], labels[i][0]), dur_ev.clone());
                duration_mapping.insert((labels[i][0], labels[i - 1][0]), dur_ev.clone());
            } else {
                rules.push(Rule {
                    source: labels[i].clone(),
                    symbol: labels[i][0],
                    probability: 1.0 - pnext - pprev,
                });
                rules.push(Rule {
                    source: labels[i].clone(),
                    symbol: labels[i + 1][0],
                    probability: pnext,
                });
                rules.push(Rule {
                    source: labels[i].clone(),
                    symbol: labels[i - 1][0],
                    probability: pprev,
                });
                duration_mapping.insert((labels[i][0], labels[i][0]), dur_ev.clone());
                duration_mapping.insert((labels[i][0], labels[i + 1][0]), dur_ev.clone());
                duration_mapping.insert((labels[i][0], labels[i - 1][0]), dur_ev.clone());
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
