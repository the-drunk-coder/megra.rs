use crate::event::{Event, EventOperation};
use crate::music_theory;
use crate::parameter::{DynVal, ParameterValue};
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::sample_set::SampleLookup;
use crate::{OutputMode, SampleAndWavematrixSet, VariableStore};
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::HashSet;
use std::sync;

pub fn sample_keys(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
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
        return None;
    };

    let mut keyword_set = HashSet::new();

    for p in tail_drain {
        if let EvaluatedExpr::Symbol(s) = p {
            keyword_set.insert(s);
        }
    }

    let mut ev = Event::with_name_and_operation("keys".to_string(), op);

    // an "empty" lookup to be merged later down the line ...
    ev.sample_lookup = Some(SampleLookup::Key("".to_string(), keyword_set));

    Some(EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(ev)))
}

pub fn sample_number(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
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
        return None;
    };

    let snum = if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
        f as usize
    } else {
        return None;
    };

    let mut ev = Event::with_name_and_operation("snum".to_string(), op);

    // an "empty" lookup to be merged later down the line ...
    ev.sample_lookup = Some(SampleLookup::N("".to_string(), snum));

    Some(EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(ev)))
}

pub fn random_sample(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut ev = Event::with_name_and_operation("randsam".to_string(), EventOperation::Replace);

    // an "empty" lookup to be merged later down the line ...
    ev.sample_lookup = Some(SampleLookup::Random("".to_string()));

    Some(EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(ev)))
}

#[allow(clippy::excessive_precision)]
pub fn transpose(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    if let Some(EvaluatedExpr::Float(n)) = tail_drain.next() {
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
            SynthParameterLabel::PlaybackRate,
            ParameterValue::Scalar(DynVal::with_value(factor)),
        );

        ev.params.insert(
            SynthParameterLabel::PitchFrequency,
            ParameterValue::Scalar(DynVal::with_value(factor)),
        );
        Some(EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(ev)))
    } else {
        None
    }
}

pub fn parameter(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
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
                ev.params.insert(
                    param_key,
                    match p {
                        EvaluatedExpr::Float(n) => ParameterValue::Scalar(DynVal::with_value(n)),
                        EvaluatedExpr::BuiltIn(BuiltIn::Parameter(pl)) => {
                            ParameterValue::Scalar(pl)
                        }
                        EvaluatedExpr::BuiltIn(BuiltIn::Modulator(m)) => m,
                        EvaluatedExpr::Symbol(s)
                            if param_key == SynthParameterLabel::PitchFrequency
                                || param_key == SynthParameterLabel::LowpassCutoffFrequency
                                || param_key == SynthParameterLabel::HighpassCutoffFrequency
                                || param_key == SynthParameterLabel::PeakFrequency =>
                        {
                            ParameterValue::Scalar(DynVal::with_value(music_theory::to_freq(
                                music_theory::from_string(&s),
                                music_theory::Tuning::EqualTemperament,
                            )))
                        }
                        EvaluatedExpr::Symbol(s) => {
                            // jump out if the user entered garbage ...
                            crate::parser::eval::events::sound::map_symbolic_param_value(&s)?
                        }
                        _ => ParameterValue::Scalar(DynVal::with_value(0.5)), // should be save ...
                    },
                );
                //println!("{:?}", ev);
                Some(EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(ev)))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
