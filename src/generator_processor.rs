use rand::*;
use std::collections::HashMap;

use crate::{event::Event,
	    event::StaticEvent,
	    parameter::Parameter,
	    markov_sequence_generator::MarkovSequenceGenerator};

pub trait GeneratorProcessor {    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>);    
    fn process_generator(&mut self, generator: &mut MarkovSequenceGenerator);    
    fn process_transition(&mut self, transition: &mut StaticEvent);
}

/// Apple-ys events to the throughcoming ones 
pub struct PearProcessor {
    pub events_to_be_applied: Vec<(Parameter, HashMap<Vec<String>, Vec<Event>>)>,    
    pub last_static: Vec<(usize, HashMap<Vec<String>, Vec<StaticEvent>>)>,
}

impl PearProcessor {
    pub fn new() -> Self {
	PearProcessor {
	    events_to_be_applied: Vec::new(),
	    last_static: Vec::new()
	}	    
    }
}

// zip mode etc seem to be outdated ... going for any mode for now
impl GeneratorProcessor for PearProcessor {    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>) {
	self.last_static.clear();
	let mut rng = rand::thread_rng();
	for (prob, filtered_events) in self.events_to_be_applied.iter_mut() {
	    let mut stat_evs = HashMap::new();
	    let cur_prob:usize = (prob.evaluate() as usize) % 101; // make sure prob is always between 0 and 100
	    //println!("cur p {}", cur_prob);
	    for (filter, evs) in filtered_events.iter_mut() {
		let mut evs_static = Vec::new();
		for ev in evs.iter_mut() {
		    let ev_static = ev.to_static();	    
		    for in_ev in events.iter_mut() {
			if rng.gen_range(0, 100) < cur_prob {
			    in_ev.apply(&ev_static, filter);
			}			
		    }
		    evs_static.push(ev_static);
		}	    
		stat_evs.insert(filter.to_vec(), evs_static);
	    }
	    self.last_static.push((cur_prob, stat_evs));
	}	    	
    }

    fn process_generator(&mut self, _: &mut MarkovSequenceGenerator) {
	// pass
    }
    
    fn process_transition(&mut self, trans: &mut StaticEvent) {
	let mut rng = rand::thread_rng();
	for (prob, filtered_events) in self.last_static.iter_mut() {
	    for (filter, evs) in filtered_events.iter_mut() {
		for ev in evs.iter() {
		    if (rng.gen_range(0, 100) as usize) < *prob {
			trans.apply(&ev, filter); // not sure
		    }
		}
	    }
	}
    }
}

