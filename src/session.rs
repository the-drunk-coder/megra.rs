use std::collections::{BTreeSet, HashMap};
use std::sync;
use parking_lot::Mutex;

use ruffbox_synth::ruffbox::Ruffbox;
use ruffbox_synth::ruffbox::synth::SynthParameter;

use crate::scheduler::{Scheduler, SchedulerData};
use crate::generator::Generator;
use crate::event::{InterpretableEvent};
use crate::event_helpers::*;
use crate::builtin_types::GlobalParameters;

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

#[derive(Clone)]
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
    
    pub fn handle_context(ctx: &mut SyncContext,
			  session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,			  
			  ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
			  global_parameters: &sync::Arc<GlobalParameters> ) {
	let name = ctx.name.clone();
	if ctx.active {	    
	    let mut new_gens = BTreeSet::new();

	    // collect id_tags
	    for c in ctx.generators.iter() {		
		new_gens.insert(c.id_tags.clone());		
	    }

	    // calc difference, stop vanished ones, sync new ones ...
	    let mut difference:Vec<_> = Vec::new();
	    let mut remainders:Vec<_> = Vec::new();

	    {
		let sess = session.lock();
		if let Some(old_gens) = sess.contexts.get(&name) { // this means context is running		
		    remainders = new_gens.intersection(&old_gens).cloned().collect();
		    difference = new_gens.symmetric_difference(&old_gens).cloned().collect();				    		
		}
	    }
	    
	    for tags in difference.iter() {		    
		Session::stop_generator(session, &tags);
	    }
	    
	    if let Some(sync) = &ctx.sync_to {
		
		let mut smallest_id = None;	    
		{
		    let sess = session.lock();
		    if let Some(sync_gens) = sess.contexts.get(sync) {
			// if there's both old and new generators	    	    
			
			let mut last_len:usize = usize::MAX; // usize max would be better ...
			
			for tags in sync_gens.iter() {		  		    
			    if tags.len() < last_len {
				last_len = tags.len();
				smallest_id = Some(tags.clone());
			    }
			}								    		    
		    }
		}

		if let Some(ref tags) = smallest_id {
		    print!("sync to existing: \'");
		    for tag in tags.iter() {
			print!("{} ", tag);
		    }
		    println!("\'");			
		}

		for c in ctx.generators.drain(..) {
		    // sync to what is most likely the root generator 
		    Session::start_generator(Box::new(c), session, ruffbox, global_parameters, &smallest_id);
		}
		
	    } else if !remainders.is_empty() && !difference.is_empty() {
		// if there's both old and new generators	    	    
		let mut smallest_id = None;
		let mut last_len:usize = usize::MAX; // usize max would be better ... 
		for tags in remainders.iter() {		  		    
		    if tags.len() < last_len {
			last_len = tags.len();
			smallest_id = Some(tags.clone());
		    }
		}

		if let Some(ref tags) = smallest_id {
		    print!("sync to existing: \'");
		    for tag in tags.iter() {
			print!("{} ", tag);
		    }
		    println!("\'");			
		}
		
		for c in ctx.generators.drain(..) {
		    // sync to what is most likely the root generator 
		    Session::start_generator(Box::new(c), session, ruffbox, global_parameters, &smallest_id);
		}
	    } else {
		for c in ctx.generators.drain(..) {
		    // nothing to sync to in that case ...
		    Session::start_generator(Box::new(c), session, ruffbox, global_parameters, &None);
		}
	    }

	    // insert new context
	    {
		let mut sess = session.lock();
		sess.contexts.insert(name, new_gens);
	    }
	} else {	    
	    // stop all that were kept in this context, remove context ...
	    let mut an_old_ctx = None;
	    {
		let sess = session.lock();
		if let Some(old_ctx) = sess.contexts.get(&name) {				
		    an_old_ctx = Some(old_ctx.clone());
		}
	    }
	    if let Some(old_ctx) = an_old_ctx {
		for tags in old_ctx.iter() { // this is the type of clone i hate ...		    
		    Session::stop_generator(session, tags);
		}
	    }	    	    	    	    
	    // remove context 
	    {
		let mut sess = session.lock();
		sess.contexts.remove(&name);
	    }	    
	}
    }		

    pub fn start_generator(gen: Box<Generator>,
			   session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,			   
			   ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
			   global_parameters: &sync::Arc<GlobalParameters>,
			   sync: &Option<BTreeSet<String>>) {
	
	let mut sess = session.lock();
	let id_tags = gen.id_tags.clone();
	// start scheduler if it exists ...
	if let Some((_, data)) = sess.schedulers.get_mut(&id_tags) {
	    // resume sync: later ...
	    print!("resume generator \'");
	    for tag in id_tags.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");
	    
	    // keep the scheduler running, just replace the data ...
	    let mut sched_data = data.lock();
	    *sched_data = SchedulerData::<BUFSIZE, NCHAN>::from_previous(&sched_data, gen, ruffbox, session);
	} else {
	    print!("start generator \'");
	    for tag in id_tags.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");

	    let sched_data:sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>;
	    if let Some(id_tags) = sync {
		//this is prob kinda redundant 
		if let Some((_, data)) = sess.schedulers.get_mut(&id_tags) {
		    // synchronize timing data 
		    sched_data = sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_previous(&data.lock(), gen, ruffbox, session)));	    
		} else {
		    sched_data = sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_data(gen, session, ruffbox, global_parameters, sess.output_mode)));
		}		
	    } else {
		sched_data = sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_data(gen, session, ruffbox, global_parameters, sess.output_mode)));
	    }
	    
	    // otherwise, create new sched and data ...
	    let mut sched = Scheduler::<BUFSIZE, NCHAN>::new();
	    
	    // the evaluation function ...
	    // or better, the inside part of the time recursion
	    let eval_loop = |data: &mut SchedulerData<BUFSIZE, NCHAN>| -> f64 {
		
		let events = data.generator.current_events(&data.global_parameters);
		for ev in events.iter() {
		    match ev {
			InterpretableEvent::Sound(s) => {			    
			    // no need to allocate a string everytime here, should be changed
			    if s.name == "silence" {
				continue;
			    }
			    
			    let mut bufnum:usize = 0;
			    if let Some(b) = s.params.get(&SynthParameter::SampleBufferNumber) {
				bufnum = *b as usize;
			    }

			    let mut ruff = data.ruffbox.lock();
			    // latency 0.05, should be made configurable later ...
			    let inst = ruff.prepare_instance(map_name(&s.name), data.stream_time + 0.05, bufnum);
			    // set parameters and trigger instance
			    for (k,v) in s.params.iter() {
				// special handling for stereo param
				match k {
				    SynthParameter::ChannelPosition => {
					if data.mode == OutputMode::Stereo {			
					    let pos = (*v + 1.0) * 0.5;			
					    ruff.set_instance_parameter(inst, *k, pos);
					} else {
					    ruff.set_instance_parameter(inst, *k, *v);
					}
				    },
				    SynthParameter::Duration => ruff.set_instance_parameter(inst, *k, *v * 0.001),
				    SynthParameter::Attack => ruff.set_instance_parameter(inst, *k, *v * 0.001),
				    SynthParameter::Sustain => ruff.set_instance_parameter(inst, *k, *v * 0.001),
				    SynthParameter::Release => ruff.set_instance_parameter(inst, *k, *v * 0.001),				    
				    _ => ruff.set_instance_parameter(inst, *k, *v),
				}
				
			    }			    
			    ruff.trigger(inst);
			},		    
			InterpretableEvent::Control(c) => {
			    
			    if let Some(mut contexts) = c.ctx.clone() { // this is the worst clone .... 
				for mut sx in contexts.drain(..) {				    
				    Session::handle_context(&mut sx, &data.session, &data.ruffbox, &data.global_parameters);
				}
			    }			    
			}
		    }
		}
				
		(data.generator.current_transition(&data.global_parameters).params[&SynthParameter::Duration] as f64 * 0.001) as f64
	    };
	    
	    sched.start(eval_loop, sync::Arc::clone(&sched_data));
	    sess.schedulers.insert(id_tags, (sched, sched_data));
	}		
    }

    pub fn stop_generator(session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>, gen_name: &BTreeSet<String>) {
	let mut sess = session.lock();
	let mut found = false;
	if let Some((sched, _)) = sess.schedulers.get_mut(gen_name) {
	    sched.stop();
	    found = true;
	}

	if found {
	    sess.schedulers.remove(gen_name);

	    print!("stopped/removed generator \'");
	    for tag in gen_name.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");	    
	}
    }

    pub fn clear_session(session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>) {
	let mut sess = session.lock();
	for (k,(sched, _)) in sess.schedulers.iter_mut() {	    
	    sched.stop();

	    print!("stopped/removed generator \'");
	    for tag in k.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");	    
	}
	sess.schedulers = HashMap::new();
    }
}


