use std::sync;

use parking_lot::Mutex;

use crate::{
    builtin_types::{TypedEntity, VariableStore},
    parser::{EvaluatedExpr, FunctionMap},
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
};

pub fn eval_list_midi_ports(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    Some(EvaluatedExpr::Command(
        crate::builtin_types::Command::MidiListPorts,
    ))
}

pub fn open_midi_port(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(1..);

    if let Some(EvaluatedExpr::Typed(TypedEntity::Float(port))) = tail_drain.next() {
        Some(EvaluatedExpr::Command(
            crate::builtin_types::Command::MidiStartReceiver(port as usize),
        ))
    } else {
        None
    }
}
