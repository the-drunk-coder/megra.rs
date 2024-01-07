use std::sync;

use crate::{
    builtin_types::{Comparable, Comparator, GlobalVariables, TypedEntity},
    parser::{EvaluatedExpr, FunctionMap},
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
};

pub fn greater_equal(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa >= bb),
            )))
        } else {
            Some(EvaluatedExpr::Comparator(Comparator::GreaterEqual(aa)))
        }
    } else {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Boolean(false),
        )))
    }
}

pub fn greater(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa > bb),
            )))
        } else {
            Some(EvaluatedExpr::Comparator(Comparator::Greater(aa)))
        }
    } else {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Boolean(false),
        )))
    }
}

pub fn equal(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa == bb),
            )))
        } else {
            Some(EvaluatedExpr::Comparator(Comparator::Equal(aa)))
        }
    } else {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Boolean(false),
        )))
    }
}

pub fn lesser_equal(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa <= bb),
            )))
        } else {
            Some(EvaluatedExpr::Comparator(Comparator::LesserEqual(aa)))
        }
    } else {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Boolean(false),
        )))
    }
}

pub fn lesser(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa < bb),
            )))
        } else {
            Some(EvaluatedExpr::Comparator(Comparator::Lesser(aa)))
        }
    } else {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Boolean(false),
        )))
    }
}
