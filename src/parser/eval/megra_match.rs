use crate::builtin_types::*;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use std::sync;

pub fn megra_match(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);
    // ignore function name
    tail_drain.next();

    if let Some(to_be_matched) = tail_drain.next() {
        let mut matchees = Vec::new();

        while let Some(n) = tail_drain.next() {
            if let Some(x) = tail_drain.next() {
                matchees.push((n, x));
            }
        }
        Some(EvaluatedExpr::Match(Box::new(to_be_matched), matchees))
    } else {
        None
    }
}
