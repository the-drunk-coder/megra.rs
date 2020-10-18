use std::collections::HashMap;
use std::boxed::Box;

use crate::parameter::Parameter;

pub struct Event {
    pub name: String,
    pub params: HashMap<String, Box<Parameter>>,
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
}

macro_rules! sine {
    (freq: f32) => {
	let mut ev = Event::with_name("sine");
	ev.tags.push("sine".to_string());
	ev.params.insert("freq", Parameter::with_value(freq));
	ev.params.insert("lvl", Parameter::with_value(1.0));
	ev.params.insert("atk", Parameter::with_value(0.01));
	ev.params.insert("sus", Parameter::with_value(0.1));
	ev.params.insert("rel", Parameter::with_value(0.01));
	ev
    }
}


