use crate::{event::Event,
	    event::StaticEvent,	    
	    markov_sequence_generator::MarkovSequenceGenerator};


pub trait GeneratorProcessor {
    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>);
    
    fn process_generator(&mut self, generator: &mut MarkovSequenceGenerator);
    
    fn process_transition(&mut self, transition: &mut StaticEvent);
}

/// Apple-ys events to the throughcoming ones 
struct PearProcessor {
    events_to_be_applied: Vec<Event>
}

// zip mode etc seem to be outdated ... going for any mode for now
impl GeneratorProcessor for PearProcessor {    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>) {
	for ev in self.events_to_be_applied.iter_mut() {
	    let ev_static = ev.to_static();
	    for in_ev in events.iter_mut() {
		in_ev.apply(&ev_static);
	    }	    
	}
    }

    fn process_generator(&mut self, _: &mut MarkovSequenceGenerator) {
	// pass
    }
    
    fn process_transition(&mut self, _: &mut StaticEvent) {
	// pass
    }
}

