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

pub fn stages(
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

    let mut collected_evs = Vec::new();

    let mut dur: Parameter = if let ConfigParameter::Numeric(d) = global_parameters
        .entry(BuiltinGlobalParameters::DefaultDuration)
        .or_insert(ConfigParameter::Numeric(200.0))
        .value()
    {
        Parameter::with_value(*d)
    } else {
        unreachable!()
    };

    let mut randomize_chance: f32 = 0.0;
    let mut pnext: f32 = 0.0;
    let mut pprev: f32 = 0.0;
    let mut cyclical = false;

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "cyc" => {
                    if let Some(EvaluatedExpr::Boolean(b)) = tail_drain.next() {
                        cyclical = b;
                    }
                }
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Float(n)) => {
                        dur = Parameter::with_value(n);
                    }
                    Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "pnext" => {
                    if let Some(EvaluatedExpr::Float(n)) = tail_drain.next() {
                        pnext = n / 100.0;
                    }
                }
                "pprev" => {
                    if let Some(EvaluatedExpr::Float(n)) = tail_drain.next() {
                        pprev = n / 100.0;
                    }
                }
                "rnd" => {
                    if let EvaluatedExpr::Float(n) = tail_drain.next().unwrap() {
                        randomize_chance = n;
                    }
                }
                _ => println!("{}", k),
            },
            EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e)) => {
                collected_evs.push(SourceEvent::Sound(e));
                continue;
            }
            EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(e)) => {
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
        .insert(SynthParameter::Duration, Box::new(dur.clone()));

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

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
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
