use std::collections::HashMap;
use std::boxed::Box;

use crate::parameter::Parameter;

#[derive(Clone,Copy)]
pub enum EventOperation {
    Replace,
    Add,
    Subtract,
    Multiply,
    Divide
}

pub struct Event {
    pub name: String,
    pub params: HashMap<String, Box<Parameter>>,
    pub tags: Vec<String>,
    pub op: EventOperation,
}

// an event is also an operation
pub struct StaticEvent {
    pub name: String,
    pub params: HashMap<String, f32>,
    pub tags: Vec<String>,
    pub op: EventOperation,
}

impl StaticEvent {
    pub fn apply(&mut self, other: &StaticEvent) {
	for (k,v) in other.params.iter() {
	    if self.params.contains_key(k) {
		match other.op {
		    EventOperation::Replace => {
			self.params.insert(k.to_string(), *v);
		    },
		    EventOperation::Add => {
			let new_val = self.params[k] + *v;
			self.params.insert(k.to_string(), new_val);
		    },
		    EventOperation::Subtract => {
			let new_val = self.params[k] - *v;
			self.params.insert(k.to_string(), new_val);
		    },
		    EventOperation::Multiply => {
			let new_val = self.params[k] * *v;
			self.params.insert(k.to_string(), new_val);
		    },
		    EventOperation::Divide => {
			let new_val = self.params[k] / *v;
			self.params.insert(k.to_string(), new_val);
		    },
		}
	    } else {
		self.params.insert(k.to_string(), *v);
	    }	    
	}
    }
}

impl Event {
    pub fn with_name(name: String) -> Self {
	let mut tags = Vec::new();
	tags.push(name.clone()); // add to tags, for subsequent filters ...
	Event {
	    name: name,
	    params: HashMap::new(),
	    tags: tags,
	    op: EventOperation::Replace,
	}
    }

    pub fn evaluate_parameters(&mut self) -> HashMap<String, f32> {
	let mut map = HashMap::new();
	
	for (k,v) in self.params.iter_mut() {
	    map.insert(k.clone(), v.evaluate());
	}
	
	map
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

