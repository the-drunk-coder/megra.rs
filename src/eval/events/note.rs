use std::sync;

use anyhow::{bail, Result};
use ruffbox_synth::building_blocks::SynthParameterLabel;

use crate::{
    eval::resolver::resolve_globals,
    event::Event,
    parameter::DynVal,
    parser::{EvaluatedExpr, FunctionMap},
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
) -> Result<EvaluatedExpr> {
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
            bail!("note - first arg bust be string or number")
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
            bail!("note - second arg bust be string or number")
        }
    }

    // third argument is optional
    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
        tail_drain.next()
    {
        ev.params.insert(
            SynthParameterLabel::NoteArticulation.into(),
            crate::parameter::ParameterValue::Symbolic(s),
        );
    } else {
        ev.params.insert(
            SynthParameterLabel::NoteArticulation.into(),
            crate::parameter::ParameterValue::Symbolic("none".to_string()),
        );
    }

    Ok(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
}
