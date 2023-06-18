use crate::builtin_types::TypedEntity;
use crate::event::{Event, EventOperation};
use crate::event_helpers::map_parameter;
use crate::music_theory;
use crate::parameter::{DynVal, ParameterValue};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::sample_set::SampleLookup;
use crate::{OutputMode, SampleAndWavematrixSet, VariableId, VariableStore};
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::{EnvelopeSegmentType, FilterType, SynthParameterLabel};
use std::collections::HashSet;
use std::sync;

pub fn map_symbolic_param_value(sym: &str) -> Option<ParameterValue> {
    match sym {
        "hpf12" => Some(ParameterValue::FilterType(FilterType::BiquadHpf12dB)),
        "hpf24" => Some(ParameterValue::FilterType(FilterType::BiquadHpf24dB)),
        "lpf12" => Some(ParameterValue::FilterType(FilterType::BiquadLpf12dB)),
        "lpf24" => Some(ParameterValue::FilterType(FilterType::BiquadLpf24dB)),
        "lpf18" => Some(ParameterValue::FilterType(FilterType::Lpf18)),
        "butter2lpf" => Some(ParameterValue::FilterType(FilterType::ButterworthLpf(2))),
        "butter4lpf" => Some(ParameterValue::FilterType(FilterType::ButterworthLpf(4))),
        "butter6lpf" => Some(ParameterValue::FilterType(FilterType::ButterworthLpf(6))),
        "butter8lpf" => Some(ParameterValue::FilterType(FilterType::ButterworthLpf(8))),
        "butter10lpf" => Some(ParameterValue::FilterType(FilterType::ButterworthLpf(10))),
        "butter2hpf" => Some(ParameterValue::FilterType(FilterType::ButterworthHpf(2))),
        "butter4hpf" => Some(ParameterValue::FilterType(FilterType::ButterworthHpf(4))),
        "butter6hpf" => Some(ParameterValue::FilterType(FilterType::ButterworthHpf(6))),
        "butter8hpf" => Some(ParameterValue::FilterType(FilterType::ButterworthHpf(8))),
        "butter10hpf" => Some(ParameterValue::FilterType(FilterType::ButterworthHpf(10))),
        "peak" => Some(ParameterValue::FilterType(FilterType::PeakEQ)),
        "none" => Some(ParameterValue::FilterType(FilterType::Dummy)),
        "lin" => Some(ParameterValue::EnvelopeSegmentType(
            EnvelopeSegmentType::Lin,
        )),
        "sin" => Some(ParameterValue::EnvelopeSegmentType(
            EnvelopeSegmentType::Lin,
        )),
        "cos" => Some(ParameterValue::EnvelopeSegmentType(
            EnvelopeSegmentType::Cos,
        )),
        "log" => Some(ParameterValue::EnvelopeSegmentType(
            EnvelopeSegmentType::Log,
        )),
        "exp" => Some(ParameterValue::EnvelopeSegmentType(
            EnvelopeSegmentType::Exp,
        )),
        "const" => Some(ParameterValue::EnvelopeSegmentType(
            EnvelopeSegmentType::Constant,
        )),
        _ => None,
    }
}

