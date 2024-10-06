use std::sync;

use anyhow::{anyhow, Result};

use crate::{
    builtin_types::{Comparable, GlobalVariables, TypedEntity},
    parser::{EvaluatedExpr, FunctionMap},
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
};

pub fn eval_list_midi_ports(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    Ok(EvaluatedExpr::Command(
        crate::builtin_types::Command::MidiListPorts,
    ))
}

pub fn open_midi_port(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(1..);

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(port)))) =
        tail_drain.next()
    {
        Ok(EvaluatedExpr::Command(
            crate::builtin_types::Command::MidiStartReceiver(port as usize),
        ))
    } else {
        Err(anyhow!("can't open midi port - invalid port"))
    }
}
