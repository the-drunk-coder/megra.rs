use crate::event::{Event, StaticEvent};
use vom_rs::pfa;
use std::collections::HashMap;

pub struct MarkovSequenceGenerator {
    pub name: String,
    pub generator: pfa::Pfa<char>,
    pub event_mapping: HashMap<char, Vec<Event>>,
    pub duration_mapping: HashMap<(char, char), u64>,
    pub modified: bool,    
    pub symbol_ages: HashMap<char, u64>,
    pub default_duration: u64,
    pub last_transition: Option<pfa::PfaQueryResult<char>>,    
}

impl MarkovSequenceGenerator {
    
    pub fn current_events(&mut self) -> Vec<StaticEvent> {
	let mut static_events = Vec::new();

	self.last_transition = self.generator.next_transition();

	if let Some(trans) = &self.last_transition {
	    // increment symbol age ...
	    *self.symbol_ages.entry(trans.next_symbol).or_insert(0) += 1;

	    // get static events ...
	    if let Some(events) = self.event_mapping.get_mut(&trans.next_symbol) {
		for e in events.iter_mut() {
		    static_events.push(e.to_static());
		}		
	    }	    	 
	}
	static_events
    }
    pub fn current_transition(&mut self) -> StaticEvent {
	Event::with_name("transition".to_string()).to_static()
	// needs duration
    }
}
