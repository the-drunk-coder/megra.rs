use crate::builtin_types::*;
use std::sync;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;

pub fn generator_list(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut gen_list = Vec::new();

    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    for c in tail_drain {
        match c {
            EvaluatedExpr::Typed(TypedEntity::Generator(g)) => {
                gen_list.push(g);
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorList(mut gl)) => {
                gen_list.append(&mut gl);
            }
            _ => {
                println!("u can't list this ...");
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::GeneratorList(gen_list)))
}
