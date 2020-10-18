use std::boxed::Box;
use crate::{event::Event,
	    event_processor::EventProcessor,
	    markov_sequence_generator::MarkovSequenceGenerator};

pub struct Generator {
    pub name: String,
    pub root_generator: MarkovSequenceGenerator,
    processors: Vec<Box<dyn EventProcessor + Send>>,
}

impl Generator {

    pub fn current_events(&mut self) -> Vec<Event> {
	Vec::new()
    }

    pub fn current_transition(&mut self) -> Event {
	Event::with_name("transition".to_string())
	// needs duration
    }
}
    
    
