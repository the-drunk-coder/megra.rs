use crate::{event::Event,
	    event::StaticEvent,	    
	    markov_sequence_generator::MarkovSequenceGenerator};

use std::collections::HashMap;

pub trait GeneratorProcessor {    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>);
    
    fn process_generator(&mut self, generator: &mut MarkovSequenceGenerator);
    
    fn process_transition(&mut self, transition: &mut StaticEvent);
}

/// Apple-ys events to the throughcoming ones 
pub struct PearProcessor {
    pub events_to_be_applied: HashMap<Vec<String>, Vec<Event>>,    
    pub last_static: HashMap<Vec<String>, Vec<StaticEvent>>
}

impl PearProcessor {
    pub fn new() -> Self {
	PearProcessor {
	    events_to_be_applied: HashMap::new(),
	    last_static: HashMap::new()
	}	    
    }
}

// zip mode etc seem to be outdated ... going for any mode for now
impl GeneratorProcessor for PearProcessor {    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>) {
	self.last_static.clear();
	for (filter, evs) in self.events_to_be_applied.iter_mut() {
	    let mut evs_static = Vec::new();
	    for ev in evs.iter_mut() {
		let ev_static = ev.to_static();	    
		for in_ev in events.iter_mut() {
		    in_ev.apply(&ev_static, filter);
		}
		evs_static.push(ev_static);
	    }
	    
	    self.last_static.insert(filter.to_vec(), evs_static);
	}
    }

    fn process_generator(&mut self, _: &mut MarkovSequenceGenerator) {
	// pass
    }
    
    fn process_transition(&mut self, trans: &mut StaticEvent) {
	for (filter, evs) in self.last_static.iter_mut() {
	    for ev in evs.iter() {
		trans.apply(&ev, filter); // not sure 
	    }
	}	
    }
}

