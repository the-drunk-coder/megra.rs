use crate::builtin_types::*;
use crate::eval::resolver::resolve_globals;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;

use anyhow::{bail, Result};
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::eval::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn nuc(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(n)))) =
        tail_drain.next()
    {
        n
    } else {
        bail!("nuc - missing name");
    };

    let mut event_mapping = BTreeMap::<char, (Vec<SourceEvent>, Event)>::new();

    let mut rules = Vec::new();
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
        bail!("nuc - global default duration not present");
    };

    let mut ev_vec = Vec::new();
    let mut keep_root = false;

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => ev_vec.push(SourceEvent::Sound(e)),
            EvaluatedExpr::Typed(TypedEntity::ControlEvent(c)) => {
                ev_vec.push(SourceEvent::Control(c))
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
                "keep" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        keep_root = b;
                    }
                }
                "shift" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        time_shift = n as i32;
                        tail_drain.next();
                    }
                }
                _ => println!("{k}"),
            },
            // resolve vec
            EvaluatedExpr::Typed(TypedEntity::Vec(v)) => {
                for x in v {
                    match *x {
                        TypedEntity::SoundEvent(e) => {
                            ev_vec.push(SourceEvent::Sound(e));
                        }
                        TypedEntity::ControlEvent(e) => {
                            ev_vec.push(SourceEvent::Control(e));
                        }
                        _ => {}
                    }
                }
            }
            _ => println! {"ignored"},
        }
    }

    // only re-generate if necessary ...
    let pfa = if !keep_root {
        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev.params.insert(
            SynthParameterLabel::Duration.into(),
            ParameterValue::Scalar(dur.clone()),
        );

        event_mapping.insert('a', (ev_vec, dur_ev));

        // one rule to rule them all
        rules.push(Rule {
            source: vec!['a'],
            symbol: 'a',
            probability: 1.0,
        });

        Pfa::<char>::infer_from_rules(&mut rules, false)
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
            override_durations: None,
            event_mapping,
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_eval_nuc() {
        let snippet = "(nuc 'da (bd))";
        let functions = FunctionMap::new();
        let sample_set = SampleAndWavematrixSet::new();

        functions
            .std_lib
            .insert("nuc".to_string(), crate::eval::constructors::nuc::nuc);
        functions.std_lib.insert("bd".to_string(), |_, _, _, _, _| {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::String("bd".to_string()),
            )))
        });

        let globals = sync::Arc::new(GlobalVariables::new());

        match crate::eval::parse_and_eval_from_str(
            snippet,
            &functions,
            &globals,
            sample_set,
            OutputMode::Stereo,
        ) {
            Ok(res) => {
                assert!(matches!(
                    res,
                    EvaluatedExpr::Typed(TypedEntity::Generator(_))
                ));
            }
            Err(e) => {
                println!("err {e}");
                panic!();
            }
        }
    }
}
