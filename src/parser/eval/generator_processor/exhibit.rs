use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::HashMap;

use crate::builtin_types::Comparable;
use crate::builtin_types::TypedEntity;
use crate::event::*;
use crate::generator_processor::*;
use crate::parameter::{DynVal, ParameterValue};
use crate::parser::EvaluatedExpr;

// this is basically a shorthand for a pear processor
pub fn collect_exhibit(tail: &mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send + Sync> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // skip function name

    let mut proc = PearProcessor::new();

    let mut last_filters = Vec::new();

    let mut evs = Vec::new();
    let mut silencer = Event::with_name("silencer".to_string());
    silencer.params.insert(
        SynthParameterLabel::EnvelopeLevel.into(),
        ParameterValue::Scalar(DynVal::with_value(0.0)),
    );
    evs.push(silencer);

    let mut cur_prob = DynVal::with_value(100.0); // if nothing is specified, it's always or prob 100

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
            cur_prob = DynVal::with_value(f);
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
            cur_prob = p;
        }
        _ => {}
    }

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                last_filters.push(s)
            }
            _ => {}
        }
    }

    let mut filtered_events = HashMap::new();
    if last_filters.is_empty() {
        last_filters.push("".to_string());
    }
    filtered_events.insert(last_filters.clone(), (false, evs.clone()));
    proc.events_to_be_applied.push((cur_prob, filtered_events));

    Box::new(proc)
}
