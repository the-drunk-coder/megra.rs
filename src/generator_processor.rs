use crate::{event::Event,
	    event::StaticEvent,	    
	    markov_sequence_generator::MarkovSequenceGenerator};


pub trait GeneratorProcessor {    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>);
    
    fn process_generator(&mut self, generator: &mut MarkovSequenceGenerator);
    
    fn process_transition(&mut self, transition: &mut StaticEvent);
}

/// Apple-ys events to the throughcoming ones 
pub struct PearProcessor {
    pub apply_to_transition: bool,
    pub events_to_be_applied: Vec<Event>,
    pub last_static: Vec<StaticEvent>
}

impl PearProcessor {
    pub fn new() -> Self {
	PearProcessor {
	    apply_to_transition: false,
	    events_to_be_applied: Vec::new(),
	    last_static: Vec::new()
	}	    
    }
}

// zip mode etc seem to be outdated ... going for any mode for now
impl GeneratorProcessor for PearProcessor {    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>) {
	self.last_static.clear();
	for ev in self.events_to_be_applied.iter_mut() {
	    let ev_static = ev.to_static();	    
	    for in_ev in events.iter_mut() {
		in_ev.apply(&ev_static);
	    }
	    self.last_static.push(ev_static);
	}
    }

    fn process_generator(&mut self, _: &mut MarkovSequenceGenerator) {
	// pass
    }
    
    fn process_transition(&mut self, trans: &mut StaticEvent) {
	if self.apply_to_transition {
	    for ev in self.last_static.iter() {
		trans.apply(&ev); // not sure 
	    }
	}
    }
}

