use crate::builtin_types::*;
use std::sync;

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;

pub fn generator_list(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut gen_list = Vec::new();

    let mut tail_drain = tail.drain(..);
    tail_drain.next();

    for c in tail_drain {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::Generator(g)) => {
                gen_list.push(g);
            }
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(mut gl)) => {
                gen_list.append(&mut gl);
            }
            _ => {
                println!("u can't list this ...");
            }
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(gen_list)))
}
