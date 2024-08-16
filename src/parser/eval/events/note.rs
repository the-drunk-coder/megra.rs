use std::sync;

use ruffbox_synth::building_blocks::SynthParameterLabel;

use crate::{
    event::Event,
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

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
        tail_drain.next()
    {
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
            tail_drain.next()
        {
            let mut ev = Event::with_name("note".to_string());
            ev.params.insert(
                SynthParameterLabel::PitchNote.into(),
                crate::parameter::ParameterValue::Symbolic(s),
            );
            ev.params.insert(
                SynthParameterLabel::Duration.into(),
                crate::parameter::ParameterValue::Symbolic(format!("{f}")),
            );
            ev.tags.insert("note".to_string());
            return Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)));
        }
    }

    None
}
