use std::collections::{BTreeSet, HashMap};
use std::sync;
use parking_lot::Mutex;

use ruffbox_synth::ruffbox::Ruffbox;
use ruffbox_synth::ruffbox::synth::SynthParameter;

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
    pub sync_to: Option<String>,
    pub active: bool,
    pub generators: Vec<Generator>
}

pub struct Session <const BUFSIZE:usize, const NCHAN:usize> {
    schedulers: HashMap<BTreeSet<String>, (Scheduler<BUFSIZE, NCHAN>, sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>)>,
    output_mode: OutputMode,
    contexts: HashMap<String, BTreeSet<BTreeSet<String>>>,
}

impl <const BUFSIZE:usize, const NCHAN:usize> Session<BUFSIZE, NCHAN> {

    pub fn with_mode(mode: OutputMode) -> Self {
	Session {
	    schedulers: HashMap::new(),
	    output_mode: mode,
	    contexts: HashMap::new(),	    
	}
    }

    pub fn handle_context(&mut self, ctx: &mut SyncContext, ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>) {
	let name = ctx.name.clone();
	if ctx.active {	    
	    let mut new_gens = BTreeSet::new();

	    // collect id_tags
	    for c in ctx.generators.iter() {		
		new_gens.insert(c.id_tags.clone());		
	    }

	    // c=alc difference, stop vanished ones, sync new ones ...	    
	    if let Some(old_gens) = self.contexts.get(&name) { // this means context is running		
		let diff:Vec<_> = old_gens.difference(&new_gens).cloned().collect();
		for tags in diff.iter() {		    
		    self.stop_generator(&tags);
		}		
	    }

	    // is there rally no way around the second lookup ?
	    let mut remainders:Vec<_> = Vec::new();
	    if let Some(old_gens) = self.contexts.get(&name) { // this means context is running
		remainders = old_gens.intersection(&new_gens).cloned().collect();				
	    }
	    
	    if !remainders.is_empty() {

		let mut smallest_id = None;
		let mut last_len:usize = 10000000; // usize max would be better ... 
		for tags in remainders.iter() {		  		    
		    if tags.len() < last_len {
			last_len = tags.len();
			smallest_id = Some(tags);
		    }
		}

		if let Some(tags) = smallest_id {
		    print!("sync to existing: \'");
		    for tag in tags.iter() {
			print!("{} ", tag);
		    }
		    println!("\'");			
		}
		
		
		for c in ctx.generators.drain(..) {
		    // nothing to sync to in that case ...
		    self.start_generator(Box::new(c), sync::Arc::clone(ruffbox), smallest_id);
		}
	    } else {
		for c in ctx.generators.drain(..) {
		    // nothing to sync to in that case ...
		    self.start_generator(Box::new(c), sync::Arc::clone(ruffbox), None);
		}
	    }
	    
	    self.contexts.insert(name, new_gens);	    	    
	} else {
	    // stop all that were kept in this context, remove context ...
	    if let Some(old_ctx) = self.contexts.get(&name) {				
		for tags in old_ctx.clone().iter() { // this is the type of clone i hate ...		    
		    self.stop_generator(&tags);
		}
		self.contexts.remove(&name);
	    }
	}		
    }
    
    pub fn start_generator(&mut self, gen: Box<Generator>, ruffbox: sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>, sync: Option<&BTreeSet<String>>) {

	let id_tags = gen.id_tags.clone();
	// start scheduler if it exists ...
	if let Some((_, data)) = self.schedulers.get_mut(&id_tags) {
	    // resume sync: later ...
	    print!("resume generator \'");
	    for tag in id_tags.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");
	    
	    // keep the scheduler running, just replace the data ...
	    let mut sched_data = data.lock();
	    *sched_data = SchedulerData::<BUFSIZE, NCHAN>::from_previous(&sched_data, gen, ruffbox);
	} else {
	    print!("start generator \'");
	    for tag in id_tags.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");

	    let sched_data:sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>;
	    if let Some(id_tags) = sync {
		//this is prob kinda redundant 
		if let Some((_, data)) = self.schedulers.get_mut(&id_tags) {
		    sched_data = sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_previous(&data.lock(), gen, ruffbox)));	    
		} else {
		    sched_data = sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_data(gen, ruffbox, self.output_mode)));	    
		}
		
	    } else {
		sched_data = sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_data(gen, ruffbox, self.output_mode)));	    
	    }
	    
	    // otherwise, create new sched and data ...
	    let mut sched = Scheduler::<BUFSIZE, NCHAN>::new();
		    
	    // the evaluation function ...
	    // or better, the inside part of the time recursion
	    let eval_loop = |data: &mut SchedulerData<BUFSIZE, NCHAN>| -> f64 {
		
		let events = data.generator.current_events();
		let mut ruff = data.ruffbox.lock();
		for ev in events.iter() {

		    // no need to allocate a string everytime here, should be changed
		    if ev.name == "silence" {
			continue;
		    }
		    
		    let mut bufnum:usize = 0;
		    if let Some(b) = ev.params.get(&SynthParameter::SampleBufferNumber) {
			bufnum = *b as usize;
		    }
		    
		    // latency 0.05, should be made configurable later ...
		    let inst = ruff.prepare_instance(map_name(&ev.name), data.stream_time + 0.05, bufnum);
		    
		    for (k,v) in ev.params.iter() {
			// special handling for stereo param
			if k == &SynthParameter::ChannelPosition && data.mode == OutputMode::Stereo {			
			    let pos = (*v + 1.0) * 0.5;			
			    ruff.set_instance_parameter(inst, *k, pos);
			} else {
			    ruff.set_instance_parameter(inst, *k, *v);
			}
		    }
		    ruff.trigger(inst);
		}

		(data.generator.current_transition().params[&SynthParameter::Duration] as f64 / 1000.0) as f64
	    };
	    
	    sched.start(eval_loop, sync::Arc::clone(&sched_data));
	    self.schedulers.insert(id_tags, (sched, sched_data));
	}		
    }

    pub fn stop_generator(&mut self, gen_name: &BTreeSet<String>) {
	let mut found = false;
	if let Some((sched, _)) = self.schedulers.get_mut(gen_name) {
	    sched.stop();
	    found = true;
	}

	if found {
	    self.schedulers.remove(gen_name);

	    print!("stopped/removed generator \'");
	    for tag in gen_name.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");	    
	}
    }

    pub fn clear_session(&mut self) {
	for (k,(sched, _)) in self.schedulers.iter_mut() {	    
	    sched.stop();

	    print!("stopped/removed generator \'");
	    for tag in k.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");	    
	}
	self.schedulers = HashMap::new();
    }
}
