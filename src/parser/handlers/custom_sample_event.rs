use std::collections::HashSet;
use crate::builtin_types::*;
use crate::event::Event;
use crate::parameter::Parameter;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;

pub fn handle(tail: &mut Vec<Expr>, bufnum: usize, set: &String, keywords: &HashSet<String>) -> Atom {
    
    let mut tail_drain = tail.drain(..);
    
    let mut ev = Event::with_name("sampler".to_string());
    ev.tags.insert(set.to_string());
    for k in keywords.iter() {
	ev.tags.insert(k.to_string());
    }
    
    ev.params.insert(SynthParameter::SampleBufferNumber, Box::new(Parameter::with_value(bufnum as f32)));
    
    // set some defaults
    ev.params.insert(SynthParameter::Level, Box::new(Parameter::with_value(0.4)));
    ev.params.insert(SynthParameter::Attack, Box::new(Parameter::with_value(1.0)));
    ev.params.insert(SynthParameter::Sustain, Box::new(Parameter::with_value(200.0)));
    ev.params.insert(SynthParameter::Release, Box::new(Parameter::with_value(1.0)));
    ev.params.insert(SynthParameter::ChannelPosition, Box::new(Parameter::with_value(0.00)));
    ev.params.insert(SynthParameter::PlaybackRate, Box::new(Parameter::with_value(1.0)));
    ev.params.insert(SynthParameter::LowpassFilterDistortion, Box::new(Parameter::with_value(0.0)));
    ev.params.insert(SynthParameter::PlaybackStart, Box::new(Parameter::with_value(0.0)));    
    
    get_keyword_params(&mut ev.params, &mut tail_drain);
    
    Atom::SoundEvent (ev)
}