fn collect_param_value(
    tail_drain: &mut std::iter::Peekable<std::vec::Drain<EvaluatedExpr>>,
) -> ParameterValue {
    let mut par_vec = Vec::new();
    while let Some(e) = tail_drain.peek() {
        match e {
            EvaluatedExpr::Typed(TypedEntity::Float(f)) => {
                par_vec.push(DynVal::with_value(*f));
                tail_drain.next();
            }

            EvaluatedExpr::Typed(TypedEntity::Symbol(s)) => {
                if let Some(p) = map_symbolic_param_value(s) {
                    let pc = p;
                    tail_drain.next();
                    return pc;
                } else {
                    break;
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                // this is an annoying clone, really ...
                par_vec.push(p.clone());
                tail_drain.next();
            }
            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => {
                let mc = m.clone();
                tail_drain.next();
                return mc;
            }
            _ => {
                break;
            }
        }
    }
    if par_vec.is_empty() {
        ParameterValue::Scalar(DynVal::with_value(0.0))
    } else if par_vec.len() == 1 {
        ParameterValue::Scalar(par_vec[0].clone())
    } else {
        ParameterValue::Vector(par_vec)
    }
}

fn get_pitch_param(
    ev: &mut Event,
    tail_drain: &mut std::iter::Peekable<std::vec::Drain<EvaluatedExpr>>,
) {
    // get pitch param if possible, or return a default
    // it's not the most elegant solution to find out whether
    // there's a pitch param or not ...
    let mut advance = false;
    ev.params.insert(
        SynthParameterLabel::PitchFrequency,
        match tail_drain.peek() {
            Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(m))) => {
                advance = true;
                m.clone()
            }
            Some(EvaluatedExpr::Typed(TypedEntity::Float(n))) => {
                advance = true;
                ParameterValue::Scalar(DynVal::with_value(*n))
            }
            Some(EvaluatedExpr::Typed(TypedEntity::Parameter(pl))) => {
                advance = true;
                ParameterValue::Scalar(pl.clone())
            }
            Some(EvaluatedExpr::Typed(TypedEntity::Symbol(s))) => {
                advance = true;
                ParameterValue::Scalar(DynVal::with_value(music_theory::to_freq(
                    music_theory::from_string(s),
                    music_theory::Tuning::EqualTemperament,
                )))
            }
            Some(EvaluatedExpr::Identifier(i)) => {
                advance = true;
                ParameterValue::Placeholder(VariableId::Custom(i.to_string()))
            }
            _ => ParameterValue::Scalar(DynVal::with_value(100.0)),
        },
    );
    if advance {
        tail_drain.next();
    }
}

// optional bufnum param
fn get_bufnum_param(
    ev: &mut Event,
    tail_drain: &mut std::iter::Peekable<std::vec::Drain<EvaluatedExpr>>,
) {
    ev.params.insert(
        SynthParameterLabel::SampleBufferNumber,
        ParameterValue::Scalar(match tail_drain.peek() {
            Some(EvaluatedExpr::Typed(TypedEntity::Float(n))) => {
                let nn = *n;
                tail_drain.next();
                if nn as usize > 0 {
                    DynVal::with_value(nn - 1.0)
                } else {
                    DynVal::with_value(0.0)
                }
            }
            Some(EvaluatedExpr::Typed(TypedEntity::Parameter(pl))) => {
                let p = pl.clone();
                tail_drain.next();
                p
            }
            _ => DynVal::with_value(0.0),
        }),
    );
}

