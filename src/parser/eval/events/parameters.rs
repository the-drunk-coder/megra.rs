use crate::event::{Event, EventOperation};
use crate::music_theory;
use crate::parameter::{DynVal, ParameterValue};
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{GlobalParameters, OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::sync;

pub fn parameter(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);

    // get function name, check which parameter we're dealing with
    if let Some(EvaluatedExpr::FunctionName(f)) = tail_drain.next() {
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
