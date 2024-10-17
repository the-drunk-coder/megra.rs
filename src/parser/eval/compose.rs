use anyhow::{bail, Result};

use crate::builtin_types::*;
use crate::generator_processor::GeneratorWrapperProcessor;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

use std::sync;

use super::resolver::resolve_globals;

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
    // logical order from bottom to top
    gen_procs.reverse();
    gen_procs
}

pub fn compose(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    let last = tail.pop();
    Ok(match last {
        Some(EvaluatedExpr::Typed(TypedEntity::Generator(mut g))) => {
            let mut proc_or_mods = collect_compose(tail);
            let mut procs = Vec::new();

            for gpom in proc_or_mods.drain(..) {
                match gpom {
                    GeneratorProcessorOrModifier::GeneratorProcessor(gp) => procs.push(gp),
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
                            gen.processors.push(gproc.clone())
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
        _ => bail!("can't compose this ..."),
    })
}
