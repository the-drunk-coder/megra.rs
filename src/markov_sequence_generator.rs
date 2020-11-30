use crate::event::{Event, StaticEvent};
use vom_rs::pfa;
use std::collections::HashMap;
use ruffbox_synth::ruffbox::synth::SynthParameter;

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
    pub init_symbol: char,
    pub last_transition: Option<pfa::PfaQueryResult<char>>,    
}

impl MarkovSequenceGenerator {
    
    pub fn current_events(&mut self) -> Vec<StaticEvent> {
	let mut static_events = Vec::new();
	
	// try to get a transition if there wasn't one
	// that'd mean it's probably the initial one, or there's something wrong ... 
	if self.last_transition.is_none() {
	    self.last_transition = self.generator.next_transition();
	}
	
	if let Some(trans) = &self.last_transition {
	    // increment symbol age ...
	    *self.symbol_ages.entry(trans.last_symbol).or_insert(0) += 1;
	    // get static events ...
	    if let Some(events) = self.event_mapping.get_mut(&trans.last_symbol) {
		for e in events.iter_mut() {
		    static_events.push(e.to_static());
		}		
	    }	    	 
	} else {
	    // increment symbol age ...
	    *self.symbol_ages.entry(self.init_symbol).or_insert(0) += 1;
	    // get static events ...
	    if let Some(events) = self.event_mapping.get_mut(&self.init_symbol) {
		for e in events.iter_mut() {
		    static_events.push(e.to_static());
		}		
	    }	
	}
	    		
	static_events
    }

    pub fn current_transition(&mut self) -> StaticEvent {
	let mut transition = None;
	
	if let Some(trans) = &self.last_transition {
	    if let Some(dur) = self.duration_mapping.get_mut(&(trans.last_symbol, trans.next_symbol)) {
		transition = Some(dur.to_static());
	    } else {		
		let mut t = Event::with_name("transition".to_string()).to_static();
		t.params.insert(SynthParameter::Duration, self.default_duration as f32);
		transition = Some(t);
	    }
	}

	// advance pfa ...
	self.last_transition = self.generator.next_transition();
	
	if let Some(t) = transition {
	    t
	} else {
	    let mut t = Event::with_name("transition".to_string()).to_static();
	    t.params.insert(SynthParameter::Duration, self.default_duration as f32);
	    t
	}	
    }
}
