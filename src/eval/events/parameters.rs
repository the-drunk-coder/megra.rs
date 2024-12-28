use crate::builtin_types::{Comparable, TypedEntity};
use crate::event::{Event, EventOperation};
use crate::music_theory;
use crate::parameter::{DynVal, ParameterValue};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::sample_set::SampleLookup;
use crate::{GlobalVariables, OutputMode, SampleAndWavematrixSet};

use anyhow::{anyhow, bail, Result};
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::HashSet;
use std::sync;

pub fn sample_keys(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);

    // get function name, check which parameter we're dealing with
    let op = if let Some(EvaluatedExpr::Identifier(f)) = tail_drain.next() {
        let parts: Vec<&str> = f.split('-').collect();
        if parts.len() == 1 || parts.len() == 2 {
            // operatron
            if parts.len() == 2 {
                match parts[1] {
                    "add" => EventOperation::Add,
                    "sub" => EventOperation::Subtract,
                    _ => EventOperation::Replace,
                }
            } else {
                EventOperation::Replace
            }
        } else {
            EventOperation::Replace
        }
    } else {
        bail!("sample keys - invalid identifier")
    };

    let mut keyword_set = HashSet::new();

    for p in tail_drain {
        if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) = p {
            keyword_set.insert(s);
        }
    }

    let mut ev = Event::with_name_and_operation("keys".to_string(), op);

    // an "empty" lookup to be merged later down the line ...
    ev.sample_lookup = Some(SampleLookup::Key("".to_string(), keyword_set));

    Ok(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
}

pub fn sample_number(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);

    // get function name, check which parameter we're dealing with
    let op = if let Some(EvaluatedExpr::Identifier(f)) = tail_drain.next() {
        let parts: Vec<&str> = f.split('-').collect();
        if parts.len() == 1 || parts.len() == 2 {
            // operatron
            if parts.len() == 2 {
                match parts[1] {
                    "add" => EventOperation::Add,
                    "sub" => EventOperation::Subtract,
                    "div" => EventOperation::Divide,
                    "mul" => EventOperation::Multiply,
                    _ => EventOperation::Replace,
                }
            } else {
                EventOperation::Replace
            }
        } else {
            EventOperation::Replace
        }
    } else {
        bail!("sample number - invalid identifier")
    };

    let snum = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
        tail_drain.next()
    {
        f as usize
    } else {
        bail!("sample number - invalid number")
    };

    let mut ev = Event::with_name_and_operation("snum".to_string(), op);

    // an "empty" lookup to be merged later down the line ...
    ev.sample_lookup = Some(SampleLookup::N("".to_string(), snum));

    Ok(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
}

pub fn random_sample(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut ev = Event::with_name_and_operation("randsam".to_string(), EventOperation::Replace);

    // an "empty" lookup to be merged later down the line ...
    ev.sample_lookup = Some(SampleLookup::Random("".to_string()));

    Ok(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
}

#[allow(clippy::excessive_precision)]
pub fn transpose(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(n)))) =
        tail_drain.next()
    {
        let mut ev =
            Event::with_name_and_operation("ratefreq".to_string(), EventOperation::Multiply);

        let factor = if n < 0.0 {
            1.0 / f32::powf(1.05946309436, -n) // twelfth root of two, no complex scales so far ...
        } else if n > 0.0 {
            f32::powf(1.05946309436, n)
        } else {
            1.0
        };

        ev.params.insert(
            SynthParameterLabel::PlaybackRate.into(),
            ParameterValue::Scalar(DynVal::with_value(factor)),
        );

        ev.params.insert(
            SynthParameterLabel::PitchFrequency.into(),
            ParameterValue::Scalar(DynVal::with_value(factor)),
        );
        Ok(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
    } else {
        Err(anyhow!("can't transpose this, only numeric arguments"))
    }
}

pub fn parameter(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);

    // get function name, check which parameter we're dealing with
    if let Some(EvaluatedExpr::Identifier(f)) = tail_drain.next() {
        let parts: Vec<&str> = f.split('-').collect();
        if parts.len() == 1 || parts.len() == 2 {
            // operatron
            let op = if parts.len() == 2 {
                match parts[1] {
                    "add" => EventOperation::Add,
                    "sub" => EventOperation::Subtract,
                    "mul" => EventOperation::Multiply,
                    "div" => EventOperation::Divide,
                    _ => EventOperation::Replace,
                }
            } else {
                EventOperation::Replace
            };

            let param_key = crate::event_helpers::map_parameter(parts[0]);

            if let Some(p) = tail_drain.next() {
                let mut ev = Event::with_name_and_operation(parts[0].to_string(), op);
                let par = match p {
                    EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(n))) => {
                        Some(ParameterValue::Scalar(DynVal::with_value(n)))
                    }
                    EvaluatedExpr::Typed(TypedEntity::Parameter(pl)) => {
                        Some(ParameterValue::Scalar(pl))
                    }
                    EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => Some(m),
                    EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))
                        if param_key.label == SynthParameterLabel::PitchFrequency
                            || param_key.label == SynthParameterLabel::LowpassCutoffFrequency
                            || param_key.label == SynthParameterLabel::HighpassCutoffFrequency
                            || param_key.label == SynthParameterLabel::PeakFrequency =>
                    {
                        music_theory::from_string(&s)
                            .map(|note| {
                                ParameterValue::Scalar(DynVal::with_value(music_theory::to_freq(
                                    note,
                                    music_theory::Tuning::EqualTemperament,
                                )))
                            })
                            .ok()
                    }
                    EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))
                        if param_key.label == SynthParameterLabel::NoteArticulation =>
                    {
                        Some(ParameterValue::Symbolic(s))
                    }
                    EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) => {
                        // jump out if the user entered garbage ...
                        crate::eval::events::sound::map_symbolic_param_value(&s)
                    }
                    _ => Some(ParameterValue::Scalar(DynVal::with_value(0.5))), // should be save ...
                };

                // see if we have an explicit index for a parameter address
                let idx = if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
                    if k == "idx" {
                        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                            Comparable::Float(i),
                        ))) = tail_drain.next()
                        {
                            Some(i.floor() as usize)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(p) = par {
                    if let Some(i) = idx {
                        // set the index ...
                        ev.params.insert(param_key.label.with_index(i), p);
                    } else {
                        ev.params.insert(param_key, p);
                    }
                }

                //println!("{:?}", ev);
                Ok(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
            } else {
                Err(anyhow!("invalid parameter function"))
            }
        } else {
            Err(anyhow!("invalid parameter function"))
        }
    } else {
        Err(anyhow!("invalid parameter function"))
    }
}