fn synth_defaults(ev: &mut Event) {
    // set some defaults 2
    ev.params.insert(
        SynthParameterLabel::EnvelopeLevel,
        ParameterValue::Scalar(DynVal::with_value(0.5)),
    );
    // no distortion per default ...
    ev.params.insert(
        SynthParameterLabel::WaveshaperMix,
        ParameterValue::Scalar(DynVal::with_value(0.0)),
    );
    ev.params.insert(
        SynthParameterLabel::OscillatorAmplitude,
        ParameterValue::Scalar(DynVal::with_value(0.6)),
    );
    ev.params.insert(
        SynthParameterLabel::Attack,
        ParameterValue::Scalar(DynVal::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::Sustain,
        ParameterValue::Scalar(DynVal::with_value(48.0)),
    );
    ev.params.insert(
        SynthParameterLabel::Release,
        ParameterValue::Scalar(DynVal::with_value(100.0)),
    );
    ev.params.insert(
        SynthParameterLabel::ChannelPosition,
        ParameterValue::Scalar(DynVal::with_value(0.00)),
    );
}

fn sample_defaults(ev: &mut Event) {
    // set some defaults
    ev.params.insert(
        SynthParameterLabel::EnvelopeLevel,
        ParameterValue::Scalar(DynVal::with_value(0.5)),
    );
    // no distortion per default ...
    ev.params.insert(
        SynthParameterLabel::WaveshaperMix,
        ParameterValue::Scalar(DynVal::with_value(0.0)),
    );
    ev.params.insert(
        SynthParameterLabel::OscillatorAmplitude,
        ParameterValue::Scalar(DynVal::with_value(0.77)),
    );
    ev.params.insert(
        SynthParameterLabel::Attack,
        ParameterValue::Scalar(DynVal::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::Release,
        ParameterValue::Scalar(DynVal::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::ChannelPosition,
        ParameterValue::Scalar(DynVal::with_value(0.00)),
    );
    ev.params.insert(
        SynthParameterLabel::PlaybackRate,
        ParameterValue::Scalar(DynVal::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::LowpassFilterDistortion,
        ParameterValue::Scalar(DynVal::with_value(0.0)),
    );
    ev.params.insert(
        SynthParameterLabel::PlaybackStart,
        ParameterValue::Scalar(DynVal::with_value(0.0)),
    );
}

fn nofilter_defaults(ev: &mut Event) {
    // set some defaults
    ev.params.insert(
        SynthParameterLabel::EnvelopeLevel,
        ParameterValue::Scalar(DynVal::with_value(0.5)),
    );
    ev.params.insert(
        SynthParameterLabel::LowpassFilterType,
        ParameterValue::FilterType(FilterType::Dummy),
    );
    ev.params.insert(
        SynthParameterLabel::HighpassFilterType,
        ParameterValue::FilterType(FilterType::Dummy),
    );
    ev.params.insert(
        SynthParameterLabel::OscillatorAmplitude,
        ParameterValue::Scalar(DynVal::with_value(0.77)),
    );
    ev.params.insert(
        SynthParameterLabel::Attack,
        ParameterValue::Scalar(DynVal::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::Sustain,
        ParameterValue::Scalar(DynVal::with_value(48.0)),
    );
    ev.params.insert(
        SynthParameterLabel::Release,
        ParameterValue::Scalar(DynVal::with_value(100.0)),
    );
    ev.params.insert(
        SynthParameterLabel::ChannelPosition,
        ParameterValue::Scalar(DynVal::with_value(0.00)),
    );
    ev.params.insert(
        SynthParameterLabel::PlaybackRate,
        ParameterValue::Scalar(DynVal::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::LowpassFilterDistortion,
        ParameterValue::Scalar(DynVal::with_value(0.0)),
    );
    ev.params.insert(
        SynthParameterLabel::PlaybackStart,
        ParameterValue::Scalar(DynVal::with_value(0.0)),
    );
}

pub fn sound(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<VariableStore>,
    sample_set_sync: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).peekable();

    // get the function name ...
    let fname = if let Some(EvaluatedExpr::Identifier(f)) = tail_drain.next() {
        f
    } else {
        // nothing to do ...
        return None;
    };

    // here's where the sound events are taken apart ...
    // the string matching makes this a bit inflexible,
    // so it'd be nice to find a better solution in the future ...
    let mut ev = match fname.as_str() {
        "sine" => {
            let mut ev =
                Event::with_name_and_operation("sine".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            nofilter_defaults(&mut ev);
            ev
        }
        "tri" => {
            let mut ev = Event::with_name_and_operation("tri".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "saw" => {
            let mut ev = Event::with_name_and_operation("saw".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "fmsaw" => {
            let mut ev =
                Event::with_name_and_operation("fmsaw".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "fmsqr" => {
            let mut ev =
                Event::with_name_and_operation("fmsqr".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "fmtri" => {
            let mut ev =
                Event::with_name_and_operation("fmtri".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "wsaw" => {
            let mut ev =
                Event::with_name_and_operation("wsaw".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "sqr" => {
            let mut ev = Event::with_name_and_operation("sqr".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "cub" => {
            let mut ev = Event::with_name_and_operation("cub".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            nofilter_defaults(&mut ev);
            ev
        }
        "risset" => {
            let mut ev =
                Event::with_name_and_operation("risset".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "wtab" => {
            let mut ev =
                Event::with_name_and_operation("wavetable".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "wmat" => {
            let mut ev =
                Event::with_name_and_operation("wavematrix".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "white" => {
            let mut ev =
                Event::with_name_and_operation("white".to_string(), EventOperation::Replace);
            synth_defaults(&mut ev);
            ev
        }
        "brown" => {
            let mut ev =
                Event::with_name_and_operation("brown".to_string(), EventOperation::Replace);
            synth_defaults(&mut ev);
            ev
        }
        "silence" => Event::with_name_and_operation("silence".to_string(), EventOperation::Replace),
        "~" => Event::with_name_and_operation("silence".to_string(), EventOperation::Replace),
        "feedr" => {
            let mut ev = Event::with_name("livesampler".to_string());
            ev.tags.insert(fname);

            get_bufnum_param(&mut ev, &mut tail_drain);
            sample_defaults(&mut ev);

            ev // return event
        }
        "freezr" => {
            // this one needs extra treatment because the
            // offsets for the buffer number need to be calculated
            // in the ruffbox backend ...
            let mut ev = Event::with_name("frozensampler".to_string());
            ev.tags.insert(fname);

            get_bufnum_param(&mut ev, &mut tail_drain);
            sample_defaults(&mut ev);

            ev // return event
        }
        _ => {
            // check if it's a sample event
            // to see whether it's a sample event,
            // we check whether the function name represents
            // a sample set ...
            let sample_set = sample_set_sync.lock();
            if sample_set.exists_not_empty(&fname) {
                let mut ev = Event::with_name("sampler".to_string());

                // insert the function name as tag
                ev.tags.insert(fname.clone());

                // set some defaults
                sample_defaults(&mut ev);

                // instead of resolving the sample buffer number directly,
                // we send the info down the line so it can be manipulated
                // along the way and evaluated later on ..
                ev.sample_lookup = match tail_drain.peek() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Symbol(s))) => {
                        let mut keyword_set: HashSet<String> = HashSet::new();
                        keyword_set.insert(s.to_string());
                        ev.tags.insert(s.to_string());
                        while let Some(EvaluatedExpr::Typed(TypedEntity::Symbol(s))) =
                            tail_drain.peek()
                        {
                            keyword_set.insert(s.to_string());
                            tail_drain.next();
                        }
                        Some(SampleLookup::Key(fname.to_string(), keyword_set))
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Float(pos))) => {
                        Some(SampleLookup::N(fname.to_string(), *pos as usize))
                    }
                    _ => {
                        let info = sample_set.random(&fname).unwrap();
                        Some(SampleLookup::FixedRandom(fname.to_string(), info.clone()))
                    }
                };

                ev // return event
            } else {
                return None;
            }
        }
    };

    // collect keyword params
    while let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        if k == "tags" {
            while let Some(EvaluatedExpr::Typed(TypedEntity::Symbol(s))) = tail_drain.peek() {
                ev.tags.insert(s.clone());
                tail_drain.next();
            }
        } else if k == "wm" {
            // wavematrix lookup
            if let Some(EvaluatedExpr::Typed(TypedEntity::Symbol(s))) = tail_drain.peek() {
                if let Some(wavematrix) = sample_set_sync.lock().get_wavematrix(s) {
                    //println!("found wavematrix {}", s);
                    ev.params.insert(
                        map_parameter(&k),
                        ParameterValue::Matrix(wavematrix.clone()),
                    );
                    tail_drain.next();
                } else {
                    println!("couldn't find wavematrix {s}")
                }
            } else {
                ev.params
                    .insert(map_parameter(&k), collect_param_value(&mut tail_drain));
            }
        } else {
            ev.params
                .insert(map_parameter(&k), collect_param_value(&mut tail_drain));
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::SoundEvent(ev)))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parser::*;

    #[test]
    fn test_eval_sound() {
        let snippet = "(risset 4000 :lvl 1.0)";
        let mut functions = FunctionMap::new();
        let sample_set = sync::Arc::new(Mutex::new(SampleAndWavematrixSet::new()));

        functions
            .fmap
            .insert("risset".to_string(), eval::events::sound::sound);

        let globals = sync::Arc::new(VariableStore::new());

        match eval_from_str(
            snippet,
            &functions,
            &globals,
            &sample_set,
            OutputMode::Stereo,
        ) {
            Ok(res) => {
                assert!(matches!(
                    res,
                    EvaluatedExpr::Typed(TypedEntity::SoundEvent(_))
                ));
            }
            Err(e) => {
                println!("err {e}");
                assert!(false)
            }
        }
    }
}
