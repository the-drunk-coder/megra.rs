use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet, VariableStore};

use parking_lot::Mutex;
use std::sync;

use super::resolver::resolve_globals_or_lazy;

pub fn lazy_resolve(
    functions: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    var_store: &sync::Arc<VariableStore>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    output_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    if let Some(EvaluatedExpr::Identifier(f)) = tail.get(0) {
        if functions.std_lib.contains_key(f) {
            return functions.std_lib[f](functions, tail, var_store, sample_set, output_mode);
        }
    } else {
        return None;
    }

    None
}
