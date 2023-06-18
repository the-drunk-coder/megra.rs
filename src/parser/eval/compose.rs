use crate::builtin_types::*;
use crate::generator_processor::GeneratorWrapperProcessor;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use std::sync;

fn collect_compose(tail: &mut Vec<EvaluatedExpr>) -> Vec<GeneratorProcessorOrModifier> {
    let mut gen_procs = Vec::new();
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // skip function name

    for c in tail_drain {
        match c {
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(gp)) => {
                gen_procs.push(gp);
            }
            EvaluatedExpr::Typed(TypedEntity::Generator(g)) => {
                gen_procs.push(GeneratorProcessorOrModifier::GeneratorProcessor(Box::new(
                    GeneratorWrapperProcessor::with_generator(g),
                )));
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(mut gpl)) => {
                gen_procs.append(&mut gpl);
            }
            _ => {}
        }
    }
    gen_procs
}

pub fn compose(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let last = tail.pop();
    Some(match last {
        Some(EvaluatedExpr::Typed(TypedEntity::Symbol(s))) => EvaluatedExpr::Typed(
            TypedEntity::PartProxy(PartProxy::Proxy(s, collect_compose(tail))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::PartProxy(PartProxy::Proxy(s, mut proxy_mods)))) => {
            proxy_mods.append(&mut collect_compose(tail));
            EvaluatedExpr::Typed(TypedEntity::PartProxy(PartProxy::Proxy(s, proxy_mods)))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::ProxyList(mut l))) => {
            let gp = collect_compose(tail);
            let mut pdrain = l.drain(..);
            let mut new_list = Vec::new();
            while let Some(PartProxy::Proxy(s, mut proxy_mods)) = pdrain.next() {
                proxy_mods.append(&mut gp.clone());
                new_list.push(PartProxy::Proxy(s, proxy_mods));
            }
            EvaluatedExpr::Typed(TypedEntity::ProxyList(new_list))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Generator(mut g))) => {
            let mut proc_or_mods = collect_compose(tail);
            let mut procs = Vec::new();

            for gpom in proc_or_mods.drain(..) {
                match gpom {
                    GeneratorProcessorOrModifier::GeneratorProcessor(gp) => {
                        procs.push((gp.get_id(), gp))
                    }
                    GeneratorProcessorOrModifier::GeneratorModifierFunction((fun, pos, named)) => {
                        fun(&mut g, &pos, &named, globals)
                    }
                }
            }

            g.processors.append(&mut procs);
            EvaluatedExpr::Typed(TypedEntity::Generator(g))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorList(mut gl))) => {
            let gp = collect_compose(tail);
            for gen in gl.iter_mut() {
                for gpom in gp.iter() {
                    match gpom {
                        GeneratorProcessorOrModifier::GeneratorProcessor(gproc) => {
                            gen.processors.push((gproc.get_id(), gproc.clone()))
                        }
                        GeneratorProcessorOrModifier::GeneratorModifierFunction((
                            fun,
                            pos,
                            named,
                        )) => fun(gen, pos, named, globals),
                    }
                }
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorList(gl))
        }
        Some(l) => {
            tail.push(l);
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(
                collect_compose(tail),
            ))
        }
        _ => return None,
    })
}
