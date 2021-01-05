use std::collections::HashSet;
use crate::builtin_types::*;
use crate::sample_set::SampleSet;
use crate::event::Event;
use crate::parameter::Parameter;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;

// this works a bit different than the others in that it returns an option, not the plain
// atom ...
pub fn handle(tail: &mut Vec<Expr>, set: String, sample_set: &SampleSet) -> Option<Expr> {
	    
    if sample_set.exists_not_empty(&set) {

	let mut drain_idx = 0;
	let sample_info = if tail.len() == 0 {
	    sample_set.random(&set).unwrap()
	} else {
	    match &tail[0] {
		Expr::Constant(Atom::Symbol(s)) => {
		    let mut keyword_set:HashSet<String> = HashSet::new();
		    keyword_set.insert(s.to_string());
		    drain_idx += 1;
		    for i in 1..tail.len() {
			match &tail[i] {
			    Expr::Constant(Atom::Symbol(s)) => {
				keyword_set.insert(s.to_string());
			    },
			    _ => {}
			}
			drain_idx += 1;
		    }
		    sample_set.keys(&set, &keyword_set).unwrap() // fallback
		},
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
	
	ev.params.insert(SynthParameter::SampleBufferNumber, Box::new(Parameter::with_value(sample_info.bufnum as f32)));
	
	// set some defaults
	ev.params.insert(SynthParameter::Level, Box::new(Parameter::with_value(0.4)));
	ev.params.insert(SynthParameter::Attack, Box::new(Parameter::with_value(1.0)));
	ev.params.insert(SynthParameter::Sustain, Box::new(Parameter::with_value(200.0)));
	ev.params.insert(SynthParameter::Release, Box::new(Parameter::with_value(1.0)));
	ev.params.insert(SynthParameter::ChannelPosition, Box::new(Parameter::with_value(0.00)));
	ev.params.insert(SynthParameter::PlaybackRate, Box::new(Parameter::with_value(1.0)));
	ev.params.insert(SynthParameter::LowpassFilterDistortion, Box::new(Parameter::with_value(0.0)));
	ev.params.insert(SynthParameter::PlaybackStart, Box::new(Parameter::with_value(0.0)));    

	let mut tail_drain = tail.drain(drain_idx..);
	get_keyword_params(&mut ev.params, &mut tail_drain);
	
	Some(Expr::Constant(Atom::SoundEvent(ev)))
    } else {
	None
    }            
}
