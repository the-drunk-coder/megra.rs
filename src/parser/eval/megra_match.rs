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

    if let Some(to_be_matched) = tail_drain.next() {
        let mut matchees = Vec::new();

        while let Some(n) = tail_drain.next() {
            if let Some(x) = tail_drain.next() {
                matchees.push((n, x));
            }
        }
        Ok(EvaluatedExpr::Match(Box::new(to_be_matched), matchees))
    } else {
        Err(anyhow!("match - body empty"))
    }
}
