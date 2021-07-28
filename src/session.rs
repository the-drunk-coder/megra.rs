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
        {
            let ps = parts_store.lock();

            let mut gens = Vec::new();

            for p in ctx.part_proxies.drain(..) {
                resolve_proxy(&ps, p, &mut gens);
            }

            for (i, gen) in gens.iter_mut().enumerate() {
                gen.id_tags.insert(format!("prox-{}", i));
                gen.id_tags.insert(ctx.name.clone());
            }

            ctx.generators.append(&mut gens);
        }

        let name = ctx.name.clone();
        if ctx.active {
            let mut new_gens = BTreeSet::new();

            // collect id_tags
            for c in ctx.generators.iter() {
                new_gens.insert(c.id_tags.clone());
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

	    //println!("rem {:?}", remainders);
	    //println!("diff1 {:?}", difference);
	    //println!("diff2 {:?}", difference2);
	    	    	    
            if let Some(sync) = &ctx.sync_to {
                let mut smallest_id = None;
                {
                    let sess = session.lock();
                    if let Some(sync_gens) = sess.contexts.get(sync) {
                        // if there's both old and new generators

                        let mut last_len: usize = usize::MAX; 

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
                    Session::start_generator(
                        Box::new(c),
                        session,
                        ruffbox,
                        parts_store,
                        global_parameters,
                        &smallest_id,
                        ctx.shift as f64 * 0.001,
                    );
                }
            } else if !remainders.is_empty() && !newcomers.is_empty() {
                // if there's both old and new generators
                let mut smallest_id = None;
                let mut last_len: usize = usize::MAX; // usize max would be better ...
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
                    Session::start_generator(
                        Box::new(c),
                        session,
                        ruffbox,
                        parts_store,
                        global_parameters,
                        &smallest_id,
                        ctx.shift as f64 * 0.001,
                    );
                }
            } else {
                for c in ctx.generators.drain(..) {
                    // nothing to sync to in that case ...
                    Session::start_generator(
                        Box::new(c),
                        session,
                        ruffbox,
                        parts_store,
                        global_parameters,
                        &None,
                        ctx.shift as f64 * 0.001,
                    );
                }
            }

	    // stop asynchronously to keep main thread reactive
	    let session2 = sync::Arc::clone(session);
	    thread::spawn(move || {
		for tags in quitters.iter() {
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

    pub fn start_generator(
        gen: Box<Generator>,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
        global_parameters: &sync::Arc<GlobalParameters>,
        sync: &Option<BTreeSet<String>>,
        shift: f64,
    ) {
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
            *sched_data = SchedulerData::<BUFSIZE, NCHAN>::from_previous(
                &sched_data,
                shift,
                gen,
                ruffbox,
                session,
                parts_store,
            );
        } else {
            print!("start generator \'");
            for tag in id_tags.iter() {
                print!("{} ", tag);
            }
            println!("\'");

            let sched_data: sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>;
            if let Some(id_tags) = sync {
                //this is prob kinda redundant
                if let Some((_, data)) = sess.schedulers.get_mut(&id_tags) {
                    // synchronize timing data
                    sched_data = sync::Arc::new(Mutex::new(
                        SchedulerData::<BUFSIZE, NCHAN>::from_time_data(
                            &data.lock(),
                            shift,
                            gen,
                            ruffbox,
                            session,
                            parts_store,
                        ),
                    ));
                } else {
                    sched_data =
                        sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_data(
                            gen,
                            shift,
                            session,
                            ruffbox,
                            parts_store,
                            global_parameters,
                            sess.output_mode,
                        )));
                }
            } else {
                sched_data =
                    sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_data(
                        gen,
                        shift,
                        session,
                        ruffbox,
                        parts_store,
                        global_parameters,
                        sess.output_mode,
                    )));
            }

            // otherwise, create new sched and data ...
            let mut sched = Scheduler::<BUFSIZE, NCHAN>::new();

            //////////////////////////////////////
            // THE MAIN TIME RECURSION LOOP!!!  //
            //////////////////////////////////////

            // yes, here it is ... the evaluation function ...
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
            sess.schedulers.insert(id_tags, (sched, sched_data));
        }
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
