use crate::builtin_types::*;
use crate::generator::*;
use std::collections::HashMap;
use std::sync;

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;

/// Helper function to collect arguments
fn get_args(
    tail_drain: &mut std::vec::Drain<EvaluatedExpr>,
) -> (Vec<ConfigParameter>, HashMap<String, ConfigParameter>) {
    let mut pos_args = Vec::new();
    let mut named_args = HashMap::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Float(f) => pos_args.push(ConfigParameter::Numeric(f)),
            EvaluatedExpr::Keyword(k) => {
                named_args.insert(
                    k,
                    match tail_drain.next() {
                        Some(EvaluatedExpr::Float(f)) => ConfigParameter::Numeric(f),
                        Some(EvaluatedExpr::Symbol(s)) => ConfigParameter::Symbolic(s),
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
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(haste, tail)
}

pub fn eval_keep(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(keep, tail)
}

pub fn eval_relax(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(relax, tail)
}

pub fn eval_blur(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(blur, tail)
}

pub fn eval_sharpen(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(sharpen, tail)
}

pub fn eval_solidify(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(solidify, tail)
}

pub fn eval_rep(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(rep, tail)
}

pub fn eval_shake(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(shake, tail)
}

pub fn eval_skip(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(skip, tail)
}

pub fn eval_rewind(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(rewind, tail)
}

pub fn eval_rnd(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(rnd, tail)
}

pub fn eval_grow(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(grow, tail)
}

pub fn eval_shrink(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(shrink, tail)
}

pub fn eval_reverse(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_modifier(reverse, tail)
}

fn eval_generator_modifier(fun: GenModFun, tail: &mut Vec<EvaluatedExpr>) -> Option<EvaluatedExpr> {
    let last = tail.pop();
    Some(match last {
        Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(mut g))) => {
            let mut tail_drain = tail.drain(..);
            tail_drain.next();
            let (pos_args, named_args) = get_args(&mut tail_drain);
            // apply to generator
            fun(
                &mut g.root_generator,
                &mut g.time_mods,
                &mut g.keep_root,
                &pos_args,
                &named_args,
            );
            EvaluatedExpr::BuiltIn(BuiltIn::Generator(g))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(gpom))) => {
            let mut tail_drain = tail.drain(..);
            tail_drain.next(); // ignore function name
            let (pos_args, named_args) = get_args(&mut tail_drain);
            match gpom {
                GeneratorProcessorOrModifier::GeneratorProcessor(_) => {
                    EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(vec![
                        gpom,
                        GeneratorProcessorOrModifier::GeneratorModifierFunction((
                            fun, pos_args, named_args,
                        )),
                    ]))
                }
                GeneratorProcessorOrModifier::GeneratorModifierFunction(_) => {
                    EvaluatedExpr::BuiltIn(BuiltIn::GeneratorModifierList(vec![
                        gpom,
                        GeneratorProcessorOrModifier::GeneratorModifierFunction((
                            fun, pos_args, named_args,
                        )),
                    ]))
                }
            }
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(mut gpoml))) => {
            let mut tail_drain = tail.drain(..);
            tail_drain.next(); // ignore function name
            let (pos_args, named_args) = get_args(&mut tail_drain);
            gpoml.push(GeneratorProcessorOrModifier::GeneratorModifierFunction((
                fun, pos_args, named_args,
            )));
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(gpoml))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorModifierList(mut gpoml))) => {
            let mut tail_drain = tail.drain(..);
            tail_drain.next(); // ignore function name
            let (pos_args, named_args) = get_args(&mut tail_drain);
            gpoml.push(GeneratorProcessorOrModifier::GeneratorModifierFunction((
                fun, pos_args, named_args,
            )));
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorModifierList(gpoml))
        }
        Some(l) => {
            tail.push(l);
            let mut tail_drain = tail.drain(..);
            tail_drain.next(); // ignore function name
            let (pos_args, named_args) = get_args(&mut tail_drain);
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction((
                    fun, pos_args, named_args,
                )),
            ))
        }
        None => EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
            GeneratorProcessorOrModifier::GeneratorModifierFunction((
                fun,
                Vec::new(),
                HashMap::new(),
            )),
        )),
    })
}
