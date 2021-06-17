use parking_lot::Mutex;
use std::sync;

use crate::builtin_types::*;
use crate::event::Event;
use crate::parameter::Parameter;
use crate::parser::parser_helpers::*;
use crate::sample_set::SampleSet;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::HashSet;

// this works a bit different than the others in that it returns an option, not the plain
// atom ...
pub fn handle(
    tail: &mut Vec<Expr>,
    set: String,
    sample_set_sync: &sync::Arc<Mutex<SampleSet>>,
) -> Option<Expr> {
    let sample_set = sample_set_sync.lock();
    if sample_set.exists_not_empty(&set) {
        let mut drain_idx = 0;
        let sample_info = if tail.is_empty() {
            sample_set.random(&set).unwrap()
        } else {
            match &tail[0] {
                Expr::Constant(Atom::Symbol(s)) => {
                    let mut keyword_set: HashSet<String> = HashSet::new();
                    keyword_set.insert(s.to_string());
                    drain_idx += 1;
                    for t in tail.iter().skip(1) {
                        match t {
                            Expr::Constant(Atom::Symbol(s)) => {
                                keyword_set.insert(s.to_string());
                                drain_idx += 1;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    sample_set.keys(&set, &keyword_set).unwrap() // fallback
                }
                Expr::Constant(Atom::Float(f)) => {
                    drain_idx += 1;
                    sample_set.pos(&set, *f as usize).unwrap()
                }
                _ => {
                    sample_set.random(&set).unwrap() // fallback
                }
            }
        };

        let mut ev = Event::with_name("sampler".to_string());
        ev.tags.insert(set);
        for k in sample_info.key.iter() {
            ev.tags.insert(k.to_string());
        }

        ev.params.insert(
            SynthParameter::SampleBufferNumber,
            Box::new(Parameter::with_value(sample_info.bufnum as f32)),
        );

        // set some defaults
        ev.params
            .insert(SynthParameter::Level, Box::new(Parameter::with_value(0.4)));
        ev.params
            .insert(SynthParameter::Attack, Box::new(Parameter::with_value(1.0)));
        ev.params.insert(
            SynthParameter::Sustain,
            Box::new(Parameter::with_value((sample_info.duration - 2) as f32)),
        );
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

        let mut tail_drain = tail.drain(drain_idx..);
        get_keyword_params(&mut ev.params, &mut tail_drain);

        Some(Expr::Constant(Atom::SoundEvent(ev)))
    } else if set == "feedr" {
        // this is quick and dirty ...
        let mut ev = Event::with_name("livesampler".to_string());

        ev.tags.insert(set);

        ev.params.insert(
            SynthParameter::SampleBufferNumber,
            Box::new(Parameter::with_value(0.0)),
        );

        // set some defaults
        ev.params
            .insert(SynthParameter::Level, Box::new(Parameter::with_value(0.4)));
        ev.params
            .insert(SynthParameter::Attack, Box::new(Parameter::with_value(1.0)));
        ev.params.insert(
            SynthParameter::Sustain,
            Box::new(Parameter::with_value(500 as f32)),
        );
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

        let mut tail_drain = tail.drain(..);
        get_keyword_params(&mut ev.params, &mut tail_drain);

        Some(Expr::Constant(Atom::SoundEvent(ev)))
    } else if set == "freezr" {
        // this is quick and dirty ...
        // read from freeze buffers ...
        let mut ev = Event::with_name("sampler".to_string());

        ev.tags.insert(set);

        // set some defaults
        ev.params
            .insert(SynthParameter::Level, Box::new(Parameter::with_value(0.4)));
        ev.params
            .insert(SynthParameter::Attack, Box::new(Parameter::with_value(1.0)));
        ev.params.insert(
            SynthParameter::Sustain,
            Box::new(Parameter::with_value(500 as f32)),
        );
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

        let mut tail_drain = tail.drain(..);

        // get freeze buffer
        let freeze_buffer = if let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
            // just to be sure
            if f > 10.0 {
                10.0
            } else if f < 1.0 {
                1.0
            } else {
                f
            }
        } else {
            1.0
        };

        ev.params.insert(
            SynthParameter::SampleBufferNumber,
            Box::new(Parameter::with_value(freeze_buffer)),
        );

        get_keyword_params(&mut ev.params, &mut tail_drain);

        Some(Expr::Constant(Atom::SoundEvent(ev)))
    } else {
        None
    }
}
