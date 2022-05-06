use parking_lot::Mutex;
use std::sync;

use crate::builtin_types::*;
use crate::parameter::*;

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn vec(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);

    let mut pvec = Vec::new();

    for p in tail_drain {
        match p {
            EvaluatedExpr::Float(f) => {
                pvec.push(Parameter::with_value(f));
            }
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => {
                pvec.push(p);
            }
            _ => {}
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Vector(
        ParameterValue::Vector(pvec),
    )))
}

pub fn mat(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
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
            EvaluatedExpr::Float(f) => {
                row.push(Parameter::with_value(f));
            }
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => {
                row.push(p);
            }
            _ => {}
        }
    }

    if !row.is_empty() {
        pmat.push(row.clone());
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Matrix(
        ParameterValue::Matrix(pmat),
    )))
}
