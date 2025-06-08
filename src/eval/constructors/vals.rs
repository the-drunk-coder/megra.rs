use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync;

use anyhow::{bail, Result};
use ruffbox_synth::building_blocks::SynthParameterLabel;
use vom_rs::pfa::{Pfa, Rule};

use crate::builtin_types::*;
use crate::eval::{resolver::resolve_globals, EvaluatedExpr, FunctionMap};
use crate::event::*;
use crate::event_helpers::map_parameter;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn vals(
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
        bail!("vals - missing name");
    };

    // the param to be replaced by the values
    let param = match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(n)))) => n,
        Some(EvaluatedExpr::Keyword(n)) => n,
        _ => {
            bail!("vals - missing param descriptor");
        }
    };

    let dur: DynVal = if let TypedEntity::ConfigParameter(ConfigParameter::Numeric(d)) = globals
        .entry(VariableId::DefaultDuration)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Numeric(
            200.0,
        )))
        .value()
    {
        DynVal::with_value(*d)
    } else {
        bail!("vals - global default duration not present");
    };

    let mut ev_vecs = Vec::new();

    let mut keep_root = false;
    let mut randomize_chance: f32 = 0.0;

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
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
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                let mut e = Event::with_name_and_operation(
                    format!("{param}-vals"),
                    EventOperation::Replace,
                );
                e.params.insert(
                    map_parameter(&param),
                    ParameterValue::Scalar(DynVal::with_value(f)),
                );
                ev_vecs.push(vec![SourceEvent::Sound(e)]);
                continue;
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                if param == "keys" {
                    let mut ev =
                        Event::with_name_and_operation("keys".to_string(), EventOperation::Replace);
                    let mut keyword_set = HashSet::new();
                    keyword_set.insert(s);
                    // an "empty" lookup to be merged later down the line ...
                    ev.sample_lookup = Some(crate::sample_set::SampleLookup::Key(
                        "".to_string(),
                        keyword_set,
                    ));
                    ev_vecs.push(vec![SourceEvent::Sound(ev)]);
                    continue;
                } else if param == "art" || param == "articulation" {
                    // currently handled explicitly here, even though that's
                    // a bit clunky ... a more "generic" way to do this would be
                    // nice
                    let mut ev = Event::with_name_and_operation(
                        "articulation".to_string(),
                        EventOperation::Replace,
                    );
                    ev.params
                        .insert(map_parameter(&param), ParameterValue::Symbolic(s));
                    ev_vecs.push(vec![SourceEvent::Sound(ev)]);
                    continue;
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                let mut e = Event::with_name_and_operation(
                    format!("{param}-mul"),
                    EventOperation::Multiply,
                );
                e.params
                    .insert(map_parameter(&param), ParameterValue::Scalar(p));
                ev_vecs.push(vec![SourceEvent::Sound(e)]);
                continue;
            }
            _ => {}
        }
    }

    if ev_vecs.is_empty() {
        bail!("vals - missing value arguments");
    }

    /////////////////////////////////
    // assemble rules and mappings //
    /////////////////////////////////

    let mut event_mapping = BTreeMap::<char, (Vec<SourceEvent>, Event)>::new();

    let mut dur_ev = Event::with_name("transition".to_string());
    dur_ev.params.insert(
        SynthParameterLabel::Duration.into(),
        ParameterValue::Scalar(dur.clone()),
    );

    let pfa = if !keep_root {
        // generated ids
        let mut last_char: char = '1';
        let first_char: char = last_char;

        // collect cycle rules
        let mut rules = Vec::new();
        let len = ev_vecs.len() - 1;

        for (count, ev) in ev_vecs.drain(..).enumerate() {
            event_mapping.insert(last_char, (ev, dur_ev.clone()));

            if count < len {
                let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();

                let mut dur_ev = Event::with_name("transition".to_string());
                dur_ev.params.insert(
                    SynthParameterLabel::Duration.into(),
                    ParameterValue::Scalar(dur.clone()),
                );

                rules.push(Rule {
                    source: vec![last_char],
                    symbol: next_char,
                    probability: 1.0,
                });

                last_char = next_char;
            }
        }

        // close the loop
        rules.push(Rule {
            source: vec![last_char],
            symbol: first_char,
            probability: 1.0,
        });

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
            label_mapping: None,
            event_mapping,
            override_durations: None,
            modified: true,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
        time_shift: 0,
        keep_root,
    })))
}
