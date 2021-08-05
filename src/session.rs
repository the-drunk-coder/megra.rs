use parking_lot::Mutex;
use std::collections::{BTreeSet, HashMap};
use std::{sync, thread};

use ruffbox_synth::ruffbox::synth::SynthParameter;
use ruffbox_synth::ruffbox::Ruffbox;

use crate::builtin_types::{
    BuiltinGlobalParameters, Command, ConfigParameter, GeneratorProcessorOrModifier,
    GlobalParameters, Part, PartProxy, PartsStore,
};
use crate::commands;
use crate::event::InterpretableEvent;
use crate::event_helpers::*;
use crate::generator::Generator;
use crate::parameter::*;
use crate::scheduler::{Scheduler, SchedulerData};

#[derive(Clone, Copy, PartialEq)]
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
    pub generators: Vec<Generator>,
    pub part_proxies: Vec<PartProxy>,
    pub shift: i32,
}

pub struct Session<const BUFSIZE: usize, const NCHAN: usize> {
    schedulers: HashMap<
            BTreeSet<String>,
        (
            Scheduler<BUFSIZE, NCHAN>,
            sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>,
        ),
	>,
    pub output_mode: OutputMode,
    contexts: HashMap<String, BTreeSet<BTreeSet<String>>>,
}

// basically a bfs on a dag !
fn resolve_proxy(parts_store: &PartsStore, proxy: PartProxy, generators: &mut Vec<Generator>) {
    match proxy {
        PartProxy::Proxy(s, procs) => {
            //visited.push(s);
            if let Some(Part::Combined(part_generators, proxies)) = parts_store.get(&s) {
                // this can be done for sure ...
                for mut gen in part_generators.clone().drain(..) {
                    let mut procs_clone = procs.clone();
                    let mut gpl_drain = procs_clone.drain(..);
                    while let Some(gpom) = gpl_drain.next() {
                        match gpom {
                            GeneratorProcessorOrModifier::GeneratorProcessor(gp) => {
                                gen.processors.push(gp)
                            }
                            GeneratorProcessorOrModifier::GeneratorModifierFunction((
                                fun,
                                pos,
                                named,
                            )) => fun(&mut gen.root_generator, &mut Vec::new(), &pos, &named),
                        }
                    }
                    //gen.id_tags.insert(s.clone());
                    generators.push(gen);
                }

                for sub_proxy in proxies.clone().drain(..) {
                    let mut sub_gens = Vec::new();
                    resolve_proxy(parts_store, sub_proxy, &mut sub_gens);
                    for mut gen in sub_gens.drain(..) {
                        let mut procs_clone = procs.clone();
                        let mut gpl_drain = procs_clone.drain(..);
                        while let Some(gpom) = gpl_drain.next() {
                            match gpom {
                                GeneratorProcessorOrModifier::GeneratorProcessor(gp) => {
                                    gen.processors.push(gp)
                                }
                                GeneratorProcessorOrModifier::GeneratorModifierFunction((
                                    fun,
                                    pos,
                                    named,
                                )) => fun(&mut gen.root_generator, &mut Vec::new(), &pos, &named),
                            }
                        }
                        //gen.id_tags.insert(s.clone());
                        generators.push(gen);
                    }
                }
            }
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Session<BUFSIZE, NCHAN> {
    pub fn with_mode(mode: OutputMode) -> Self {
        Session {
            schedulers: HashMap::new(),
            output_mode: mode,
            contexts: HashMap::new(),
        }
    }

    pub fn handle_context(
        ctx: &mut SyncContext,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
        global_parameters: &sync::Arc<GlobalParameters>,
    ) {
        // resolve part proxies ..
        // at some point this should probably check if
        // there's loops and so on ...
	// do it in a block to keep locking time short 
        {            	    
            let mut gens = Vec::new();

	    let ps = parts_store.lock();
            for p in ctx.part_proxies.drain(..) {
                resolve_proxy(&ps, p, &mut gens);
            }

            for (i, gen) in gens.iter_mut().enumerate() {
                gen.id_tags.insert(format!("prox-{}", i));
                gen.id_tags.insert(ctx.name.clone());
            }

            ctx.generators.append(&mut gens);
        } // end resolve proxies

        let name = ctx.name.clone(); // keep a copy for later
        if ctx.active {
	    
	    // otherwise, handle internal sync relations ...
	    let mut new_gens = BTreeSet::new();
	    let mut gen_map: HashMap<BTreeSet<String>, Generator> = HashMap::new();
            // collect id_tags and organize in map
            for g in ctx.generators.drain(..) {
                new_gens.insert(g.id_tags.clone());
		gen_map.insert(g.id_tags.clone(), g);
            }

            // calc difference, stop vanished ones, sync new ones ...
            let mut newcomers: Vec<_> = Vec::new();
	    let mut quitters: Vec<_> = Vec::new();
            let mut remainders: Vec<_> = Vec::new();
            {
                let sess = session.lock();
                if let Some(old_gens) = sess.contexts.get(&name) {
                    // this means context is running
                    remainders = new_gens.intersection(&old_gens).cloned().collect();
                    newcomers = new_gens.difference(&old_gens).cloned().collect();
		    quitters = old_gens.difference(&new_gens).cloned().collect();
                }
            }

	    println!("newcomers {:?}", newcomers);
	    println!("remainders {:?}", remainders);
	    println!("quitters {:?}", quitters);

	    // EXTERNAL SYNC
	    // are we supposed to sync to some other context ??
	    // get external sync ...
            let external_sync = if let Some(sync) = &ctx.sync_to {
		let mut smallest_id = None;
                {
                    let sess = session.lock();
                    if let Some(sync_gens) = sess.contexts.get(sync) {                        
                        let mut last_len: usize = usize::MAX; 			
                        for tags in sync_gens.iter() {
                            if tags.len() < last_len {
                                last_len = tags.len();
                                smallest_id = Some(tags.clone());
                            }
                        }
                    }
                }
		smallest_id			
            } else {
		None
	    }; // END EXTERNAL SYNC

	    // INTERNAL SYNC
	    // get context-internal sync in case there are newcomers	    
	    let mut internal_sync = None;
            let mut last_len: usize = usize::MAX; // usize max would be better ...
            for tags in remainders.iter() {
                if tags.len() < last_len {
                    last_len = tags.len();
                    internal_sync = Some(tags.clone());
                }
            } // END INTERNAL SYNC

	    // HANDLE REMAINDERS
	    if let Some(ext_sync) = external_sync.clone() {
		for rem in remainders.drain(..) {
		    let gen = gen_map.remove(&rem).unwrap();		    
		    Session::resume_generator_sync(Box::new(gen),
						   &session,
						   &ruffbox,
						   &parts_store,
						   &ext_sync,
						   ctx.shift as f64 * 0.001);
		}
	    } else {
		for rem in remainders.drain(..) {
		    let gen = gen_map.remove(&rem).unwrap();
		    // WHAT TO DO IN SYNC CASE ?
		    Session::resume_generator(Box::new(gen),
					      &session,
					      &ruffbox,
					      &parts_store,
					      ctx.shift as f64 * 0.001);
		}
	    } // END HANDLE REMAINDERS
	    
	    // HANDLE NEWCOMERS	    	    
	    if let Some(ext_sync) = external_sync.clone() {
		// external sync has precedence
		for nc in newcomers.drain(..) {
		    let gen = gen_map.remove(&nc).unwrap();
		    Session::start_generator_push_sync(Box::new(gen),
						       &session,
						       &ext_sync,
		    				       ctx.shift as f64 * 0.001);
		}
	    } else if let Some(int_sync) = internal_sync.clone() {
		for nc in newcomers.drain(..) {
		    let gen = gen_map.remove(&nc).unwrap();
		    Session::start_generator_push_sync(Box::new(gen),
						       &session,
						       &int_sync,
		    				       ctx.shift as f64 * 0.001);
		}
	    } else {
		for nc in newcomers.drain(..) {
		    let gen = gen_map.remove(&nc).unwrap();
		    Session::start_generator_no_sync(Box::new(gen),
						     &session,
						     &ruffbox,
						     &parts_store,
						     &global_parameters,
						     ctx.shift as f64 * 0.001);
		}
	    }
	    // END HANDLE NEWCOMERS

	    // HANDLE LEFTOVERS OR FRESH CONTEXT (most likely the latter)
	    // if there's still gens in the map, handle those
	    // this will happen if, for example, we're handling an
	    // entirely new context, and there was no old generators
	    // to compare to ...
	    let leftovers_present = !gen_map.is_empty();
	    if leftovers_present {
		if let Some(ext_sync) = external_sync {
		    // external sync has precedence
		    for (_, gen) in gen_map.drain() {
			Session::start_generator_push_sync(Box::new(gen),
							   &session,
							   &ext_sync,
		    					   ctx.shift as f64 * 0.001);
		    }
		} else if let Some(int_sync) = internal_sync {
		    // this is very unlikely to happen, but just in case ...
		    for (_, gen) in gen_map.drain() {			
			Session::start_generator_push_sync(Box::new(gen),
							   &session,
							   &int_sync,
		    					   ctx.shift as f64 * 0.001);
		    }
		} else {
		    // common case ... 
		    for (_, gen) in gen_map.drain() {
			Session::start_generator_no_sync(Box::new(gen),
							 &session,
							 &ruffbox,
							 &parts_store,
							 &global_parameters,
							 ctx.shift as f64 * 0.001);
		    }
		}		
	    }

	    // HANDLE QUITTERS (generators to be stopped ...)
	    // stop asynchronously to keep main thread reactive
	    let session2 = sync::Arc::clone(session);
	    thread::spawn(move || {
		for tags in quitters.drain(..) {
                    Session::stop_generator(&session2, &tags);
		}
	    });
	    
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
                for tags in old_ctx.iter() {
                    // this is the type of clone i hate ...
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

    /// if a generater is already active, it'll be resumed by replacing its scheduler data
    fn resume_generator(gen: Box<Generator>,
			session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
			ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
			parts_store: &sync::Arc<Mutex<PartsStore>>,
			shift: f64) {
	let mut sess = session.lock();
        let id_tags = gen.id_tags.clone();
        // start scheduler if it exists ...
        if let Some((_, data)) = sess.schedulers.get_mut(&id_tags) {            
            print!("resume generator \'");
            for tag in id_tags.iter() {
                print!("{} ", tag);
            }
            println!("\'");

            // keep the scheduler running, just replace the data ...
            let mut sched_data = data.lock();
            *sched_data = SchedulerData::<BUFSIZE, NCHAN>::from_previous(
                &sched_data,
                shift,
                gen,
                ruffbox,
                session,
                parts_store,
            );
        }	
    }

    fn resume_generator_sync(gen: Box<Generator>,
			     session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
			     ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
			     parts_store: &sync::Arc<Mutex<PartsStore>>,
			     sync_tags: &BTreeSet<String>,
			     shift: f64) {
	let mut sess = session.lock();
        let id_tags = gen.id_tags.clone();
        // start scheduler if it exists ...

	// thanks, borrow checker, for this elegant construction ...
	let s_data = if let Some((_, sd)) = sess.schedulers.get(sync_tags) {
	    Some(sd.clone())
	} else {
	    None
	};
	
        if let Some((_, data)) = sess.schedulers.get_mut(&id_tags) {
	    if let Some(sync_data) = s_data  {
		// resume sync: later ...
		print!("resume sync  generator \'");
		for tag in id_tags.iter() {
                    print!("{} ", tag);
		}
		println!("\'");

		// keep the scheduler running, just replace the data ...
		let mut sched_data = data.lock();
		let sync_sched_data = sync_data.lock();
		*sched_data = SchedulerData::<BUFSIZE, NCHAN>::from_time_data(
                    &sync_sched_data,
                    shift,
                    gen,
                    ruffbox,
                    session,
                    parts_store,
		);
            } else {
		// resume sync: later ...
		print!("resume generator \'");
		for tag in id_tags.iter() {
                    print!("{} ", tag);
		}
		println!("\'");
		// keep the scheduler running, just replace the data ...
		let mut sched_data = data.lock();		
		*sched_data = SchedulerData::<BUFSIZE, NCHAN>::from_previous(
                    &sched_data,
                    shift,
                    gen,
                    ruffbox,
                    session,
                    parts_store,
		);
	    }
	}
    }

    /// start, sync time data ...
    fn start_generator_data_sync(
        gen: Box<Generator>,        
        data: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
    ) {
	let id_tags = gen.id_tags.clone();

	print!("start generator (sync time data) \'");
	for tag in id_tags.iter() {
	    print!("{} ", tag);
	}
	println!("\'");
	// sync to data
	// create sched data from data
	let sched_data = sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_time_data(
            data,
            shift,
	    gen,
	    &data.ruffbox,
            &data.session,
            &data.parts_store,            
        )));
	Session::start_scheduler(&data.session, sched_data, id_tags)
    }

    // push to synced gen's sync list ...
    // if it doesn't exist, just start ...
    fn start_generator_push_sync(
        gen: Box<Generator>,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        sync_tags: &BTreeSet<String>,
        shift: f64,
    ) {
	//this is prob kinda redundant
	let mut sess = session.lock();
        if let Some((_, data)) = sess.schedulers.get_mut(&sync_tags) {
	    print!("start generator \'");
	    for tag in gen.id_tags.iter() {
		print!("{} ", tag);
	    }		
	    print!("\' (push sync to existing \'");
	    for tag in sync_tags.iter() {
		print!("{} ", tag);
            }
	    println!("\')");
	    
	    let mut dlock = data.lock();
	    // push to sync ...
	    dlock.synced_generators.push((gen, shift));	    
	}
    }
    
    pub fn start_generator_no_sync(
        gen: Box<Generator>,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
        global_parameters: &sync::Arc<GlobalParameters>,        
        shift: f64,
    ) {
        let id_tags = gen.id_tags.clone();

	print!("start generator (no sync) \'");
	for tag in id_tags.iter() {
	    print!("{} ", tag);
	}
	println!("\'");
	
        let sched_data = sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_data(
            gen,
            shift,
            session,
            ruffbox,
            parts_store,
            global_parameters,
            session.lock().output_mode,
        )));
	Session::start_scheduler(session, sched_data, id_tags)
    }	    
    
    fn start_scheduler(session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
		       sched_data: sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>,
		       id_tags: BTreeSet<String>) {	
	// otherwise, create new sched and data ...
        let mut sched = Scheduler::<BUFSIZE, NCHAN>::new();

        //////////////////////////////////////
        // THE MAIN TIME RECURSION LOOP!!!  //
        //////////////////////////////////////

        // yes, here it is ... the evaluation function ...
        // or better, the inside part of the time recursion
        let eval_loop = |data: &mut SchedulerData<BUFSIZE, NCHAN>| -> f64 {
            let events = data.generator.current_events(&data.global_parameters);

	    // start the generators ready to be synced ...
	    let mut syncs = data.synced_generators.clone();
	    for (g, s) in syncs.drain(..) {
		println!("sync drain");
		Session::start_generator_data_sync(g,
						   data,
						   s);
	    }
	    data.synced_generators.clear();
	    
            for ev in events.iter() {
                match ev {
                    InterpretableEvent::Sound(s) => {
                        // no need to allocate a string everytime here, should be changed
                        if s.name == "silence" {
                            continue;
                        }

                        let mut bufnum: usize = 0;
                        if let Some(b) = s.params.get(&SynthParameter::SampleBufferNumber) {
                            bufnum = *b as usize;
                        }

                        let mut ruff = data.ruffbox.lock();
                        // latency 0.05, should be made configurable later ...
                        let inst = ruff.prepare_instance(
                            map_name(&s.name),
                            data.stream_time + 0.05,
                            bufnum,
                        );
                        // set parameters and trigger instance
                        for (k, v) in s.params.iter() {
                            // special handling for stereo param
                            match k {
                                SynthParameter::ChannelPosition => {
                                    if data.mode == OutputMode::Stereo {
                                        let pos = (*v + 1.0) * 0.5;
                                        ruff.set_instance_parameter(inst, *k, pos);
                                    } else {
                                        ruff.set_instance_parameter(inst, *k, *v);
                                    }
                                }
                                // convert milliseconds to seconds
                                SynthParameter::Duration => {
                                    ruff.set_instance_parameter(inst, *k, *v * 0.001)
                                }
                                SynthParameter::Attack => {
                                    ruff.set_instance_parameter(inst, *k, *v * 0.001)
                                }
                                SynthParameter::Sustain => {
                                    ruff.set_instance_parameter(inst, *k, *v * 0.001)
                                }
                                SynthParameter::Release => {
                                    ruff.set_instance_parameter(inst, *k, *v * 0.001)
                                }
                                _ => ruff.set_instance_parameter(inst, *k, *v),
                            }
                        }
                        ruff.trigger(inst);
                    }
                    InterpretableEvent::Control(c) => {
                        if let Some(mut contexts) = c.ctx.clone() {
                            // this is the worst clone ....
                            for mut sx in contexts.drain(..) {
                                Session::handle_context(
                                    &mut sx,
                                    &data.session,
                                    &data.ruffbox,
                                    &data.parts_store,
                                    &data.global_parameters,
                                );
                            }
                        }
                        if let Some(mut commands) = c.cmd.clone() {
                            // this is the worst clone ....
                            for c in commands.drain(..) {
                                match c {
                                    Command::LoadPart((name, part)) => {
                                        commands::load_part(&data.parts_store, name, part);
                                        println!("a command (load part)");
                                    }
                                    Command::FreezeBuffer(freezbuf) => {
                                        commands::freeze_buffer(&data.ruffbox, freezbuf);
                                        println!("freeze buffer");
                                    }
                                    Command::Tmod(p) => {
                                        commands::set_global_tmod(&data.global_parameters, p);
                                    }
                                    Command::GlobRes(v) => {
                                        commands::set_global_lifemodel_resources(
                                            &data.global_parameters,
                                            v,
                                        );
                                    }
                                    Command::GlobalRuffboxParams(m) => {
                                        commands::set_global_ruffbox_parameters(
                                            &data.ruffbox,
                                            &m,
                                        );
                                    }
                                    _ => {
                                        println!("ignore command")
                                    }
                                };
                            }
                        }
                    }
                }
            }

            // global tempo modifier, allows us to do weird stuff with the
            // global tempo ...
            let mut tmod: f64 = 1.0;

            if let ConfigParameter::Dynamic(global_tmod) = data
                .global_parameters
                .entry(BuiltinGlobalParameters::GlobalTimeModifier)
                .or_insert(ConfigParameter::Dynamic(Parameter::with_value(1.0))) // init on first attempt
                .value_mut()
            {
                tmod = global_tmod.evaluate() as f64;
            }

            (data
             .generator
             .current_transition(&data.global_parameters)
             .params[&SynthParameter::Duration] as f64
             * 0.001
             * tmod) as f64
        };

        // assemble name for thread ...
        let mut thread_name: String = "".to_owned();
        for tag in id_tags.iter() {
            thread_name.push_str(&(format!("{} ", tag)));
        }

        sched.start(
            &thread_name.trim(),
            eval_loop,
            sync::Arc::clone(&sched_data),
        );
	let mut sess = session.lock();
        sess.schedulers.insert(id_tags, (sched, sched_data));	
    }
    
    pub fn stop_generator(
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        gen_name: &BTreeSet<String>,
    ) {
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

    pub fn clear_session(
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
    ) {
        let mut sess = session.lock();
        for (k, (sched, _)) in sess.schedulers.iter_mut() {
            sched.stop();

            print!("stopped/removed generator \'");
            for tag in k.iter() {
                print!("{} ", tag);
            }
            println!("\'");
        }
        sess.schedulers = HashMap::new();
	sess.contexts = HashMap::new();
        let mut ps = parts_store.lock();
        *ps = HashMap::new();
    }
}
