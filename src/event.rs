use std::collections::HashMap;
use std::boxed::Box;

use crate::parameter::Parameter;

pub struct Event {
    pub name: String,
    pub params: HashMap<String, Box<Parameter>>,
    pub tags: Vec<String>,
}

pub struct StaticEvent {
    pub name: String,
    pub params: HashMap<String, f32>,
    pub tags: Vec<String>,
}

impl Event {
    pub fn with_name(name: String) -> Self {
	let mut tags = Vec::new();
	tags.push(name.clone()); // add to tags, for subsequent filters ...
	Event {
	    name: name,
	    params: HashMap::new(),
	    tags: tags
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
	}		    
    }
}

