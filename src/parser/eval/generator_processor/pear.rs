use std::collections::HashMap;

use crate::generator_processor::*;
use crate::parameter::DynVal;

use crate::parser::{BuiltIn, EvaluatedExpr};

pub fn collect_pear(tail: &mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send> {
    let mut tail_drain = tail.drain(..).skip(1); // skip function name

    let mut proc = PearProcessor::new();

    let mut last_filters = Vec::new();

    let mut evs = Vec::new();
    let mut collect_filters = false;
    let mut cur_prob = DynVal::with_value(100.0); // if nothing is specified, it's always or prob 100

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e)) => {
                evs.push(e);
                if collect_filters {
                    collect_filters = false;
                }
            }
            EvaluatedExpr::Keyword(k) => {
                match k.as_str() {
                    "p" => {
                        // save current context, if something has been found
                        if !evs.is_empty() {
                            let mut filtered_events = HashMap::new();
                            let mut n_evs = Vec::new();
                            let mut n_filters = Vec::new();
                            n_evs.append(&mut evs);
                            //println!("last filters {:?}", last_filters);
                            n_filters.extend_from_slice(&last_filters);
                            if n_filters.is_empty() {
                                n_filters.push("".to_string());
                            }
                            filtered_events.insert(n_filters, (true, n_evs));
                            proc.events_to_be_applied
                                .push((cur_prob.clone(), filtered_events));
                        }
                        // grab new probability

                        cur_prob = match tail_drain.next() {
                            Some(EvaluatedExpr::Float(f)) => DynVal::with_value(f),
                            Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p))) => p,
                            _ => DynVal::with_value(1.0),
                        };

                        collect_filters = false;
                    }
                    "for" => {
                        if !evs.is_empty() {
                            let mut filtered_events = HashMap::new();
                            let mut n_evs = Vec::new();
                            let mut n_filters = Vec::new();
                            n_evs.append(&mut evs);
                            n_filters.append(&mut last_filters);
                            if n_filters.is_empty() {
                                n_filters.push("".to_string());
                            }
                            filtered_events.insert(n_filters, (true, n_evs));
                            proc.events_to_be_applied
                                .push((cur_prob.clone(), filtered_events));
                        } else {
                            last_filters.clear();
                        }
                        // collect new filters
                        collect_filters = true;
                    }
                    _ => {}
                }
            }
            EvaluatedExpr::Symbol(s) => {
                if collect_filters {
                    //println!("found filter {}", s);
                    last_filters.push(s)
                }
            }
            _ => {}
        }
    }

    // save last context
    if !evs.is_empty() {
        let mut filtered_events = HashMap::new();
        if last_filters.is_empty() {
            last_filters.push("".to_string());
        }
        filtered_events.insert(last_filters, (true, evs));
        proc.events_to_be_applied.push((cur_prob, filtered_events));
    }

    Box::new(proc)
}
