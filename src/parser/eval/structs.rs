use parking_lot::Mutex;
use std::sync;

use crate::builtin_types::*;
use crate::parameter::*;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn vec(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);

    let mut pvec = Vec::new();

    for p in tail_drain {
        match p {
            EvaluatedExpr::Typed(TypedEntity::Float(f)) => {
                pvec.push(DynVal::with_value(f));
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                pvec.push(p);
            }
            _ => {}
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::Vector(pvec),
    )))
}

pub fn mat(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);

    let mut pmat = Vec::new();
    let mut row = Vec::new();

    for p in tail_drain {
        match p {
            EvaluatedExpr::Keyword(k) => {
                if k == "r" {
                    // collect row
                    if !row.is_empty() {
                        pmat.push(row.clone());
                        row = Vec::new();
                    }
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Float(f)) => {
                row.push(DynVal::with_value(f));
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                row.push(p);
            }
            _ => {}
        }
    }

    if !row.is_empty() {
        pmat.push(row.clone());
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::Matrix(pmat),
    )))
}
