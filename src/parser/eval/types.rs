use std::sync;

use crate::{
    builtin_types::{Comparable, GlobalVariables, TypedEntity},
    parser::{EvaluatedExpr, FunctionMap},
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
};

pub fn int(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
        tail_drain.next()
    {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Int32(f as i32),
        )))
    } else {
        None
    }
}

pub fn long(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
        tail_drain.next()
    {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Int64(f as i64),
        )))
    } else {
        None
    }
}

pub fn double(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
        tail_drain.next()
    {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Double(f as f64),
        )))
    } else {
        None
    }
}

pub fn to_string(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => Some(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(format!("{f}")))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Double(f)))) => Some(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(format!("{f}")))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(f)))) => Some(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(format!("{f}")))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Character(f)))) => Some(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(format!("{f}")))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Int32(f)))) => Some(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(format!("{f}")))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Int64(f)))) => Some(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(format!("{f}")))),
        ),
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(f)))) => Some(
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(f))),
        ),
        Some(EvaluatedExpr::Keyword(f)) => Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::String(f),
        ))),
        _ => None,
    }
}

pub fn pair(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    if let Some(EvaluatedExpr::Typed(t1)) = tail_drain.next() {
        if let Some(EvaluatedExpr::Typed(t2)) = tail_drain.next() {
            Some(EvaluatedExpr::Typed(TypedEntity::Pair(
                Box::new(t1),
                Box::new(t2),
            )))
        } else {
            None
        }
    } else {
        None
    }
}
