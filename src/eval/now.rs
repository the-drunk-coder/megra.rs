use std::{sync::Arc, time::SystemTime};

use crate::{
    eval::{EvaluatedExpr, FunctionMap},
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
    Comparable, GlobalVariables, TypedEntity,
};

pub fn now(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr, anyhow::Error> {
    let t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    Ok(EvaluatedExpr::Typed(TypedEntity::Comparable(
        Comparable::UInt128(t),
    )))
}
