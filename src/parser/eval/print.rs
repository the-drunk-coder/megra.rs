use crate::builtin_types::*;
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use std::sync;

use super::resolver::resolve_globals;

pub fn print(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    // ignore function name
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    if let Some(EvaluatedExpr::Typed(t)) = tail_drain.next() {
        Some(EvaluatedExpr::Command(Command::Print(t)))
    } else {
        None
    }
}
