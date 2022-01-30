use std::collections::HashMap;

use crate::builtin_types::*;
use crate::generator_processor::*;
use crate::parameter::Parameter;
use crate::parser::{BuiltIn, EvaluatedExpr};

pub fn collect_every(tail: &mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // skip function name

    let mut proc = EveryProcessor::new();

    let mut last_filters = Vec::new();

    let mut cur_step = Parameter::with_value(1.0); // if nothing is specified, it's always applied
    let mut gen_mod_funs = Vec::new();
    let mut events = Vec::new();
    let mut collect_filters = false;

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf),
            )) => {
                gen_mod_funs.push(gmf);
                collect_filters = false;
            }
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorModifierList(mut ml)) => {
                for gpom in ml.drain(..) {
                    if let GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf) = gpom {
                        gen_mod_funs.push(gmf);
                    }
                }
                collect_filters = false;
            }
            EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e)) => {
                events.push(e);
                collect_filters = false;
            }
            EvaluatedExpr::Symbol(s) => {
                if collect_filters {
                    last_filters.push(s)
                }
            }
            EvaluatedExpr::Keyword(k) => {
                match k.as_str() {
                    "for" => {
                        if !events.is_empty() || !gen_mod_funs.is_empty() {
                            let mut n_mods = Vec::new();
                            n_mods.append(&mut gen_mod_funs);

                            let mut filtered_events = HashMap::new();
                            let mut n_evs = Vec::new();
                            let mut n_filters = Vec::new();
                            n_evs.append(&mut events);
                            n_filters.append(&mut last_filters);
                            if n_filters.is_empty() {
                                n_filters.push("".to_string());
                            }
                            filtered_events.insert(n_filters, (true, n_evs));

                            proc.things_to_be_applied.push((
                                cur_step.clone(),
                                filtered_events,
                                n_mods,
                            ));
                        } else {
                            last_filters.clear();
                        }
                        // collect new filters
                        collect_filters = true;
                    }
                    "n" => {
                        if !events.is_empty() || !gen_mod_funs.is_empty() {
                            let mut n_mods = Vec::new();
                            n_mods.append(&mut gen_mod_funs);

                            let mut filtered_events = HashMap::new();
                            let mut n_evs = Vec::new();
                            let mut n_filters = Vec::new();
                            n_evs.append(&mut events);
                            n_filters.append(&mut last_filters);
                            if n_filters.is_empty() {
                                n_filters.push("".to_string());
                            }
                            filtered_events.insert(n_filters, (true, n_evs));

                            proc.things_to_be_applied.push((
                                cur_step.clone(),
                                filtered_events,
                                n_mods,
                            ));
                        }
                        // grab new probability
                        cur_step = match tail_drain.next() {
                            Some(EvaluatedExpr::Float(f)) => Parameter::with_value(f),
                            Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p))) => p,
                            _ => Parameter::with_value(1.0),
                        };

                        collect_filters = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // save last context
    if !events.is_empty() || !gen_mod_funs.is_empty() {
        let mut filtered_events = HashMap::new();
        if last_filters.is_empty() {
            last_filters.push("".to_string());
        }
        filtered_events.insert(last_filters, (true, events));
        proc.things_to_be_applied
            .push((cur_step, filtered_events, gen_mod_funs));
    }

    Box::new(proc)
}
