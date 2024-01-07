use std::sync;

use crate::{
    builtin_types::{Comparable, GlobalVariables, TypedEntity},
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

    let result = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            aa >= bb
        } else {
            false
        }
    } else {
        false
    };

    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Boolean(result),
    )))
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

    let result = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            aa > bb
        } else {
            false
        }
    } else {
        false
    };

    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Boolean(result),
    )))
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

    let result = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            aa == bb
        } else {
            false
        }
    } else {
        false
    };

    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Boolean(result),
    )))
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

    let result = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            aa <= bb
        } else {
            false
        }
    } else {
        false
    };

    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Boolean(result),
    )))
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

    let result = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            aa < bb
        } else {
            false
        }
    } else {
        false
    };

    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::Boolean(result),
    )))
}
