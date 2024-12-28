use crate::builtin_types::*;
use crate::eval::resolver::resolve_globals;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;

use anyhow::bail;
use anyhow::Result;
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::eval::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn stages(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    // eval-time resolve
    // ignore function name
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(n)))) =
        tail_drain.next()
    {
        n
    } else {
        bail!("stages - missing name");
    };

    let mut collected_evs = Vec::new();

    let mut dur: DynVal = if let TypedEntity::ConfigParameter(ConfigParameter::Numeric(d)) = globals
        .entry(VariableId::DefaultDuration)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Numeric(
            200.0,
        )))
        .value()
    {
        DynVal::with_value(*d)
    } else {
        bail!("stages - global default duration not present");
    };

    let mut keep_root = false;
    let mut randomize_chance: f32 = 0.0;
    let mut pnext: f32 = 0.0;
    let mut pprev: f32 = 0.0;
    let mut cyclical = false;
    let mut time_shift = 0;

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "cyc" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        cyclical = b;
                    }
                }
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(n)))) => {
                        dur = DynVal::with_value(n);
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                "shift" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        time_shift = n as i32;
                        tail_drain.next();
                    }
                }
                "pnext" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        pnext = n / 100.0;
                    }
                }
                "pprev" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        pprev = n / 100.0;
                    }
                }
                "rnd" => {
                    if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(n))) =
                        tail_drain.next().unwrap()
                    {
                        randomize_chance = n;
                    }
                }
                "keep" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        keep_root = b;
                    }
                }
                _ => println!("{k}"),
            },
            EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                collected_evs.push(SourceEvent::Sound(e));
                continue;
            }
            EvaluatedExpr::Typed(TypedEntity::ControlEvent(e)) => {
                collected_evs.push(SourceEvent::Control(e));
                continue;
            }
            _ => {}
        }
    }

    /////////////////////////////////
    // assemble rules and mappings //
    /////////////////////////////////

    let mut event_mapping = BTreeMap::<char, (Vec<SourceEvent>, Event)>::new();

    let pfa = if !keep_root {
        let mut last_char: char = '1'; // label chars
        let mut labels = Vec::new();

        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev.params.insert(
            SynthParameterLabel::Duration.into(),
            ParameterValue::Scalar(dur.clone()),
        );

        for ev in collected_evs.drain(..) {
            event_mapping.insert(last_char, (vec![ev], dur_ev.clone()));
            labels.push(vec![last_char]);
            last_char = std::char::from_u32(last_char as u32 + 1).unwrap();
        }

        // rules to collect ...
        let mut rules = Vec::new();

        if labels.len() == 1 {
            rules.push(Rule {
                source: labels[0].clone(),
                symbol: labels[0][0],
                probability: 1.0,
            });
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
                    }
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
                    }
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

    Ok(EvaluatedExpr::Typed(TypedEntity::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            label_mapping: None,
            override_durations: None,
            modified: true,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
        time_shift,
        keep_root,
    })))
}
