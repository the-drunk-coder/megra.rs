mod apple;
mod every;
mod exhibit;
mod inhibit;
mod lifemodel;
mod pear;

use crate::builtin_types::*;
use crate::generator_processor::GeneratorProcessor;
use std::sync;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

use super::resolver::resolve_globals;

type Collector = fn(&mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send + Sync>;

pub fn eval_pear(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    eval_generator_processor(pear::collect_pear, tail)
}

pub fn eval_inhibit(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    eval_generator_processor(inhibit::collect_inhibit, tail)
}

pub fn eval_exhibit(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    eval_generator_processor(exhibit::collect_exhibit, tail)
}

pub fn eval_apple(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    eval_generator_processor(apple::collect_apple, tail)
}

pub fn eval_every(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    eval_generator_processor(every::collect_every, tail)
}

pub fn eval_lifemodel(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    eval_generator_processor(lifemodel::collect_lifemodel, tail)
}

// store list of genProcs in a vec if there's no root gen ???
fn eval_generator_processor(
    collector: Collector,
    tail: &mut Vec<EvaluatedExpr>,
) -> Option<EvaluatedExpr> {
    let last = tail.pop();
    Some(match last {
        Some(EvaluatedExpr::Typed(TypedEntity::Generator(mut g))) => {
            let gp = collector(tail);
            g.processors.push((gp.get_id(), gp));
            EvaluatedExpr::Typed(TypedEntity::Generator(g))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorList(mut gl))) => {
            let gp = collector(tail);
            for gen in gl.iter_mut() {
                gen.processors.push((gp.get_id(), gp.clone()));
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorList(gl))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(gp))) => {
            match gp {
                GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf) => {
                    // if it's a generator modifier function, such as shrink or skip,
                    // push it back as it belongs to the overarching processor
                    // meaning, for example, if we have an "every" processor,
                    // this should be applied by the every processor.
                    // it does lead to some not-so-nice ambiguities but i guess that's
                    // what we have to deal with ... can't be decided really
                    tail.push(EvaluatedExpr::Typed(
                        TypedEntity::GeneratorProcessorOrModifier(
                            GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf),
                        ),
                    ));
                    EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(
                        GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
                    ))
                }
                _ => EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(vec![
                    gp,
                    GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
                ])),
            }
        }
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(mut l))) => {
            l.push(GeneratorProcessorOrModifier::GeneratorProcessor(collector(
                tail,
            )));
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(l))
        }
        // pure modifier lists are handled differently
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorModifierList(ml))) => {
            tail.push(EvaluatedExpr::Typed(TypedEntity::GeneratorModifierList(ml)));
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
            ))
        }
        Some(l) => {
            tail.push(l);
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
            ))
        }
        None => return None,
    })
}
