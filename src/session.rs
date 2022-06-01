use parking_lot::Mutex;
use std::collections::{BTreeSet, HashMap};
use std::{sync, thread};

use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
use ruffbox_synth::ruffbox::RuffboxControls;

use crate::builtin_types::{
    BuiltinGlobalParameters, Command, ConfigParameter, GeneratorProcessorOrModifier,
    GlobalParameters, Part, PartProxy, PartsStore,
};
use crate::commands;
use crate::event::InterpretableEvent;
use crate::event_helpers::*;
use crate::generator::Generator;
use crate::parameter::*;
use crate::real_time_streaming;
use crate::scheduler::{Scheduler, SchedulerData};
use crate::visualizer_client::VisualizerClient;

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

#[derive(Clone, Copy, PartialEq)]
pub enum SyncMode {
    All,          // sync on all events
    NotOnSilence, // don't sync on silent events
    OnlyOnSilence, // only sync on silent events
                  // OnMarkersOnly
                  // OnMarkersNotOnsilence // on specific markers ... not sure how to handle this yet ...
}

#[derive(Clone)]
pub struct SyncContext {
    pub name: String,
    pub sync_to: Option<String>,
    pub active: bool,
    pub generators: Vec<Generator>,
    pub part_proxies: Vec<PartProxy>,
    pub shift: i32,
    pub block_tags: BTreeSet<String>,
    pub solo_tags: BTreeSet<String>,
}

