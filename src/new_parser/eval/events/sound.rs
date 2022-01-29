use crate::event::{Event, EventOperation};
use crate::event_helpers::map_parameter;
use crate::music_theory;
use crate::new_parser::{BuiltIn2, EvaluatedExpr};
use crate::parameter::Parameter;
use crate::{GlobalParameters, OutputMode, SampleSet};
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::HashSet;
use std::sync;

fn get_pitch_param(
    ev: &mut Event,
    tail_drain: &mut std::iter::Peekable<std::vec::Drain<EvaluatedExpr>>,
) {
    // first arg is always freq ...
    ev.params.insert(
        SynthParameter::PitchFrequency,
        Box::new(match tail_drain.next() {
            Some(EvaluatedExpr::Float(n)) => Parameter::with_value(n),
            //Some(Expr::Constant(Atom::Parameter(pl))) => pl,
            Some(EvaluatedExpr::Symbol(s)) => Parameter::with_value(music_theory::to_freq(
                music_theory::from_string(&s),
                music_theory::Tuning::EqualTemperament,
            )),
            _ => Parameter::with_value(100.0),
        }),
    );
}

fn synth_defaults(ev: &mut Event) {
    // set some defaults 2
    ev.params
        .insert(SynthParameter::Level, Box::new(Parameter::with_value(0.4)));
    ev.params
        .insert(SynthParameter::Attack, Box::new(Parameter::with_value(1.0)));
    ev.params.insert(
        SynthParameter::Sustain,
        Box::new(Parameter::with_value(48.0)),
    );
    ev.params.insert(
        SynthParameter::Release,
        Box::new(Parameter::with_value(100.0)),
    );
    ev.params.insert(
        SynthParameter::ChannelPosition,
        Box::new(Parameter::with_value(0.00)),
    );
}

fn sample_defaults(ev: &mut Event) {
    // set some defaults
    ev.params
        .insert(SynthParameter::Level, Box::new(Parameter::with_value(0.4)));
    ev.params
        .insert(SynthParameter::Attack, Box::new(Parameter::with_value(1.0)));
    ev.params.insert(
        SynthParameter::Release,
        Box::new(Parameter::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameter::ChannelPosition,
        Box::new(Parameter::with_value(0.00)),
    );
    ev.params.insert(
        SynthParameter::PlaybackRate,
        Box::new(Parameter::with_value(1.0)),
    );
    ev.params.insert(
        SynthParameter::LowpassFilterDistortion,
        Box::new(Parameter::with_value(0.0)),
    );
    ev.params.insert(
        SynthParameter::PlaybackStart,
        Box::new(Parameter::with_value(0.0)),
    );
}

pub fn sound(
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    sample_set_sync: &sync::Arc<Mutex<SampleSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).peekable();

    let fname = if let Some(EvaluatedExpr::FunctionName(f)) = tail_drain.next() {
        f
    } else {
        // nothing to do ...
        return None;
    };

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
                    SynthParameter::SampleBufferNumber,
                    Box::new(Parameter::with_value(sample_info.bufnum as f32)),
                );
                ev.params.insert(
                    SynthParameter::Sustain,
                    Box::new(Parameter::with_value((sample_info.duration - 2) as f32)),
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
            while let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
                ev.tags.insert(s);
            }
        } else {
            ev.params.insert(
                map_parameter(&k),
                Box::new(match tail_drain.next() {
                    Some(EvaluatedExpr::Float(n)) => Parameter::with_value(n),
                    Some(EvaluatedExpr::BuiltIn(BuiltIn2::Parameter(pl))) => pl,
                    _ => Parameter::with_value(0.0),
                }),
            );
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn2::SoundEvent(ev)))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::new_parser::*;

    #[test]
    fn test_eval_sound() {
        let snippet = "(risset 4000 :lvl 1.0)";
        let mut functions = FunctionMap::new();
        let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));

        functions.insert("risset".to_string(), eval::events::sound::sound);

        let globals = sync::Arc::new(GlobalParameters::new());

        match eval_from_str2(snippet, &functions, &globals, &sample_set) {
            Ok(res) => {
                assert!(matches!(
                    res,
                    EvaluatedExpr::BuiltIn(BuiltIn2::SoundEvent(_))
                ));
            }
            Err(e) => {
                println!("err {}", e);
                assert!(false)
            }
        }
    }
}
