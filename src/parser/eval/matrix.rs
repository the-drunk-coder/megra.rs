use std::sync;

use crate::builtin_types::*;

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

pub fn mat(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
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
            EvaluatedExpr::Typed(t) => {
                row.push(Box::new(t));
            }
            _ => {}
        }
    }

    if !row.is_empty() {
        pmat.push(row.clone());
    }

    Some(EvaluatedExpr::Typed(TypedEntity::Matrix(pmat)))
}
