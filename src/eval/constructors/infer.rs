use crate::builtin_types::*;
use crate::duration_tree::{add_leaf, DurationTreeNode};
use crate::eval::resolver::resolve_globals;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::{MarkovSequenceGenerator, Rule};
use crate::parameter::*;

use anyhow::bail;
use anyhow::Result;
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa;

use crate::eval::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn rule(
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

    let source_vec: Vec<char> =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
            tail_drain.next()
        {
            s.chars().collect()
        } else {
            bail!("rule - missing source");
        };

    let sym_vec: Vec<char> =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
            tail_drain.next()
        {
            s.chars().collect()
        } else {
            bail!("rule - missing symbol");
        };

    let def_dur: f32 = if let TypedEntity::ConfigParameter(ConfigParameter::Numeric(d)) = globals
        .entry(VariableId::DefaultDuration)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Numeric(
            200.0,
        )))
        .value()
    {
        *d
    } else {
        bail!("rule - missing global default duration");
    };

    let probability =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(p)))) =
            tail_drain.next()
        {
            p / 100.0
        } else {
            1.0
        };

    let duration =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
            tail_drain.next()
        {
            f as u64
        } else {
            def_dur as u64
        };

    Ok(EvaluatedExpr::Typed(TypedEntity::Rule(Rule {
        source: source_vec,
        symbol: sym_vec[0],
        probability,
        duration,
    })))
}

pub fn infer(
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
        bail!("infer - missing name");
    };

    let mut event_mapping = BTreeMap::<char, (Vec<SourceEvent>, Event)>::new();
    let mut override_durations = DurationTreeNode::new(&vec![], None);

    let mut rules = Vec::new();

    let mut collect_events = false;
    let mut collect_rules = false;
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
        bail!("infer - global default duration not present");
    };

    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev.params.insert(
        SynthParameterLabel::Duration.into(),
        ParameterValue::Scalar(dur.clone()),
    );

    let mut ev_vec = Vec::new();
    let mut cur_key: String = "".to_string();
    let mut keep_root = false;

    while let Some(c) = tail_drain.next() {
        if collect_events {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(ref s))) => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        //println!("found event {}", cur_key);
                        event_mapping.insert(
                            cur_key.chars().next().unwrap(),
                            (ev_vec.clone(), dur_ev.clone()),
                        );
                        ev_vec.clear();
                    }
                    cur_key = s.clone();
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                    ev_vec.push(SourceEvent::Sound(e));
                    continue;
                }
                EvaluatedExpr::Typed(TypedEntity::ControlEvent(e)) => {
                    ev_vec.push(SourceEvent::Control(e));
                    continue;
                }
                _ => {
                    if !cur_key.is_empty() && !ev_vec.is_empty() {
                        //println!("found event {}", cur_key);
                        event_mapping.insert(
                            cur_key.chars().next().unwrap(),
                            (ev_vec.clone(), dur_ev.clone()),
                        );
                    }
                    collect_events = false;
                }
            }
        }

        if collect_rules {
            if let EvaluatedExpr::Typed(TypedEntity::Rule(s)) = c {
                let mut label = s.source.clone();
                label.push(s.symbol);

                add_leaf(&mut override_durations, &label, Some(s.duration));

                rules.push(s.to_pfa_rule());
                continue;
            } else {
                collect_rules = false;
            }
        }

        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "rules" => {
                    collect_rules = true;
                    continue;
                }
                "events" => {
                    collect_events = true;
                    continue;
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
            _ => println! {"ignored"},
        }
    }

    // only re-generate if necessary
    let pfa = if !keep_root {
        pfa::Pfa::<char>::infer_from_rules(&mut rules, true)
    } else {
        pfa::Pfa::<char>::new()
    };

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Ok(EvaluatedExpr::Typed(TypedEntity::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa, // will be empty if we intend on keeping the root generator
            event_mapping,
            override_durations: Some(override_durations),
            label_mapping: None,
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
