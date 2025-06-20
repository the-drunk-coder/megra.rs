use std::sync;

use anyhow::{bail, Result};

use ruffbox_synth::building_blocks::SynthParameterLabel;

use crate::{
    eval::resolver::resolve_globals,
    eval::{EvaluatedExpr, FunctionMap},
    event::Event,
    parameter::{DynVal, NoteParameterLabel},
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
                NoteParameterLabel::Pitch.into(),
                crate::parameter::ParameterValue::Symbolic(s),
            );
        }
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
            ev.params.insert(
                NoteParameterLabel::Pitch.into(),
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

    let mut art = crate::parameter::ParameterValue::Symbolic("".to_string());
    let mut syllable = crate::parameter::ParameterValue::Symbolic("none".to_string());

    while let Some(arg) = tail_drain.next() {
        match arg {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "art" | "articulation" => match tail_drain.next() {
                    Some(n) => match n {
                        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s))) => {
                            art = crate::parameter::ParameterValue::Symbolic(s);
                        }
                        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                            art = crate::parameter::ParameterValue::Symbolic(s);
                        }
                        _ => {
                            bail!("note - invalid arg type for keyword {k}")
                        }
                    },
                    None => {
                        bail!("note - arg for keyword {k} missing!")
                    }
                },
                "syl" | "syllable" => match tail_drain.next() {
                    Some(n) => match n {
                        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s))) => {
                            syllable = crate::parameter::ParameterValue::Symbolic(s);
                        }
                        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                            syllable = crate::parameter::ParameterValue::Symbolic(s);
                        }
                        _ => {
                            bail!("note - invalid arg type for keyword {k}")
                        }
                    },
                    None => {
                        bail!("note - arg for keyword {k} missing!")
                    }
                },
                _ => {
                    bail!("note - invalid keyword {k}")
                }
            },
            _ => {
                bail!("note - found something that's not a keyword argument")
            }
        }
    }

    ev.params
        .insert(NoteParameterLabel::Articulation.into(), art);

    ev.params
        .insert(NoteParameterLabel::Syllable.into(), syllable);

    Ok(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
}
