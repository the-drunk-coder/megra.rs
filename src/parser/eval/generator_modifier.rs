use crate::builtin_types::*;
use crate::generator::*;
use std::collections::HashMap;
use std::sync;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;

/// Helper function to collect arguments
fn get_args(
    tail_drain: &mut std::vec::Drain<EvaluatedExpr>,
) -> (Vec<ConfigParameter>, HashMap<String, ConfigParameter>) {
    let mut pos_args = Vec::new();
    let mut named_args = HashMap::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                pos_args.push(ConfigParameter::Numeric(f))
            }
            EvaluatedExpr::Keyword(k) => {
                named_args.insert(
                    k,
                    match tail_drain.next() {
                        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                            f,
                        )))) => ConfigParameter::Numeric(f),
                        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                            Comparable::Symbol(s),
                        ))) => ConfigParameter::Symbolic(s),
                        _ => ConfigParameter::Numeric(0.0), // dumb placeholder
                    },
                );
            }
            _ => {}
        }
    }
    (pos_args, named_args)
}

pub fn eval_haste(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(haste, tail, globals)
}

pub fn eval_keep(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(keep, tail, globals)
}

pub fn eval_relax(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(relax, tail, globals)
}

pub fn eval_blur(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(blur, tail, globals)
}

pub fn eval_sharpen(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(sharpen, tail, globals)
}

pub fn eval_solidify(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(solidify, tail, globals)
}

pub fn eval_rep(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(rep, tail, globals)
}

pub fn eval_shake(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(shake, tail, globals)
}

pub fn eval_skip(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(skip, tail, globals)
}

pub fn eval_rewind(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(rewind, tail, globals)
}

pub fn eval_rnd(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(rnd, tail, globals)
}

pub fn eval_grow(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(grow, tail, globals)
}

pub fn eval_grown(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(grown, tail, globals)
}

pub fn eval_shrink(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(shrink, tail, globals)
}

pub fn eval_reverse(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(reverse, tail, globals)
}

fn eval_generator_modifier(
    fun: GenModFun,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &std::sync::Arc<GlobalVariables>,
) -> Option<EvaluatedExpr> {
    let last = tail.pop();
    Some(match last {
        Some(EvaluatedExpr::Typed(TypedEntity::Generator(mut g))) => {
            let mut tail_drain = tail.drain(..);
            tail_drain.next();
            let (pos_args, named_args) = get_args(&mut tail_drain);
            // apply to generator
            fun(&mut g, &pos_args, &named_args, globals);
            EvaluatedExpr::Typed(TypedEntity::Generator(g))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(gpom))) => {
            let mut tail_drain = tail.drain(..);
            tail_drain.next(); // ignore function name
            let (pos_args, named_args) = get_args(&mut tail_drain);
            match gpom {
                GeneratorProcessorOrModifier::GeneratorProcessor(_) => {
                    EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(vec![
                        gpom,
                        GeneratorProcessorOrModifier::GeneratorModifierFunction((
                            fun, pos_args, named_args,
                        )),
                    ]))
                }
                GeneratorProcessorOrModifier::GeneratorModifierFunction(_) => {
                    EvaluatedExpr::Typed(TypedEntity::GeneratorModifierList(vec![
                        gpom,
                        GeneratorProcessorOrModifier::GeneratorModifierFunction((
                            fun, pos_args, named_args,
                        )),
                    ]))
                }
            }
        }
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(mut gpoml))) => {
            let mut tail_drain = tail.drain(..);
            tail_drain.next(); // ignore function name
            let (pos_args, named_args) = get_args(&mut tail_drain);
            gpoml.push(GeneratorProcessorOrModifier::GeneratorModifierFunction((
                fun, pos_args, named_args,
            )));
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(gpoml))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorModifierList(mut gpoml))) => {
            let mut tail_drain = tail.drain(..);
            tail_drain.next(); // ignore function name
            let (pos_args, named_args) = get_args(&mut tail_drain);
            gpoml.push(GeneratorProcessorOrModifier::GeneratorModifierFunction((
                fun, pos_args, named_args,
            )));
            EvaluatedExpr::Typed(TypedEntity::GeneratorModifierList(gpoml))
        }
        Some(l) => {
            tail.push(l);
            let mut tail_drain = tail.drain(..);
            tail_drain.next(); // ignore function name
            let (pos_args, named_args) = get_args(&mut tail_drain);
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction((
                    fun, pos_args, named_args,
                )),
            ))
        }
        None => EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(
            GeneratorProcessorOrModifier::GeneratorModifierFunction((
                fun,
                Vec::new(),
                HashMap::new(),
            )),
        )),
    })
}
