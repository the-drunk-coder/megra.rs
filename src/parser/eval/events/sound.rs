use crate::event::{Event, EventOperation};
use crate::event_helpers::map_parameter;
use crate::music_theory;
use crate::parameter::{Parameter, ParameterValue};
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{GlobalParameters, OutputMode, SampleSet};
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::HashSet;
use std::sync;

fn collect_param_value(
    tail_drain: &mut std::iter::Peekable<std::vec::Drain<EvaluatedExpr>>,
) -> ParameterValue {
    let mut par_vec = Vec::new();
    while let Some(e) = tail_drain.peek() {
        match e {
            EvaluatedExpr::Float(f) => {
                par_vec.push(Parameter::with_value(*f));
                tail_drain.next();
            }
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => {
                // this is an annoying clone, really ...
                par_vec.push(p.clone());
                tail_drain.next();
            } EvaluatedExpr::BuiltIn(BuiltIn::Modulator(m)) => {
		return m.clone();
	    }		
            _ => {
                break;
            }
        }
    }
    if par_vec.is_empty() {
        ParameterValue::Scalar(Parameter::with_value(0.0))
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
    // first arg is always freq ...
    ev.params.insert(
        SynthParameterLabel::PitchFrequency,
        match tail_drain.next() {
	    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(m))) => m,
            Some(EvaluatedExpr::Float(n)) => ParameterValue::Scalar(Parameter::with_value(n)),
            Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(pl))) => ParameterValue::Scalar(pl),
            Some(EvaluatedExpr::Symbol(s)) => ParameterValue::Scalar(Parameter::with_value(music_theory::to_freq(
                music_theory::from_string(&s),
                music_theory::Tuning::EqualTemperament,
            ))),
            _ => ParameterValue::Scalar(Parameter::with_value(100.0)),
        }
    );
}

// optional bufnum param
fn get_bufnum_param(
    ev: &mut Event,
    tail_drain: &mut std::iter::Peekable<std::vec::Drain<EvaluatedExpr>>,
) {
    ev.params.insert(
        SynthParameterLabel::SampleBufferNumber,
        ParameterValue::Scalar(match tail_drain.peek() {
            Some(EvaluatedExpr::Float(n)) => {
                let nn = *n;
                tail_drain.next();
                if nn as usize > 0 {
                    Parameter::with_value(nn - 1.0)
                } else {
                    Parameter::with_value(0.0)
                }
            }
            Some(EvaluatedExpr::BuiltIn(BuiltIn::Parameter(pl))) => {
                let p = pl.clone();
                tail_drain.next();
                p
            }
            _ => Parameter::with_value(0.0),
        }),
    );
}

fn synth_defaults(ev: &mut Event) {
    // set some defaults 2
    ev.params.insert(
        SynthParameterLabel::Level,
        ParameterValue::Scalar(Parameter::with_value(0.4)),
    );
    ev.params.insert(
        SynthParameterLabel::Attack,
        ParameterValue::Scalar(Parameter::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::Sustain,
        ParameterValue::Scalar(Parameter::with_value(48.0)),
    );
    ev.params.insert(
        SynthParameterLabel::Release,
        ParameterValue::Scalar(Parameter::with_value(100.0)),
    );
    ev.params.insert(
        SynthParameterLabel::ChannelPosition,
        ParameterValue::Scalar(Parameter::with_value(0.00)),
    );
}

fn sample_defaults(ev: &mut Event) {
    // set some defaults
    ev.params.insert(
        SynthParameterLabel::Level,
        ParameterValue::Scalar(Parameter::with_value(0.4)),
    );
    ev.params.insert(
        SynthParameterLabel::Attack,
        ParameterValue::Scalar(Parameter::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::Release,
        ParameterValue::Scalar(Parameter::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::ChannelPosition,
        ParameterValue::Scalar(Parameter::with_value(0.00)),
    );
    ev.params.insert(
        SynthParameterLabel::PlaybackRate,
        ParameterValue::Scalar(Parameter::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameterLabel::LowpassFilterDistortion,
        ParameterValue::Scalar(Parameter::with_value(0.0)),
    );
    ev.params.insert(
        SynthParameterLabel::PlaybackStart,
        ParameterValue::Scalar(Parameter::with_value(0.0)),
    );
}

pub fn sound(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    sample_set_sync: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).peekable();

    // get the function name ...
    let fname = if let Some(EvaluatedExpr::FunctionName(f)) = tail_drain.next() {
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
            synth_defaults(&mut ev);
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
        "sqr" => {
            let mut ev = Event::with_name_and_operation("sqr".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
            ev
        }
        "cub" => {
            let mut ev = Event::with_name_and_operation("cub".to_string(), EventOperation::Replace);
            get_pitch_param(&mut ev, &mut tail_drain);
            synth_defaults(&mut ev);
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
            let sample_set = sample_set_sync.lock();
            if sample_set.exists_not_empty(&fname) {
                let mut keyword_set: HashSet<String> = HashSet::new();

                let sample_info = match tail_drain.peek() {
                    Some(EvaluatedExpr::Symbol(s)) => {
                        keyword_set.insert(s.to_string());
                        while let Some(EvaluatedExpr::Symbol(s)) = tail_drain.peek() {
                            keyword_set.insert(s.to_string());
                            tail_drain.next();
                        }
                        sample_set.keys(&fname, &keyword_set).unwrap() // fallback
                    }
                    Some(EvaluatedExpr::Float(pos)) => {
                        sample_set.pos(&fname, *pos as usize).unwrap()
                    }
                    _ => {
                        sample_set.random(&fname).unwrap() // fallback
                    }
                };

                let mut ev = Event::with_name("sampler".to_string());
                ev.tags.insert(fname);
                if !keyword_set.is_empty() {
                    for kw in keyword_set.drain() {
                        ev.tags.insert(kw);
                    }
                }
                for k in sample_info.key.iter() {
                    ev.tags.insert(k.to_string());
                }

                ev.params.insert(
                    SynthParameterLabel::SampleBufferNumber,
                    ParameterValue::Scalar(Parameter::with_value(sample_info.bufnum as f32)),
                );
                ev.params.insert(
                    SynthParameterLabel::Sustain,
                    ParameterValue::Scalar(Parameter::with_value(
                        (sample_info.duration - 2) as f32,
                    )),
                );
                sample_defaults(&mut ev);

                ev // return event
            } else {
                return None;
            }
        }
    };

    // collect keyword params
    while let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        if k == "tags" {
            while let Some(EvaluatedExpr::Symbol(s)) = tail_drain.peek() {
                ev.tags.insert(s.clone());
                tail_drain.next();
            }
        } else {
            ev.params
                .insert(map_parameter(&k), collect_param_value(&mut tail_drain));
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(ev)))
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
        let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));

        functions
            .fmap
            .insert("risset".to_string(), eval::events::sound::sound);

        let globals = sync::Arc::new(GlobalParameters::new());

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
                    EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(_))
                ));
            }
            Err(e) => {
                println!("err {}", e);
                assert!(false)
            }
        }
    }
}
