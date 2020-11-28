use std::collections::HashMap;
use std::sync;
use parking_lot::Mutex;

use ruffbox_synth::ruffbox::Ruffbox;

use crate::scheduler::{Scheduler, SchedulerData};
use crate::generator::Generator;
use crate::event_helpers::*;

#[derive(Clone,Copy,PartialEq)]
pub enum OutputMode {
    Stereo,
    // AmbisonicsBinaural,
    // Ambisonics
    FourChannel,
    EightChannel,
    //SixteenChannel,
    //TwentyFourChannel,           
}

pub struct SyncContext {
    pub name: String,
    pub synced: Vec<String>,
    pub generators: Vec<Generator>
}

pub struct Session <const BUFSIZE:usize, const NCHAN:usize> {
    schedulers: HashMap<String, Scheduler<BUFSIZE, NCHAN>>,
    output_mode: OutputMode,
}

impl <const BUFSIZE:usize, const NCHAN:usize> Session<BUFSIZE, NCHAN> {

    pub fn with_mode(mode: OutputMode) -> Self {
	Session {
	    schedulers: HashMap::new(),
	    output_mode: mode,
	}
    }
    
    pub fn start_generator(&mut self, gen: Box<Generator>, ruffbox: sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>) {

	// start scheduler if it exists ...
	if let Some(sched) = self.schedulers.get_mut(&gen.name) {
	    sched.stop();
	}

	// replace old scheduler (this will be where state handover needs to happen)
	self.schedulers.insert(gen.name.clone(), Scheduler::<BUFSIZE, NCHAN>::new());	

	// the evaluation function ...
	// or better, the inside part of the time recursion
	let eval_loop = |data: &mut SchedulerData<BUFSIZE, NCHAN>| -> f64 {
	    
	    let events = data.generator.current_events();
	    let mut ruff = data.ruffbox.lock();
	    for ev in events.iter() {
		//println!("event: {}",  ev.name);
		
		if ev.name == "silence" {
		    continue;
		}
		
		let mut bufnum:usize = 0;
		if let Some(b) = ev.params.get("bufnum") {
		    bufnum = *b as usize;
		}
		
		// latency 0.05, should be made configurable later ...
		let inst = ruff.prepare_instance(map_name(&ev.name), data.stream_time + 0.05, bufnum);
		
		for (k,v) in ev.params.iter() {
		    //println!("{} {}",k,v);

		    // special handling for stereo param
		    if k == "pos" && data.mode == OutputMode::Stereo {			
			let pos = (*v + 1.0) / 2.0;			
			ruff.set_instance_parameter(inst, map_parameter(k), pos);
		    } else {
			ruff.set_instance_parameter(inst, map_parameter(k), *v);
		    }
		}
		ruff.trigger(inst);
	    }
	    
	    (data.generator.current_transition().params["duration"] as f64 / 1000.0) as f64
	};

	// start scheduler if it exists ...
	if let Some(sched) = self.schedulers.get_mut(&gen.name) {
	    sched.start(self.output_mode, eval_loop, gen, ruffbox);
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
