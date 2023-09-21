use parking_lot::Mutex;
use std::sync;

use crate::builtin_types::*;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn vec(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);

    let mut pvec = Vec::new();

    for p in tail_drain {
        if let EvaluatedExpr::Typed(t) = p {
            pvec.push(Box::new(t));
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::Vec(pvec)))
}

pub fn push(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(1..);

    let place = match tail_drain.next() {
        Some(EvaluatedExpr::Identifier(i)) => VariableId::Custom(i),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            VariableId::Symbol(s)
        }
        _ => {
            return None;
        }
    };

    if let Some(EvaluatedExpr::Typed(t)) = tail_drain.next() {
        Some(EvaluatedExpr::Command(Command::Push(place, t)))
    } else {
        None
    }
}
