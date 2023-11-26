use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync;

use ruffbox_synth::building_blocks::SynthParameterLabel;
use vom_rs::pfa::{Pfa, Rule};

use crate::builtin_types::*;
use crate::event::*;
use crate::event_helpers::map_parameter;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;
use crate::parser::eval::resolver::resolve_globals;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn vals(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
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
        return None;
    };

    // the param to be factorized
    let param = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(n)))) =
        tail_drain.next()
    {
        n
    } else {
        return None;
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
        unreachable!()
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
                    map_parameter(&param).into(),
                    ParameterValue::Scalar(DynVal::with_value(f)),
                );
                ev_vecs.push(vec![SourceEvent::Sound(e)]);
                continue;
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                let mut e = Event::with_name_and_operation(
                    format!("{param}-mul"),
                    EventOperation::Multiply,
                );
                e.params
                    .insert(map_parameter(&param).into(), ParameterValue::Scalar(p));
                ev_vecs.push(vec![SourceEvent::Sound(e)]);
                continue;
            }
            _ => {}
        }
    }

    if ev_vecs.is_empty() {
        return None;
    }

    /////////////////////////////////
    // assemble rules and mappings //
    /////////////////////////////////

    let mut event_mapping = BTreeMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::new();

    let pfa = if !keep_root {
        // generated ids
        let mut last_char: char = '1';
        let first_char: char = last_char;

        // collect cycle rules
        let mut rules = Vec::new();
        let len = ev_vecs.len() - 1;

        for (count, ev) in ev_vecs.drain(..).enumerate() {
            event_mapping.insert(last_char, ev);

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

                duration_mapping.insert((last_char, next_char), dur_ev);

                last_char = next_char;
            }
        }

        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev.params.insert(
            SynthParameterLabel::Duration.into(),
            ParameterValue::Scalar(dur.clone()),
        );

        // close the loop
        rules.push(Rule {
            source: vec![last_char],
            symbol: first_char,
            probability: 1.0,
        });

        duration_mapping.insert((last_char, first_char), dur_ev);

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

    Some(EvaluatedExpr::Typed(TypedEntity::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            label_mapping: None,
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
        keep_root,
    })))
}
