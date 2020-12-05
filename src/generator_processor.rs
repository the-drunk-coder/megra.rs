use rand::*;
use std::collections::HashMap;

use crate::{event::Event,
	    event::StaticEvent,
	    parameter::Parameter,
	    generator::{TimeMod, GenModFun},
	    markov_sequence_generator::MarkovSequenceGenerator};

pub trait GeneratorProcessor {    
    fn process_events(&mut self, events: &mut Vec<StaticEvent>);    
    fn process_generator(&mut self, generator: &mut MarkovSequenceGenerator, time_mods: &mut Vec<TimeMod>);    
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

    fn process_generator(&mut self, _: &mut MarkovSequenceGenerator, _: &mut Vec<TimeMod>) {
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

/// Apple-ys modifiers to the underlying processors
pub struct AppleProcessor {
    pub modifiers_to_be_applied: Vec<(Parameter, Vec<(GenModFun, Vec<f32>)>)>
}

impl AppleProcessor {
    pub fn new() -> Self {
	AppleProcessor {
	    modifiers_to_be_applied: Vec::new(),	    
	}	    
    }
}


impl GeneratorProcessor for AppleProcessor {    
    fn process_events(&mut self, _: &mut Vec<StaticEvent>) {
	// pass
    }

    fn process_generator(&mut self, gen: &mut MarkovSequenceGenerator, time_mods: &mut Vec<TimeMod>) {	
	let mut rng = rand::thread_rng();
	for (prob, gen_mods) in self.modifiers_to_be_applied.iter_mut() {	    
	    let cur_prob:usize = (prob.evaluate() as usize) % 101; // make sure prob is always between 0 and 100
	    for (gen_mod, args) in gen_mods.iter() {						
		if rng.gen_range(0, 100) < cur_prob {
		    gen_mod(gen, time_mods, args)
		}				
	    }	    
	}	    	
    }
    
    fn process_transition(&mut self, _: &mut StaticEvent) {
	// pass
    }
}

type StaticEventsAndFilters = HashMap<Vec<String>, Vec<StaticEvent>>;
type EventsAndFilters = HashMap<Vec<String>, Vec<Event>>;
type GenModFunsAndArgs = Vec<(GenModFun, Vec<f32>)>;

/// Apple-ys events to the throughcoming ones 
pub struct EveryProcessor {
    pub step_count:usize,
    pub max_step_count:usize,
    pub things_to_be_applied: Vec<(Parameter, EventsAndFilters, GenModFunsAndArgs)>,    
    pub last_static: Vec<(usize, StaticEventsAndFilters)>, // only needed for events, not filters
}

impl GeneratorProcessor for EveryProcessor {
    // this one 
    fn process_events(&mut self, events: &mut Vec<StaticEvent>) {
	self.last_static.clear();
	for (step, filtered_events, _) in self.things_to_be_applied.iter_mut() { // genmodfuns not needed here ...
	    let cur_step:usize = (step.evaluate() as usize) % 101; // make sure prob is always between 0 and 100
	    if self.step_count % cur_step == 0 {
		let mut stat_evs = HashMap::new();
		//println!("cur p {}", cur_prob);
		for (filter, evs) in filtered_events.iter_mut() {
		    let mut evs_static = Vec::new();
		    for ev in evs.iter_mut() {
			let ev_static = ev.to_static();	    
			for in_ev in events.iter_mut() {			    
			    in_ev.apply(&ev_static, filter);
			}			
			evs_static.push(ev_static);
		    }		    
		    stat_evs.insert(filter.to_vec(), evs_static);
		}	    		
		self.last_static.push((cur_step, stat_evs));
	    }	    
	}
    }

    fn process_generator(&mut self, gen: &mut MarkovSequenceGenerator, time_mods: &mut Vec<TimeMod>) {	
	for (step, _, gen_mods) in self.things_to_be_applied.iter_mut() { // genmodfuns not needed here ...
	    
	    let cur_step:usize = (step.static_val as usize) % 101; 
	    if self.step_count % cur_step == 0 {		    
		for (gen_mod, args) in gen_mods.iter() {		    
		    gen_mod(gen, time_mods, args)
		}				
	    }	    
	}	    	
    }
    
    fn process_transition(&mut self, trans: &mut StaticEvent) {	
	for (cur_step, filtered_events) in self.last_static.iter() {
	    if self.step_count % cur_step == 0 {
		for (filter, evs) in filtered_events.iter() {
		    for ev in evs.iter() {			
			trans.apply(&ev, filter); // not sure
		    }
		}
	    }
	}
	self.step_count += 1;
    }
}
