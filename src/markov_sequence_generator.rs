use crate::event::{Event, StaticEvent};
use vom_rs::pfa;
use std::collections::HashMap;

pub struct Rule {
    pub source: Vec<char>,
    pub symbol: char,
    pub probability: f32,
    pub duration: u64
}

impl Rule {
    pub fn to_pfa_rule(&self) -> pfa::Rule<char> {
	pfa::Rule {
	    source: self.source.clone(),
	    symbol: self.symbol,
	    probability: self.probability
	}
    }
}

pub struct MarkovSequenceGenerator {
    pub name: String,
    pub generator: pfa::Pfa<char>,
    pub event_mapping: HashMap<char, Vec<Event>>,
    pub duration_mapping: HashMap<(char, char), Event>,
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
	if let Some(trans) = &self.last_transition {
	    if let Some(dur) = self.duration_mapping.get_mut(&(trans.last_symbol, trans.next_symbol)) {
		dur.to_static()		
	    } else {		
		let mut t = Event::with_name("transition".to_string()).to_static();
		t.params.insert("duration".to_string(), self.default_duration as f32);
		t
	    }
	} else {
	    let mut t = Event::with_name("transition".to_string()).to_static();
	    t.params.insert("duration".to_string(), self.default_duration as f32);
	    t
	}	
    }
}
