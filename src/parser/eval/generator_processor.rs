mod apple;
mod every;
mod exhibit;
mod inhibit;
mod lifemodel;
mod pear;

use crate::builtin_types::*;
use crate::generator_processor::GeneratorProcessor;
use std::sync;

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;

type Collector = fn(&mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send>;

pub fn eval_pear(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_processor(pear::collect_pear, tail)
}

pub fn eval_inhibit(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_processor(inhibit::collect_inhibit, tail)
}

pub fn eval_exhibit(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_processor(exhibit::collect_exhibit, tail)
}

pub fn eval_apple(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_processor(apple::collect_apple, tail)
}

pub fn eval_every(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_processor(every::collect_every, tail)
}

pub fn eval_lifemodel(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_generator_processor(lifemodel::collect_lifemodel, tail)
}

// store list of genProcs in a vec if there's no root gen ???
fn eval_generator_processor(
    collector: Collector,
    tail: &mut Vec<EvaluatedExpr>,
) -> Option<EvaluatedExpr> {
    let last = tail.pop();
    Some(match last {
        Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(mut g))) => {
            g.processors.push(collector(tail));

            EvaluatedExpr::BuiltIn(BuiltIn::Generator(g))
        }
        Some(EvaluatedExpr::Symbol(s)) => {
            // check if previous is a keyword ...
            // if not, assume it's a part proxy
            let prev = tail.pop();
            match prev {
                Some(EvaluatedExpr::Keyword(_)) => {
                    tail.push(prev.unwrap()); // push back for further processing
                    tail.push(EvaluatedExpr::Symbol(s));
                    EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
                        GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
                    ))
                }
                _ => {
                    tail.push(prev.unwrap()); // push back for further processing
                    EvaluatedExpr::BuiltIn(BuiltIn::PartProxy(PartProxy::Proxy(
                        s,
                        vec![GeneratorProcessorOrModifier::GeneratorProcessor(collector(
                            tail,
                        ))],
                    )))
                }
            }
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::PartProxy(PartProxy::Proxy(s, mut proxy_mods)))) => {
            proxy_mods.push(GeneratorProcessorOrModifier::GeneratorProcessor(collector(
                tail,
            )));
            EvaluatedExpr::BuiltIn(BuiltIn::PartProxy(PartProxy::Proxy(s, proxy_mods)))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(mut l))) => {
            let gp = collector(tail);
            let mut pdrain = l.drain(..);
            let mut new_list = Vec::new();
            while let Some(PartProxy::Proxy(s, mut proxy_mods)) = pdrain.next() {
                proxy_mods.push(GeneratorProcessorOrModifier::GeneratorProcessor(gp.clone()));
                new_list.push(PartProxy::Proxy(s, proxy_mods));
            }
            EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(new_list))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(mut gl))) => {
            let gp = collector(tail);
            for gen in gl.iter_mut() {
                gen.processors.push(gp.clone());
            }
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(gl))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(gp))) => {
            match gp {
                GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf) => {
                    // if it's a generator modifier function, such as shrink or skip,
                    // push it back as it belongs to the overarching processor
                    tail.push(EvaluatedExpr::BuiltIn(
                        BuiltIn::GeneratorProcessorOrModifier(
                            GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf),
                        ),
                    ));
                    EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
                        GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
                    ))
                }
                _ => EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(vec![
                    gp,
                    GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
                ])),
            }
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(mut l))) => {
            l.push(GeneratorProcessorOrModifier::GeneratorProcessor(collector(
                tail,
            )));
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(l))
        }
        // pure modifier lists are handled differently
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorModifierList(ml))) => {
            tail.push(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorModifierList(ml)));
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
            ))
        }
        Some(l) => {
            tail.push(l);
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorProcessor(collector(tail)),
            ))
        }
        None => return None,
    })
}
