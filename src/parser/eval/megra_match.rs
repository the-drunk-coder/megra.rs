use anyhow::{anyhow, Result};

use crate::builtin_types::*;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

use std::sync;

pub fn megra_match(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    // ignore function name
    tail_drain.next();

    if let Some(temp) = tail_drain.next() {
        let mut exprs = Vec::new();

        while let Some(n) = tail_drain.next() {
            if let Some(x) = tail_drain.next() {
                exprs.push((n, x));
            }
        }

        if let EvaluatedExpr::Typed(t1) = temp {
            for (comp, expr) in exprs {
                match comp {
                    EvaluatedExpr::Typed(TypedEntity::Comparable(t2)) => {
                        if let TypedEntity::Comparable(ref ct1) = t1 {
                            if *ct1 == t2 {
                                return Ok(expr);
                            }
                        }
                    }
                    EvaluatedExpr::Comparator(t2) => match t2 {
                        Comparator::GreaterEqual(x) => {
                            if let TypedEntity::Comparable(ref ct1) = t1 {
                                if *ct1 >= x {
                                    return Ok(expr);
                                }
                            }
                        }
                        Comparator::Greater(x) => {
                            if let TypedEntity::Comparable(ref ct1) = t1 {
                                if *ct1 > x {
                                    return Ok(expr);
                                }
                            }
                        }
                        Comparator::Equal(x) => {
                            if let TypedEntity::Comparable(ref ct1) = t1 {
                                if *ct1 == x {
                                    return Ok(expr);
                                }
                            }
                        }
                        Comparator::LesserEqual(x) => {
                            if let TypedEntity::Comparable(ref ct1) = t1 {
                                if *ct1 <= x {
                                    return Ok(expr);
                                }
                            }
                        }
                        Comparator::Lesser(x) => {
                            if let TypedEntity::Comparable(ref ct1) = t1 {
                                if *ct1 < x {
                                    return Ok(expr);
                                }
                            }
                        }
                        Comparator::IsNumerical => {
                            if matches!(t1, TypedEntity::Comparable(Comparable::Double(_)))
                                || matches!(t1, TypedEntity::Comparable(Comparable::Float(_)))
                                || matches!(t1, TypedEntity::Comparable(Comparable::Int32(_)))
                                || matches!(t1, TypedEntity::Comparable(Comparable::Int64(_)))
                            {
                                return Ok(expr);
                            }
                        }

                        Comparator::IsString => {
                            if matches!(t1, TypedEntity::Comparable(Comparable::String(_))) {
                                return Ok(expr);
                            }
                        }
                        Comparator::IsSymbol => {
                            if matches!(t1, TypedEntity::Comparable(Comparable::Symbol(_))) {
                                return Ok(expr);
                            }
                        }
                        Comparator::IsMap => {
                            if matches!(t1, TypedEntity::Map(_)) {
                                return Ok(expr);
                            }
                        }
                        Comparator::IsVec => {
                            if matches!(t1, TypedEntity::Vec(_)) {
                                return Ok(expr);
                            }
                        }
                    },
                    _ => {}
                }
            }
        };

        // nothing matches
        Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
            Comparable::Boolean(false),
        )))
    } else {
        Err(anyhow!("match - body empty"))
    }
}
