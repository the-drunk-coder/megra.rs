use crate::generator_processor::*;
use crate::parser::EvaluatedExpr;

pub fn collect_mapper(tail: &mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send + Sync> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // skip function name

    let proc = if let Some(EvaluatedExpr::Identifier(fun)) = tail_drain.next() {
        MapperProcessor { fun }
    } else {
        MapperProcessor {
            fun: "hi".to_string(),
        }
    };

    Box::new(proc)
}
