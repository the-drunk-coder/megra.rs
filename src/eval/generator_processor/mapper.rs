use crate::eval::EvaluatedExpr;
use crate::generator_processor::*;

use super::{Comparable, TypedEntity};

pub fn collect_mapper(tail: &mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send + Sync> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // skip function name

    // so far it's mostly used for side effects
    let mut mapper = MapperProcessor {
        keep: true,
        fun: "hi".to_string(),
    };

    while let Some(arg) = tail_drain.next() {
        match arg {
            EvaluatedExpr::Keyword(k) => {
                if k == "keep" {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        mapper.keep = b;
                    }
                }
            }
            EvaluatedExpr::Identifier(fun) => {
                mapper.fun = fun;
            }
            _ => {}
        }
    }

    Box::new(mapper)
}
