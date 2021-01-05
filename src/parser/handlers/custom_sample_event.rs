use crate::builtin_types::*;
use crate::event::Event;
use crate::parameter::Parameter;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;

// this works a bit different than the others in that it returns an option, not the plain
// atom ...
pub fn handle(tail: &mut Vec<Expr>, set: String, sample_set: &SampleSet) -> Option<Expr> {
    if let Some(sample_info) = sample_set.get(&set) {

	let mut tail_drain = tail.drain(..);
	let keywords = &sample_info[0].0;
	let bufnum = sample_info[0].1;
	
	let mut ev = Event::with_name("sampler".to_string());
	ev.tags.insert(set);
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
	
	Some(Expr::Constant(Atom::SoundEvent(ev)))
    } else {
	None
    }            
}
