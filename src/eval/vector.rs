use std::sync;

use anyhow::{anyhow, bail, Result};

use crate::builtin_types::*;

use crate::eval::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

use super::resolver::resolve_globals;

pub fn vec(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    resolve_globals(tail, globals);

    let tail_drain = tail.drain(..).skip(1);

    let mut pvec = Vec::new();

    for p in tail_drain {
        if let EvaluatedExpr::Typed(t) = p {
            pvec.push(Box::new(t));
        }
    }

    Ok(EvaluatedExpr::Typed(TypedEntity::Vec(pvec)))
}

pub fn push(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(1..);

    let place = match tail_drain.next() {
        Some(EvaluatedExpr::Identifier(i)) => VariableId::Custom(i),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            VariableId::Symbol(s)
        }
        _ => {
            bail!("vec - invalid vector identifier")
        }
    };

    if let Some(EvaluatedExpr::Typed(t)) = tail_drain.next() {
        Ok(EvaluatedExpr::Command(Command::Push(place, t)))
    } else {
        Err(anyhow!("vec - can't push to {place:?}"))
    }
}
