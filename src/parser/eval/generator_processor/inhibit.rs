use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::HashMap;

use crate::builtin_types::Comparable;
use crate::builtin_types::TypedEntity;
use crate::event::*;
use crate::generator_processor::*;
use crate::parameter::{DynVal, ParameterValue};
use crate::parser::EvaluatedExpr;

// this is basically a shorthand for a pear processor
pub fn collect_inhibit(tail: &mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send + Sync> {
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

    let mut collect_filters = false;
    let mut cur_prob = DynVal::with_value(100.0); // if nothing is specified, it's always or prob 100

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Typed(TypedEntity::SoundEvent(e)) => {
                evs.push(e);
                if collect_filters {
                    collect_filters = false;
                }
            }
            EvaluatedExpr::Keyword(k) => {
                match k.as_str() {
                    "p" => {
                        // save current context, if something has been found
                        if !last_filters.is_empty() {
                            let mut filtered_events = HashMap::new();
                            let mut n_filters = Vec::new();
                            n_filters.extend_from_slice(&last_filters);
                            filtered_events.insert(n_filters, (true, evs.clone()));
                            proc.events_to_be_applied
                                .push((cur_prob.clone(), filtered_events));
                        }

                        // grab new probability
                        cur_prob = match tail_drain.next() {
                            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                                Comparable::Float(f),
                            ))) => DynVal::with_value(f),
                            Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => p,
                            _ => DynVal::with_value(1.0),
                        };
                        collect_filters = false;
                    }
                    "for" => {
                        if !last_filters.is_empty() {
                            let mut filtered_events = HashMap::new();
                            let mut n_filters = Vec::new();
                            n_filters.append(&mut last_filters);
                            filtered_events.insert(n_filters, (true, evs.clone()));
                            proc.events_to_be_applied
                                .push((cur_prob.clone(), filtered_events));
                        }

                        // collect new filters
                        collect_filters = true;
                    }
                    _ => {}
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                if collect_filters {
                    //println!("found filter {}", s);
                    last_filters.push(s)
                }
            }
            _ => {}
        }
    }

    // save last context

    let mut filtered_events = HashMap::new();
    if last_filters.is_empty() {
        last_filters.push("".to_string());
    }
    filtered_events.insert(last_filters.clone(), (true, evs.clone()));
    proc.events_to_be_applied.push((cur_prob, filtered_events));

    Box::new(proc)
}
