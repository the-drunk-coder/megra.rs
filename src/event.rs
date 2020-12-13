use std::collections::{HashMap, HashSet};
use std::boxed::Box;
use ruffbox_synth::ruffbox::synth::SynthParameter;

use crate::parameter::Parameter;
use crate::session::SyncContext;

#[derive(Clone,Copy)]
pub enum EventOperation {
    Replace,
    Add,
    Subtract,
    Multiply,
    Divide
}

#[derive(Clone)]
pub struct Event {
    pub name: String,
    pub params: HashMap<SynthParameter, Box<Parameter>>,
    pub tags: HashSet<String>,
    pub op: EventOperation,
}

// an event is also an operation
#[derive(Clone)]
pub struct StaticEvent {
    pub name: String,
    pub params: HashMap<SynthParameter, f32>,
    pub tags: HashSet<String>,
    pub op: EventOperation,
}

#[derive(Clone)]
pub struct ControlEvent {
    pub tags: HashSet<String>,
    pub ctx: Option<Vec<SyncContext>>,
    // later: command
}

#[derive(Clone)]
pub enum SourceEvent {
    Sound(Event),
    Control(ControlEvent)
}

#[derive(Clone)]
pub enum InterpretableEvent {
    Sound(StaticEvent),
    Control(ControlEvent)
}

impl StaticEvent {
    pub fn apply(&mut self, other: &StaticEvent, filters: &Vec<String>) {

	let mut apply = false;

	// check if tags contain one of the filters (filters are always or-matched)
	for f in filters.into_iter() {	    
	    if f == "" || self.tags.contains(f) {
		apply = true;
	    }
	}

	if !apply {
	    return;
	}
	
	for (k,v) in other.params.iter() {
	    if self.params.contains_key(k) {
		match other.op {
		    EventOperation::Replace => {
			self.params.insert(*k, *v);
		    },
		    EventOperation::Add => {
			let new_val = self.params[k] + *v;
			self.params.insert(*k, new_val);
		    },
		    EventOperation::Subtract => {
			let new_val = self.params[k] - *v;
			self.params.insert(*k, new_val);
		    },
		    EventOperation::Multiply => {
			let new_val = self.params[k] * *v;
			self.params.insert(*k, new_val);
		    },
		    EventOperation::Divide => {
			let new_val = self.params[k] / *v;
			self.params.insert(*k, new_val);
		    },
		}
	    } else {
		self.params.insert(*k, *v);
	    }	    
	}
    }
}

impl Event {
    pub fn with_name_and_operation(name: String, op: EventOperation) -> Self {
	let mut tags = HashSet::new();
	tags.insert(name.clone()); // add to tags, for subsequent filters ...
	Event {
	    name: name,
	    params: HashMap::new(),
	    tags: tags,
	    op: op,
	}
    }

    pub fn with_name(name: String) -> Self {
	let mut tags = HashSet::new();
	tags.insert(name.clone()); // add to tags, for subsequent filters ...
	Event {
	    name: name,
	    params: HashMap::new(),
	    tags: tags,
	    op: EventOperation::Replace,
	}
    }

    pub fn evaluate_parameters(&mut self) -> HashMap<SynthParameter, f32> {
	let mut map = HashMap::new();
	
	for (k,v) in self.params.iter_mut() {
	    map.insert(*k, v.evaluate());
	}
	
	map
    }

    pub fn shake(&mut self, factor: f32) {
	for (_,v) in self.params.iter_mut() {
	    v.shake(factor);
	}
    }
    
    pub fn to_static(&mut self) -> StaticEvent {
	StaticEvent {
	    name: self.name.clone(),
	    params: self.evaluate_parameters(),
	    tags: self.tags.clone(),
	    op: self.op.clone(),
	}		    
    }
}

