use crate::builtin_types::*;
use crate::generator_processor::GeneratorWrapperProcessor;

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use std::sync;

fn collect_compose(tail: &mut Vec<EvaluatedExpr>) -> Vec<GeneratorProcessorOrModifier> {
    let mut gen_procs = Vec::new();
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // skip function name

    for c in tail_drain {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(gp)) => {
                gen_procs.push(gp);
            }
            EvaluatedExpr::BuiltIn(BuiltIn::Generator(g)) => {
                gen_procs.push(GeneratorProcessorOrModifier::GeneratorProcessor(Box::new(
                    GeneratorWrapperProcessor::with_generator(g),
                )));
            }
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(mut gpl)) => {
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
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let last = tail.pop();
    Some(match last {
        Some(EvaluatedExpr::Symbol(s)) => EvaluatedExpr::BuiltIn(BuiltIn::PartProxy(
            PartProxy::Proxy(s, collect_compose(tail)),
        )),
        Some(EvaluatedExpr::BuiltIn(BuiltIn::PartProxy(PartProxy::Proxy(s, mut proxy_mods)))) => {
            proxy_mods.append(&mut collect_compose(tail));
            EvaluatedExpr::BuiltIn(BuiltIn::PartProxy(PartProxy::Proxy(s, proxy_mods)))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(mut l))) => {
            let gp = collect_compose(tail);
            let mut pdrain = l.drain(..);
            let mut new_list = Vec::new();
            while let Some(PartProxy::Proxy(s, mut proxy_mods)) = pdrain.next() {
                proxy_mods.append(&mut gp.clone());
                new_list.push(PartProxy::Proxy(s, proxy_mods));
            }
            EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(new_list))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(mut g))) => {
            let mut proc_or_mods = collect_compose(tail);
            let mut procs = Vec::new();

            for gpom in proc_or_mods.drain(..) {
                match gpom {
                    GeneratorProcessorOrModifier::GeneratorProcessor(gp) => {
                        procs.push((gp.get_id(), gp))
                    }
                    GeneratorProcessorOrModifier::GeneratorModifierFunction((fun, pos, named)) => {
                        fun(&mut g, &pos, &named)
                    }
                }
            }

            g.processors.append(&mut procs);
            EvaluatedExpr::BuiltIn(BuiltIn::Generator(g))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(mut gl))) => {
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
                        )) => fun(gen, pos, named),
                    }
                }
            }
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(gl))
        }
        Some(l) => {
            tail.push(l);
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(collect_compose(
                tail,
            )))
        }
        _ => return None,
    })
}
