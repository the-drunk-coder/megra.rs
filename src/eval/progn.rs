use anyhow::Result;

use crate::builtin_types::*;
use crate::eval::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

use std::sync;

pub fn progn(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    // ignore function name
    tail_drain.next();

    let exprs: Vec<EvaluatedExpr> = tail_drain.collect();

    Ok(EvaluatedExpr::Progn(exprs))
}
