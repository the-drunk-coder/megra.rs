use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use anyhow::bail;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::eval::{resolver::resolve_globals, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn fully(
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
        bail!("fully - name not specified");
    };

    //let mut collect_labeled = false;

    //let mut collected_evs = Vec::new();
    //let mut collected_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    //let mut cur_key: String = "".to_string();

    let mut keep_root = false;

    let mut event_mapping = BTreeMap::new();
    let mut last_char: char = 'a'; // label chars
    let mut labels = Vec::new();
    let mut time_shift = 0;

    let mut dur: DynVal = if let TypedEntity::ConfigParameter(ConfigParameter::Numeric(d)) = globals
        .entry(VariableId::DefaultDuration)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Numeric(
            200.0,
        )))
        .value()
    {
        DynVal::with_value(*d)
    } else {
        bail!("fully - global default duration not available");
    };

    while let Some(c) = tail_drain.next() {
        /*if collect_labeled {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(ref s))) => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        //println!("found event {}", cur_key);
                        collected_mapping
                            .insert(cur_key.chars().next().unwrap(), collected_evs.clone());
                        collected_evs.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                    collected_evs.push(SourceEvent::Sound(e));
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::ControlEvent(e)) => {
                    collected_evs.push(SourceEvent::Control(e));
                    continue;
                }
                _ => {
                    if !cur_key.is_empty() && !collected_evs.is_empty() {
                        //println!("found event {}", cur_key);
                        collected_mapping
                            .insert(cur_key.chars().next().unwrap(), collected_evs.clone());
                    }
                    collect_labeled = false;
                }
            }
        }*/

        match c {
            /*EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(ref s))) => {
                let mut final_vec = Vec::new();
                let label = s.chars().next().unwrap();
                if collected_mapping.contains_key(&label) {
                    final_vec.append(&mut collected_mapping.get(&label).unwrap().clone());
                }
                event_mapping.insert(label, final_vec);
            }*/
            EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
                last_char = next_char;
                labels.push(next_char);
                let final_vec = vec![SourceEvent::Sound(e)];
                event_mapping.insert(next_char, (final_vec, Event::transition(dur.clone())));
            }
            EvaluatedExpr::Typed(TypedEntity::ControlEvent(e)) => {
                let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();
                last_char = next_char;
                labels.push(next_char);
                let final_vec = vec![SourceEvent::Control(e)];
                event_mapping.insert(next_char, (final_vec, Event::transition(dur.clone())));
            }
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "dur" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(n)))) => {
                        dur = DynVal::with_value(n);
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
                        dur = p;
                    }
                    _ => {}
                },
                //"events" => {
                //    collect_labeled = true;
                //    continue;
                //}
                "shift" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        time_shift = n as i32;
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
            _ => {}
        }
    }

    let pfa = if !keep_root {
        let prob = 1.0 / (labels.len() - 1) as f32;
        // rules to collect ...
        let mut rules = Vec::new();
        for label_a in labels.iter() {
            for label_b in labels.iter() {
                rules.push(Rule {
                    source: vec![*label_a],
                    symbol: *label_b,
                    probability: prob,
                });
            }
        }

        Pfa::<char>::infer_from_rules(&mut rules, true)
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
