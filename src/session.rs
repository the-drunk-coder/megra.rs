use crate::scheduler::{Scheduler, SchedulerData};
use crate::generator::Generator;
use crate::event_helpers::*;
use std::collections::HashMap;
use std::sync;
use ruffbox_synth::ruffbox::Ruffbox;
use parking_lot::Mutex;

pub struct Session {
    schedulers: HashMap<String, Scheduler>,    
}

impl Session {

    pub fn new() -> Self {
	Session {
	    schedulers: HashMap::new(),	    
	}
    }
    
    pub fn start_generator(&mut self, gen: Box<Generator>, ruffbox: sync::Arc<Mutex<Ruffbox<512>>>) {

	// start scheduler if it exists ...
	if let Some(sched) = self.schedulers.get_mut(&gen.name) {
	    sched.stop();
	}

	// replace old scheduler (this will be where state handover needs to happen)
	self.schedulers.insert(gen.name.clone(), Scheduler::new());	

	// the evaluation function ...
	let eval_loop = |data: &mut SchedulerData| -> f64 {
	    
	    let events = data.generator.current_events();
	    let mut ruff = data.ruffbox.lock();
	    for ev in events.iter() {
		//println!("event log time: {}",  data.logical_time);
		// latency, should be made configurable later ...
		let inst = ruff.prepare_instance(map_name(&ev.name), 2.0, 0);
		
		for (k,v) in ev.params.iter() {
		    // println!("{} {}",k,v);
		    ruff.set_instance_parameter(inst, map_parameter(k), *v);
		}
		ruff.trigger(inst);
	    }
	    
	    (data.generator.current_transition().params["duration"] as f64 / 1000.0) as f64
	};

	// start scheduler if it exists ...
	if let Some(sched) = self.schedulers.get_mut(&gen.name) {
	    sched.start(eval_loop, gen, ruffbox);
	}		
    }

    pub fn stop_generator(&mut self, gen_name: &String) {
	if let Some(sched) = self.schedulers.get_mut(gen_name) {
	    sched.stop();
	}
    }

    pub fn clear_session(&mut self) {
	for (k,v) in self.schedulers.iter_mut() {
	    println!("stop generator \'{}\'", k);
	    v.stop();
	}
	self.schedulers = HashMap::new();
    }
}
