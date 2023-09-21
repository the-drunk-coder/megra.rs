use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync;

use crate::builtin_types::*;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn map(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);

    let mut pmap = HashMap::new();

    for p in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Pair(key, val)) = p {
            match *key {
                TypedEntity::Comparable(Comparable::String(k)) => {
                    pmap.insert(VariableId::Custom(k), *val);
                }
                TypedEntity::Comparable(Comparable::Symbol(k)) => {
                    pmap.insert(VariableId::Symbol(k), *val);
                }
                _ => { /* ignore */ }
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::Map(pmap)))
}

pub fn insert(
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

    let key = match tail_drain.next() {
        //Some(EvaluatedExpr::Identifier(i)) => VariableId::Custom(i),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) => {
            VariableId::Custom(s)
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            VariableId::Symbol(s)
        }
        _ => {
            return None;
        }
    };

    if let Some(EvaluatedExpr::Typed(t)) = tail_drain.next() {
        Some(EvaluatedExpr::Command(Command::Insert(place, key, t)))
    } else {
        None
    }
}
