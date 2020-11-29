use std::boxed::Box;
use crate::{event::StaticEvent,
	    generator_processor::GeneratorProcessor,
	    markov_sequence_generator::MarkovSequenceGenerator};

pub struct Generator {
    pub name: String,
    pub root_generator: MarkovSequenceGenerator,
    pub processors: Vec<Box<dyn GeneratorProcessor + Send>>,
}

impl Generator {

    pub fn current_events(&mut self) -> Vec<StaticEvent> {
	let mut events = self.root_generator.current_events();
	for proc in self.processors.iter_mut() {
	    proc.process_events(&mut events);
	    proc.process_generator(&mut self.root_generator);
	}
	events
    }
    
    pub fn current_transition(&mut self) -> StaticEvent {
	let mut trans = self.root_generator.current_transition();
	for proc in self.processors.iter_mut() {
	    proc.process_transition(&mut trans);
	}
	trans
    }
}
