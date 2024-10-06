use std::sync;

use anyhow::Result;

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
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa >= bb),
            )))
        } else {
            Ok(EvaluatedExpr::Comparator(Comparator::GreaterEqual(aa)))
        }
    } else {
        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
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
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa > bb),
            )))
        } else {
            Ok(EvaluatedExpr::Comparator(Comparator::Greater(aa)))
        }
    } else {
        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
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
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa == bb),
            )))
        } else {
            Ok(EvaluatedExpr::Comparator(Comparator::Equal(aa)))
        }
    } else {
        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
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
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa <= bb),
            )))
        } else {
            Ok(EvaluatedExpr::Comparator(Comparator::LesserEqual(aa)))
        }
    } else {
        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
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
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(aa))) = a {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(bb))) = b {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(aa < bb),
            )))
        } else {
            Ok(EvaluatedExpr::Comparator(Comparator::Lesser(aa)))
        }
    } else {
        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Boolean(false),
        )))
    }
}

pub fn is_type(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // don't need the function name

    let a = tail_drain.next();
    let b = tail_drain.next();

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(type_sym)))) = a {
        if let Some(EvaluatedExpr::Typed(bb)) = b {
            Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
                Comparable::Boolean(match type_sym.as_str() {
                    "num" => {
                        matches!(bb, TypedEntity::Comparable(Comparable::Double(_)))
                            || matches!(bb, TypedEntity::Comparable(Comparable::Float(_)))
                            || matches!(bb, TypedEntity::Comparable(Comparable::Int32(_)))
                            || matches!(bb, TypedEntity::Comparable(Comparable::Int64(_)))
                    }
                    "vec" => matches!(bb, TypedEntity::Vec(_)),
                    "sym" => matches!(bb, TypedEntity::Comparable(Comparable::Symbol(_))),
                    "map" => matches!(bb, TypedEntity::Map(_)),
                    "str" => matches!(bb, TypedEntity::Comparable(Comparable::String(_))),
                    _ => false,
                }),
            )))
        } else {
            Ok(EvaluatedExpr::Comparator(match type_sym.as_str() {
                "num" => Comparator::IsNumerical,
                "str" => Comparator::IsString,
                "vec" => Comparator::IsVec,
                "sym" => Comparator::IsSymbol,
                "map" => Comparator::IsMap,
                _ => Comparator::IsNumerical,
            }))
        }
    } else {
        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Boolean(false),
        )))
    }
}
