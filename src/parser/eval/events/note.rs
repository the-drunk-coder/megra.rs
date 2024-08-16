use std::sync;

use ruffbox_synth::building_blocks::SynthParameterLabel;

use crate::{
    event::Event,
    parameter::DynVal,
    parser::{eval::resolver::resolve_globals, EvaluatedExpr, FunctionMap},
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
    Comparable, GlobalVariables, TypedEntity,
};

pub fn event_note(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    let mut ev = Event::with_name("note".to_string());

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) => {
            ev.params.insert(
                SynthParameterLabel::PitchNote.into(),
                crate::parameter::ParameterValue::Symbolic(s),
            );
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
            ev.params.insert(
                SynthParameterLabel::PitchNote.into(),
                crate::parameter::ParameterValue::Scalar(DynVal::with_value(f)),
            );
        }
        _ => {
            return None;
        }
    }

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) => {
            ev.params.insert(
                SynthParameterLabel::Duration.into(),
                crate::parameter::ParameterValue::Symbolic(s),
            );
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
            ev.params.insert(
                SynthParameterLabel::Duration.into(),
                crate::parameter::ParameterValue::Scalar(DynVal::with_value(f)),
            );
        }
        _ => {
            return None;
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
}
