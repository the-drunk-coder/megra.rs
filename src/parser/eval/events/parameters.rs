use crate::event::{Event, EventOperation};
use crate::music_theory;
use crate::parameter::Parameter;
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{GlobalParameters, OutputMode, SampleSet};
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::sync;

pub fn parameter(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
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

            let param_key = match parts[0] {
                "freq " => SynthParameter::PitchFrequency,
                "pitch " => SynthParameter::PitchFrequency,
                "atk" => SynthParameter::Attack,
                "rel" => SynthParameter::Release,
                "sus" => SynthParameter::Sustain,
                "pos" => SynthParameter::ChannelPosition,
                "lvl" => SynthParameter::Level,
                "dur" => SynthParameter::Duration,
                "rev" => SynthParameter::ReverbMix,
                "del" => SynthParameter::DelayMix,
                "lpf" => SynthParameter::LowpassCutoffFrequency,
                "lpq" => SynthParameter::LowpassQFactor,
                "lpd" => SynthParameter::LowpassFilterDistortion,
                "hpf" => SynthParameter::HighpassCutoffFrequency,
                "hpq" => SynthParameter::HighpassQFactor,
                "pff" => SynthParameter::PeakFrequency,
                "pfq" => SynthParameter::PeakQFactor,
                "pfg" => SynthParameter::PeakGain,
                "pw" => SynthParameter::Pulsewidth,
                "start" => SynthParameter::PlaybackStart,
                "rate" => SynthParameter::PlaybackRate,
                _ => SynthParameter::PitchFrequency,
            };

            if let Some(p) = tail_drain.next() {
                let mut ev = Event::with_name_and_operation(parts[0].to_string(), op);
                ev.params.insert(
                    param_key,
                    Box::new(match p {
                        EvaluatedExpr::Float(n) => Parameter::with_value(n),
                        EvaluatedExpr::BuiltIn(BuiltIn::Parameter(pl)) => pl,
                        EvaluatedExpr::Symbol(s)
                            if param_key == SynthParameter::PitchFrequency
                                || param_key == SynthParameter::LowpassCutoffFrequency
                                || param_key == SynthParameter::HighpassCutoffFrequency
                                || param_key == SynthParameter::PeakFrequency =>
                        {
                            Parameter::with_value(music_theory::to_freq(
                                music_theory::from_string(&s),
                                music_theory::Tuning::EqualTemperament,
                            ))
                        }
                        _ => Parameter::with_value(0.5), // should be save ...
                    }),
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