pub struct Session<const BUFSIZE: usize, const NCHAN: usize> {
    pub schedulers: HashMap<
        BTreeSet<String>,
        (
            Scheduler<BUFSIZE, NCHAN>,
            sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>,
        ),
    >,
    contexts: HashMap<String, BTreeSet<BTreeSet<String>>>,
    pub visualizer_client: Option<sync::Arc<VisualizerClient>>,
    pub rec_control: Option<real_time_streaming::RecordingControl<BUFSIZE, NCHAN>>,
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
                    for gpom in procs_clone.drain(..) {
                        match gpom {
                            GeneratorProcessorOrModifier::GeneratorProcessor(gp) => {
                                gen.processors.push(gp)
                            }
                            GeneratorProcessorOrModifier::GeneratorModifierFunction((
                                fun,
                                pos,
                                named,
                            )) => fun(&mut gen, &pos, &named),
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
                        for gpom in procs_clone.drain(..) {
                            match gpom {
                                GeneratorProcessorOrModifier::GeneratorProcessor(gp) => {
                                    gen.processors.push(gp)
                                }
                                GeneratorProcessorOrModifier::GeneratorModifierFunction((
                                    fun,
                                    pos,
                                    named,
                                )) => fun(&mut gen, &pos, &named),
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

//////////////////////////////////////
// THE MAIN TIME RECURSION LOOP!!!  //
//////////////////////////////////////

// yes, here it is ... the evaluation function ...
// or better, the inside part of the time iteration
fn eval_loop<const BUFSIZE: usize, const NCHAN: usize>(
    data: &mut SchedulerData<BUFSIZE, NCHAN>,
) -> (f64, bool, bool) {
    // global tempo modifier, allows us to do weird stuff with the
    // global tempo ...
    let mut tmod: f64 = 1.0;
    let mut latency: f64 = 0.05;

    if let ConfigParameter::Dynamic(global_tmod) = data
        .global_parameters
        .entry(BuiltinGlobalParameters::GlobalTimeModifier)
        .or_insert(ConfigParameter::Dynamic(Parameter::with_value(1.0))) // init on first attempt
        .value_mut()
    {
        tmod = global_tmod.evaluate_numerical() as f64;
    }

    if let ConfigParameter::Dynamic(global_latency) = data
        .global_parameters
        .entry(BuiltinGlobalParameters::GlobalLatency)
        .or_insert(ConfigParameter::Dynamic(Parameter::with_value(0.05))) // init on first attempt
        .value_mut()
    {
        latency = global_latency.evaluate_numerical() as f64;
    }

    if let Some(vc) = &data.visualizer_client {
        if data.generator.root_generator.is_modified() {
            vc.create_or_update(&data.generator);
            data.generator.root_generator.clear_modified()
        }
        vc.update_active_node(&data.generator);
        for proc in data.generator.processors.iter_mut() {
            proc.visualize_if_possible(vc);
        }
    }

    let time = if let SynthParameterValue::ScalarF32(t) = data
        .generator
        .current_transition(&data.global_parameters)
        .params[&SynthParameterLabel::Duration]
    {
        (t * 0.001) as f64 * tmod
    } else {
        0.2
    };

    // retrieve the current events
    let events = data.generator.current_events(&data.global_parameters);
    //if events.is_empty() {
    //    println!("really no events");
    //}
    let end_state = data.generator.reached_end_state();
    // the sync flag will be returned alongside the
    // time to let the scheduler know that it should
    // trigger the synced generators
    let mut sync = false;

    // start the generators ready to be synced ...
    if data.sync_mode == SyncMode::All {
        //println!("sync all");
        sync = true;
    }

    for ev in events.iter() {
        match ev {
            InterpretableEvent::Sound(s) => {
                // no need to allocate a string everytime here, should be changed
                if s.name == "silence" {
                    // start the generators ready to be synced ...
                    if data.sync_mode == SyncMode::OnlyOnSilence {
                        //println!("sync silence");
                        sync = true;
                    }
                    continue;
                }

                // start the generators ready to be synced ...
                if data.sync_mode == SyncMode::NotOnSilence {
                    //println!("sync nosilence");
                    sync = true;
                }

                //println!("solo: {:?}", data.solo_tags);
                //println!("block: {:?}", data.block_tags);

                if !data.block_tags.is_empty() && !data.block_tags.is_disjoint(&s.tags) {
                    // ignore event
                    continue;
                }

                if !data.solo_tags.is_empty() && data.solo_tags.is_disjoint(&s.tags) {
                    // ignore event
                    continue;
                }

                let mut bufnum: usize = 0;
                if let Some(SynthParameterValue::ScalarUsize(b)) =
                    s.params.get(&SynthParameterLabel::SampleBufferNumber)
                {
                    bufnum = *b;
                }

                if let Some(mut inst) = data.ruffbox.prepare_instance(
                    map_name(&s.name),
                    data.stream_time + latency,
                    bufnum,
                ) {
                    // set parameters and trigger instance
                    for (k, v) in s.params.iter() {
                        // special handling for stereo param
                        //println!("{:?}", k);
                        match k {
                            SynthParameterLabel::ChannelPosition => match v {
                                SynthParameterValue::ScalarF32(p) => {
                                    //println!("hello");
                                    if data.output_mode == OutputMode::Stereo {
                                        let pos = (*p + 1.0) * 0.5;
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::ScalarF32(pos),
                                        );
                                    } else {
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::ScalarF32(*p),
                                        );
                                    }
                                }
                                SynthParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => {
                                    //println!("hello {}", init);
                                    if data.output_mode == OutputMode::Stereo {
                                        let pos = (*init + 1.0) * 0.5;
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::Lfo(
                                                pos, *freq, *eff_phase, *amp, *add, *op,
                                            ),
                                        );
                                    } else {
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::Lfo(
                                                *init, *freq, *eff_phase, *amp, *add, *op,
                                            ),
                                        );
                                    }
                                }
                                SynthParameterValue::LFSaw(init, freq, eff_phase, amp, add, op) => {
                                    //println!("hello {}", init);
                                    if data.output_mode == OutputMode::Stereo {
                                        let pos = (*init + 1.0) * 0.5;
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::LFSaw(
                                                pos, *freq, *eff_phase, *amp, *add, *op,
                                            ),
                                        );
                                    } else {
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::LFSaw(
                                                *init, *freq, *eff_phase, *amp, *add, *op,
                                            ),
                                        );
                                    }
                                }
                                SynthParameterValue::LFTri(init, freq, eff_phase, amp, add, op) => {
                                    //println!("hello {}", init);
                                    if data.output_mode == OutputMode::Stereo {
                                        let pos = (*init + 1.0) * 0.5;
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::LFTri(
                                                pos, *freq, *eff_phase, *amp, *add, *op,
                                            ),
                                        );
                                    } else {
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::LFTri(
                                                *init, *freq, *eff_phase, *amp, *add, *op,
                                            ),
                                        );
                                    }
                                }
                                SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                                    //println!("hello {}", init);
                                    if data.output_mode == OutputMode::Stereo {
                                        let pos = (*init + 1.0) * 0.5;
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::LFSquare(
                                                pos, *freq, *pw, *amp, *add, *op,
                                            ),
                                        );
                                    } else {
                                        inst.set_instance_parameter(
                                            *k,
                                            &SynthParameterValue::LFSquare(
                                                *init, *freq, *pw, *amp, *add, *op,
                                            ),
                                        );
                                    }
                                }
                                _ => {}
                            },
                            // convert milliseconds to seconds
                            SynthParameterLabel::Duration => {
                                if let SynthParameterValue::ScalarF32(val) = v {
                                    inst.set_instance_parameter(
                                        *k,
                                        &SynthParameterValue::ScalarF32(*val * 0.001),
                                    )
                                }
                            }
                            SynthParameterLabel::Attack => {
                                if let SynthParameterValue::ScalarF32(val) = v {
                                    inst.set_instance_parameter(
                                        *k,
                                        &SynthParameterValue::ScalarF32(*val * 0.001),
                                    )
                                }
                            }
                            SynthParameterLabel::Sustain => {
                                if let SynthParameterValue::ScalarF32(val) = v {
                                    inst.set_instance_parameter(
                                        *k,
                                        &SynthParameterValue::ScalarF32(*val * 0.001),
                                    )
                                }
                            }
                            SynthParameterLabel::Release => {
                                if let SynthParameterValue::ScalarF32(val) = v {
                                    inst.set_instance_parameter(
                                        *k,
                                        &SynthParameterValue::ScalarF32(*val * 0.001),
                                    )
                                }
                            }
                            _ => inst.set_instance_parameter(*k, v),
                        }
                    }
                    data.ruffbox.trigger(inst);
                } else {
                    println!("can't prepare instance !");
                }
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
                            data.output_mode,
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
                            Command::FreezeBuffer(freezbuf, inbuf) => {
                                commands::freeze_buffer(&data.ruffbox, freezbuf, inbuf);
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
                            Command::GlobalRuffboxParams(mut m) => {
                                commands::set_global_ruffbox_parameters(&data.ruffbox, &mut m);
                            }
                            Command::Clear => {
                                let session2 = sync::Arc::clone(&data.session);
                                let parts_store2 = sync::Arc::clone(&data.parts_store);
                                thread::spawn(move || {
                                    Session::clear_session(&session2, &parts_store2);
                                    println!("a command (stop session)");
                                });
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

    (time, sync, end_state)
}
// END INNER MAIN SCHEDULER FUNCTION ...

impl<const BUFSIZE: usize, const NCHAN: usize> Session<BUFSIZE, NCHAN> {
    pub fn new() -> Self {
        Session {
            schedulers: HashMap::new(),
            contexts: HashMap::new(),
            visualizer_client: None,
            rec_control: None,
        }
    }

    pub fn handle_context(
        ctx: &mut SyncContext,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
        global_parameters: &sync::Arc<GlobalParameters>,
        output_mode: OutputMode,
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
                    remainders = new_gens.intersection(old_gens).cloned().collect();
                    newcomers = new_gens.difference(old_gens).cloned().collect();
                    quitters = old_gens.difference(&new_gens).cloned().collect();
                }
            }

            println!("newcomers {:?}", newcomers);
            println!("remainders {:?}", remainders);
            println!("quitters {:?}", quitters);

            // HANDLE QUITTERS (generators to be stopped ...)
            // stop asynchronously to keep main thread reactive
            let session2 = sync::Arc::clone(session);
            thread::spawn(move || {
                Session::stop_generators(&session2, &quitters);
            });

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

            // HANDLE NEWCOMERS
            if let Some(ext_sync) = external_sync.clone() {
                // external sync has precedence
                for nc in newcomers.drain(..) {
                    let gen = gen_map.remove(&nc).unwrap();
                    Session::start_generator_push_sync(
                        Box::new(gen),
                        session,
                        &ext_sync,
                        ctx.shift as f64 * 0.001,
                    );
                }
            } else if let Some(int_sync) = internal_sync.clone() {
                for nc in newcomers.drain(..) {
                    let gen = gen_map.remove(&nc).unwrap();
                    Session::start_generator_push_sync(
                        Box::new(gen),
                        session,
                        &int_sync,
                        ctx.shift as f64 * 0.001,
                    );
                }
            } else {
                for nc in newcomers.drain(..) {
                    let gen = gen_map.remove(&nc).unwrap();
                    Session::start_generator_no_sync(
                        Box::new(gen),
                        session,
                        ruffbox,
                        parts_store,
                        global_parameters,
                        output_mode,
                        ctx.shift as f64 * 0.001,
                        &ctx.block_tags,
                        &ctx.solo_tags,
                    );
                }
            }
            // END HANDLE NEWCOMERS

            // HANDLE REMAINDERS
            if let Some(ext_sync) = external_sync.clone() {
                for rem in remainders.drain(..) {
                    let gen = gen_map.remove(&rem).unwrap();
                    Session::resume_generator_sync(
                        Box::new(gen),
                        session,
                        ruffbox,
                        parts_store,
                        &ext_sync,
                        ctx.shift as f64 * 0.001,
                        &ctx.block_tags,
                        &ctx.solo_tags,
                    );
                }
            } else {
                for rem in remainders.drain(..) {
                    let gen = gen_map.remove(&rem).unwrap();
                    Session::resume_generator(
                        Box::new(gen),
                        session,
                        ruffbox,
                        parts_store,
                        global_parameters,
                        output_mode,
                        ctx.shift as f64 * 0.001,
                        &ctx.block_tags,
                        &ctx.solo_tags,
                    );
                }
            } // END HANDLE REMAINDERS

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
                        Session::start_generator_push_sync(
                            Box::new(gen),
                            session,
                            &ext_sync,
                            ctx.shift as f64 * 0.001,
                        );
                    }
                } else if let Some(int_sync) = internal_sync {
                    // this is very unlikely to happen, but just in case ...
                    for (_, gen) in gen_map.drain() {
                        Session::start_generator_push_sync(
                            Box::new(gen),
                            session,
                            &int_sync,
                            ctx.shift as f64 * 0.001,
                        );
                    }
                } else {
                    // common case ...
                    for (_, gen) in gen_map.drain() {
                        Session::start_generator_no_sync(
                            Box::new(gen),
                            session,
                            ruffbox,
                            parts_store,
                            global_parameters,
                            output_mode,
                            ctx.shift as f64 * 0.001,
                            &ctx.block_tags,
                            &ctx.solo_tags,
                        );
                    }
                }
            }

            // insert new context
            {
                let mut sess = session.lock();
                sess.contexts.insert(name, new_gens);
            }
        } else {
            // stop all that were kept in this context, remove context ...
            let an_old_ctx;
            {
                let mut sess = session.lock();
                an_old_ctx = sess.contexts.remove(&name);
            }

            if let Some(old_ctx) = an_old_ctx {
                let old_ctx_vec: Vec<BTreeSet<String>> =
                    old_ctx.difference(&BTreeSet::new()).cloned().collect();
                let session2 = sync::Arc::clone(session);
                thread::spawn(move || {
                    Session::stop_generators(&session2, &old_ctx_vec);
                });
            }
        }
    }

    /// if a generater is already active, it'll be resumed by replacing its scheduler data
    #[allow(clippy::too_many_arguments)]
    fn resume_generator(
        gen: Box<Generator>,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
        global_parameters: &sync::Arc<GlobalParameters>,
        output_mode: OutputMode,
        shift: f64,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) {
        let id_tags = gen.id_tags.clone();

        let mut finished = false;

        {
            // lock release block
            let mut sess = session.lock();
            // start scheduler if it exists ...
            if let Some((_, data)) = sess.schedulers.get_mut(&id_tags) {
                print!("resume generator \'");
                for tag in id_tags.iter() {
                    print!("{} ", tag);
                }
                println!("\'");

                // keep the scheduler running, just replace the data ...
                let sched_data = data.lock();
                finished = sched_data.finished;
            }
        }

        if finished {
            Session::stop_generator(session, &id_tags);
            Session::start_generator_no_sync(
                gen,
                session,
                ruffbox,
                parts_store,
                global_parameters,
                output_mode,
                shift,
                block_tags,
                solo_tags,
            );
            println!("restarted finished gen");
        } else {
            let mut sess = session.lock();
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
                    block_tags,
                    solo_tags,
                );
                println!("replaced sched data");
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_generator_sync(
        gen: Box<Generator>,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
        sync_tags: &BTreeSet<String>,
        shift: f64,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) {
        let id_tags = gen.id_tags.clone();
        // start scheduler if it exists ...

        // thanks, borrow checker, for this elegant construction ...
        let mut sess = session.lock();
        let s_data = if let Some((_, sd)) = sess.schedulers.get(sync_tags) {
            Some(sd.clone())
        } else {
            None
        };

        if let Some((_, data)) = sess.schedulers.get_mut(&id_tags) {
            if let Some(sync_data) = s_data {
                print!("resume sync generator \'");
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
                    block_tags,
                    solo_tags,
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
                    block_tags,
                    solo_tags,
                );
            }
        }
    }

    /// start, sync time data ...
    pub fn start_generator_data_sync(
        gen: Box<Generator>,
        data: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) {
        let id_tags = gen.id_tags.clone();

        print!("start generator (sync time data) \'");
        for tag in id_tags.iter() {
            print!("{} ", tag);
        }
        println!("\'");
        // sync to data
        // create sched data from data
        let sched_data =
            sync::Arc::new(Mutex::new(SchedulerData::<BUFSIZE, NCHAN>::from_time_data(
                data,
                shift,
                gen,
                &data.ruffbox,
                &data.session,
                &data.parts_store,
                block_tags,
                solo_tags,
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
        if let Some((_, data)) = sess.schedulers.get_mut(sync_tags) {
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

    #[allow(clippy::too_many_arguments)]
    pub fn start_generator_no_sync(
        gen: Box<Generator>,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
        global_parameters: &sync::Arc<GlobalParameters>,
        output_mode: OutputMode,
        shift: f64,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
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
            output_mode,
            SyncMode::NotOnSilence,
            block_tags,
            solo_tags,
        )));
        Session::start_scheduler(session, sched_data, id_tags)
    }

    /////////////////////////////////////////////
    // start scheduler and main time recursion //
    /////////////////////////////////////////////
    /// start a scheduler, create scheduler data, etc ...
    fn start_scheduler(
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        sched_data: sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>,
        id_tags: BTreeSet<String>,
    ) {
        // otherwise, create new sched and data ...
        let mut sched = Scheduler::<BUFSIZE, NCHAN>::new();

        // assemble name for thread ...
        let mut thread_name: String = "".to_owned();
        for tag in id_tags.iter() {
            thread_name.push_str(&(format!("{} ", tag)));
        }

        sched.start(thread_name.trim(), eval_loop, sync::Arc::clone(&sched_data));

        // get sched out of map, try to keep lock only shortly ...
        let sched_prox;
        {
            let mut sess = session.lock();
            sched_prox = sess.schedulers.remove(&id_tags);
            sess.schedulers.insert(id_tags.clone(), (sched, sched_data));
        }

        // prepare for replacement
        if let Some((mut sched, _)) = sched_prox {
            thread::spawn(move || {
                sched.stop();
                sched.join();
                print!("replacing generator \'");
                for tag in id_tags.iter() {
                    print!("{} ", tag);
                }
                println!("\'");
            });
        }
    }

    pub fn stop_generator(
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        gen_name: &BTreeSet<String>,
    ) {
        print!("--- stopping generator \'");
        for tag in gen_name.iter() {
            print!("{} ", tag);
        }
        println!("\'");
        // get sched out of map, try to keep lock only shortly ...
        let sched_prox;
        {
            let mut sess = session.lock();
            sched_prox = sess.schedulers.remove(gen_name);
            if let Some(c) = &sess.visualizer_client {
                c.clear(gen_name);
            }
        }

        if let Some((mut sched, data)) = sched_prox {
            sched.stop();
            sched.join();
            print!("stopped/removed generator \'");
            for tag in gen_name.iter() {
                print!("{} ", tag);
            }
            println!("\'");
            let sess = session.lock();
            if let Some(c) = &sess.visualizer_client {
                let d = data.lock();
                for proc in d.generator.processors.iter() {
                    proc.clear_visualization(c);
                }
            }
        }
    }

    pub fn stop_generators(
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        gen_names: &[BTreeSet<String>],
    ) {
        // get scheds out of map, try to keep lock only shortly ...
        let mut sched_proxies = Vec::new();
        {
            let mut sess = session.lock();
            for name in gen_names.iter() {
                sched_proxies.push(sess.schedulers.remove(name));
                if let Some(c) = &sess.visualizer_client {
                    c.clear(name);
                }
            }
        }

        // stop
        let mut sched_proxies2 = Vec::new(); // sometimes rust is really annoying ...
        let mut prox_drain = sched_proxies.drain(..);
        while let Some(Some((mut sched, data))) = prox_drain.next() {
            sched.stop();
            sched_proxies2.push(sched);
            let sess = session.lock();
            if let Some(c) = &sess.visualizer_client {
                let d = data.lock();
                for proc in d.generator.processors.iter() {
                    proc.clear_visualization(c);
                }
            }
        }

        // join
        for mut sched in sched_proxies2.drain(..) {
            sched.join();
        }
    }

    pub fn clear_session(
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
    ) {
        let mut sess = session.lock();
        for (_, (sched, _)) in sess.schedulers.iter_mut() {
            sched.stop();
        }

        for (k, (sched, _)) in sess.schedulers.iter_mut() {
            sched.join();
            print!("stopped/removed generator \'");
            for tag in k.iter() {
                print!("{} ", tag);
            }
            println!("\'");
        }

        if let Some(c) = &sess.visualizer_client {
            for (k, (_, data)) in sess.schedulers.iter() {
                c.clear(k);
                let d = data.lock();
                for proc in d.generator.processors.iter() {
                    proc.clear_visualization(c);
                }
            }
        }

        sess.schedulers = HashMap::new();
        sess.contexts = HashMap::new();
        let mut ps = parts_store.lock();
        *ps = HashMap::new();
    }
}
