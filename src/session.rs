use dashmap::DashMap;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use std::collections::{BTreeSet, HashMap};
use std::{sync, thread};

use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
use ruffbox_synth::ruffbox::RuffboxControls;

use crate::builtin_types::{Command, ConfigParameter, GlobalVariables, VariableId};
use crate::commands;
use crate::event::InterpretableEvent;
use crate::event_helpers::*;
use crate::generator::Generator;
use crate::osc_client::OscClient;
use crate::parameter::*;
use crate::real_time_streaming;
use crate::scheduler::{Scheduler, SchedulerData};
use crate::SampleAndWavematrixSet;
use crate::TypedEntity;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Stereo,
    // AmbisonicsBinaural,
    // Ambisonics
    FourChannel,
    EightChannel,
    SixteenChannel,
    //TwentyFourChannel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub shift: i32,
    pub block_tags: BTreeSet<String>,
    pub solo_tags: BTreeSet<String>,
}

#[derive(Clone)]
pub struct Session<const BUFSIZE: usize, const NCHAN: usize> {
    pub sample_set: SampleAndWavematrixSet,
    pub output_mode: OutputMode,
    pub sync_mode: SyncMode,
    pub ruffbox: sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    pub globals: sync::Arc<GlobalVariables>,

    pub schedulers: sync::Arc<
        DashMap<BTreeSet<String>, (Scheduler<BUFSIZE, NCHAN>, SchedulerData<BUFSIZE, NCHAN>)>,
    >,
    pub contexts: sync::Arc<DashMap<String, BTreeSet<BTreeSet<String>>>>,
    pub osc_client: OscClient,
    pub rec_control:
        sync::Arc<Mutex<Option<real_time_streaming::RecordingControl<BUFSIZE, NCHAN>>>>,
}

//////////////////////////////////////
// THE MAIN TIME RECURSION LOOP!!!  //
//////////////////////////////////////

// yes, here it is ... the evaluation function ...
// or better, the inside part of the time iteration
fn eval_loop<const BUFSIZE: usize, const NCHAN: usize>(
    data: &mut SchedulerData<BUFSIZE, NCHAN>,
    session: &Session<BUFSIZE, NCHAN>,
) -> (f64, bool, bool) {
    // global tempo modifier, allows us to do weird stuff with the
    // global tempo ...
    let mut tmod: f64 = 1.0;
    let mut latency: f64 = 0.05;

    if let TypedEntity::ConfigParameter(ConfigParameter::Dynamic(global_tmod)) = session
        .globals
        .entry(VariableId::GlobalTimeModifier) // fixed variable ID
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Dynamic(
            DynVal::with_value(1.0),
        ))) // init on first attempt
        .value_mut()
    {
        tmod = global_tmod.evaluate_numerical() as f64;
    }

    if let TypedEntity::ConfigParameter(ConfigParameter::Dynamic(global_latency)) = session
        .globals
        .entry(VariableId::GlobalLatency)
        .or_insert(TypedEntity::ConfigParameter(ConfigParameter::Dynamic(
            DynVal::with_value(0.05),
        ))) // init on first attempt
        .value_mut()
    {
        latency = global_latency.evaluate_numerical() as f64;
    }

    // GENERATOR LOCK !!!
    let (time, mut events, end_state) = {
        let mut gen = data.generator.lock();

        {
            if let Some(cli) = session.osc_client.vis.try_read() {
                if let Some(ref vc) = *cli {
                    if gen.root_generator.is_modified() {
                        vc.create_or_update(&gen);
                        gen.root_generator.clear_modified()
                    }
                    vc.update_active_node(&gen);
                    for (_, proc) in gen.processors.iter_mut() {
                        proc.visualize_if_possible(&vc);
                    }
                }
            }
        }

        let time = if let SynthParameterValue::ScalarF32(t) =
            gen.current_transition(&session.globals).params[&SynthParameterLabel::Duration]
        {
            (t * 0.001) as f64 * tmod
        } else {
            0.2
        };

        // retrieve the current events
        let events = gen.current_events(&session.globals);
        //if events.is_empty() {
        //    println!("really no events");
        //}
        let end_state = gen.reached_end_state();
        (time, events, end_state)
    };
    // the sync flag will be returned alongside the
    // time to let the scheduler know that it should
    // trigger the synced generators
    let mut sync = false;

    // start the generators ready to be synced ...
    if session.sync_mode == SyncMode::All {
        //println!("sync all");
        sync = true;
    }

    for ev in events.iter_mut() {
        match ev {
            InterpretableEvent::Sound(s) => {
                // no need to allocate a string everytime here, should be changed
                if s.name == "silence" {
                    // start the generators ready to be synced ...
                    if session.sync_mode == SyncMode::OnlyOnSilence {
                        //println!("sync silence");
                        sync = true;
                    }
                    continue;
                }

                // start the generators ready to be synced ...
                if session.sync_mode == SyncMode::NotOnSilence {
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

                // if this is a sampler event and contains a sample lookup,
                // resolve it NOW ... at the very end, finally ...
                let mut bufnum: usize = 0;
                if let Some(lookup) = s.sample_lookup.as_ref() {
                    if let Some((res_bufnum, duration)) = session.sample_set.resolve_lookup(lookup)
                    {
                        bufnum = res_bufnum;
                        // is this really needed ??
                        s.params.insert(
                            SynthParameterLabel::SampleBufferNumber,
                            SynthParameterValue::ScalarUsize(bufnum),
                        );

                        s.params
                            .entry(SynthParameterLabel::Sustain)
                            .or_insert_with(|| {
                                SynthParameterValue::ScalarF32((duration - 2) as f32)
                            });
                    }
                }

                // prepare a single, self-contained envelope from
                // the available information ...
                s.build_envelope();

                if let Some(mut inst) = session.ruffbox.prepare_instance(
                    map_synth_type(&s.name, &s.params),
                    data.stream_time.load() + latency,
                    bufnum,
                ) {
                    // set parameters and trigger instance
                    for (k, v) in s.params.iter() {
                        // special handling for stereo param
                        match k {
                            SynthParameterLabel::ChannelPosition => {
                                if session.output_mode == OutputMode::Stereo {
                                    inst.set_instance_parameter(*k, &translate_stereo(v.clone()));
                                } else {
                                    inst.set_instance_parameter(*k, v);
                                }
                            }
                            // convert milliseconds to seconds
                            SynthParameterLabel::Duration => {
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
                    session.ruffbox.trigger(inst);
                } else {
                    println!("can't prepare instance !");
                }
            }
            InterpretableEvent::Control(c) => {
                if let Some(mut contexts) = c.ctx.clone() {
                    // this is the worst clone ....
                    for mut sx in contexts.drain(..) {
                        Session::handle_context(&mut sx, session);
                    }
                }
                if let Some(mut commands) = c.cmd.clone() {
                    // this is the worst clone ....
                    for c in commands.drain(..) {
                        match c {
                            Command::FreezeBuffer(freezbuf, inbuf) => {
                                commands::freeze_buffer(&session.ruffbox, freezbuf, inbuf);
                                println!("freeze buffer");
                            }
                            Command::Tmod(p) => {
                                commands::set_global_tmod(&session.globals, p);
                            }
                            Command::GlobRes(v) => {
                                commands::set_global_lifemodel_resources(&session.globals, v);
                            }
                            Command::GlobalRuffboxParams(mut m) => {
                                commands::set_global_ruffbox_parameters(
                                    &session.ruffbox,
                                    &session.globals,
                                    &mut m,
                                );
                            }
                            Command::Clear => {
                                let session2 = session.clone();
                                thread::spawn(move || {
                                    Session::clear_session(session2);
                                    println!("a command (stop session)");
                                });
                            }
                            Command::Once(mut s, c) => {
                                //println!("handle once from gen");
                                commands::once(session, &mut s, &c);
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
    pub fn handle_context(ctx: &mut SyncContext, session: &Session<BUFSIZE, NCHAN>) {
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

            if let Some(old_gens) = session.contexts.get(&name) {
                // this means context is running
                remainders = new_gens.intersection(&old_gens).cloned().collect();
                newcomers = new_gens.difference(&old_gens).cloned().collect();
                quitters = old_gens.difference(&new_gens).cloned().collect();
            }

            println!("newcomers {newcomers:?}");
            println!("remainders {remainders:?}");
            println!("quitters {quitters:?}");

            // HANDLE QUITTERS (generators to be stopped ...)
            // stop asynchronously to keep main thread reactive
            let session2 = session.clone();
            thread::spawn(move || {
                Session::stop_generators(session2, &quitters);
            });

            // EXTERNAL SYNC
            // are we supposed to sync to some other context ??
            // get external sync ...
            let external_sync = if let Some(sync) = &ctx.sync_to {
                let mut smallest_id = None;

                if let Some(sync_gens) = session.contexts.get(sync) {
                    let mut last_len: usize = usize::MAX;
                    for tags in sync_gens.iter() {
                        if tags.len() < last_len {
                            last_len = tags.len();
                            smallest_id = Some(tags.clone());
                        }
                    }
                };

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
                        gen,
                        session,
                        &ext_sync,
                        ctx.shift as f64 * 0.001,
                    );
                }
            } else if let Some(int_sync) = internal_sync.clone() {
                for nc in newcomers.drain(..) {
                    let gen = gen_map.remove(&nc).unwrap();
                    Session::start_generator_push_sync(
                        gen,
                        session,
                        &int_sync,
                        ctx.shift as f64 * 0.001,
                    );
                }
            } else {
                for nc in newcomers.drain(..) {
                    let gen = gen_map.remove(&nc).unwrap();
                    Session::start_generator_no_sync(
                        gen,
                        session,
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
                        gen,
                        session,
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
                        gen,
                        session,
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
                            gen,
                            session,
                            &ext_sync,
                            ctx.shift as f64 * 0.001,
                        );
                    }
                } else if let Some(int_sync) = internal_sync {
                    // this is very unlikely to happen, but just in case ...
                    for (_, gen) in gen_map.drain() {
                        Session::start_generator_push_sync(
                            gen,
                            session,
                            &int_sync,
                            ctx.shift as f64 * 0.001,
                        );
                    }
                } else {
                    // common case ...
                    for (_, gen) in gen_map.drain() {
                        Session::start_generator_no_sync(
                            gen,
                            session,
                            ctx.shift as f64 * 0.001,
                            &ctx.block_tags,
                            &ctx.solo_tags,
                        );
                    }
                }
            }

            // insert new context
            session.contexts.insert(name, new_gens);
        } else {
            // stop all that were kept in this context, remove context ...

            let an_old_ctx = if let Some((_, v)) = session.contexts.remove(&name) {
                Some(v)
            } else {
                None
            };

            if let Some(old_ctx) = an_old_ctx {
                let old_ctx_vec: Vec<BTreeSet<String>> =
                    old_ctx.difference(&BTreeSet::new()).cloned().collect();
                let session2 = session.clone();
                thread::spawn(move || {
                    Session::stop_generators(session2, &old_ctx_vec);
                });
            }
        }
    }

    /// if a generater is already active, it'll be resumed by replacing its scheduler data
    fn resume_generator(
        gen: Generator,
        session: &Session<BUFSIZE, NCHAN>,
        shift: f64,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) {
        let id_tags = gen.id_tags.clone();

        let mut finished = false;

        // start scheduler if it exists ...
        if let Some(v) = session.schedulers.get_mut(&id_tags) {
            let (_, data) = v.value();
            print!("resume generator \'");
            for tag in id_tags.iter() {
                print!("{tag} ");
            }
            println!("\'");

            // keep the scheduler running, just replace the data ...
            finished = data.finished.load(sync::atomic::Ordering::SeqCst);
        };

        if finished {
            Session::stop_generator(session, &id_tags);
            Session::start_generator_no_sync(gen, session, shift, block_tags, solo_tags);
            println!("restarted finished gen");
        } else {
            // start scheduler if it exists ...
            if let Some(mut v) = session.schedulers.get_mut(&id_tags) {
                let (_, data) = v.value_mut();
                print!("resume generator \'");
                for tag in id_tags.iter() {
                    print!("{tag} ");
                }
                println!("\'");
                // keep the scheduler running, just replace the data ...
                *data.generator.lock() = gen;
                data.block_tags = block_tags.clone();
                data.solo_tags = block_tags.clone();

                println!("replaced sched data");
            }
        }
    }

    fn resume_generator_sync(
        gen: Generator,
        session: &Session<BUFSIZE, NCHAN>,
        sync_tags: &BTreeSet<String>,
        shift: f64,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) {
        let id_tags = gen.id_tags.clone();
        // start scheduler if it exists ...

        // thanks, borrow checker, for this elegant construction ...
        let s_data = if let Some(v) = session.schedulers.get_mut(sync_tags) {
            let (_, sd) = v.value();
            Some(sd.clone())
        } else {
            None
        };

        if let Some(mut v) = session.schedulers.get_mut(&id_tags) {
            let (_, data) = v.value_mut();
            if let Some(sync_data) = s_data {
                print!("resume sync generator \'");
                for tag in id_tags.iter() {
                    print!("{tag} ");
                }
                println!("\'");

                // update the scheduler data,
                // re-sync
                data.update_sync(
                    &sync_data,
                    shift,
                    gen,
                    block_tags.clone(),
                    solo_tags.clone(),
                );
            } else {
                // resume sync: later ...
                print!("resume generator \'");
                for tag in id_tags.iter() {
                    print!("{tag} ");
                }
                println!("\'");
                // update scheduler data ...
                data.update(shift, gen, block_tags.clone(), solo_tags.clone());
            }
        }
    }

    /// start, sync time data ...
    pub fn start_generator_data_sync(
        gen: Generator,
        session: &Session<BUFSIZE, NCHAN>,
        data: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) {
        let id_tags = gen.id_tags.clone();

        print!("start generator (sync time data) \'");
        for tag in id_tags.iter() {
            print!("{tag} ");
        }
        println!("\'");
        // sync to data
        // create sched data from data
        let sched_data =
            SchedulerData::<BUFSIZE, NCHAN>::new_sync(data, shift, gen, block_tags, solo_tags);
        Session::start_scheduler(session, sched_data, id_tags)
    }

    // push to synced gen's sync list ...
    // if it doesn't exist, just start ...
    fn start_generator_push_sync(
        gen: Generator,
        session: &Session<BUFSIZE, NCHAN>,
        sync_tags: &BTreeSet<String>,
        shift: f64,
    ) {
        //this is prob kinda redundant
        if let Some(v) = session.schedulers.get_mut(sync_tags) {
            let (_, data) = v.value();

            print!("start generator \'");
            for tag in gen.id_tags.iter() {
                print!("{tag} ");
            }
            print!("\' (push sync to existing \'");
            for tag in sync_tags.iter() {
                print!("{tag} ");
            }
            println!("\')");

            // push to sync ...
            data.synced_generators.lock().push((gen, shift));
        }
    }

    pub fn start_generator_no_sync(
        gen: Generator,
        session: &Session<BUFSIZE, NCHAN>,
        shift: f64,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) {
        let id_tags = gen.id_tags.clone();

        print!("start generator (no sync) \'");
        for tag in id_tags.iter() {
            print!("{tag} ");
        }
        println!("\'");

        let sched_data = SchedulerData::<BUFSIZE, NCHAN>::new(
            gen,
            shift,
            session.ruffbox.get_now(),
            block_tags,
            solo_tags,
        );
        Session::start_scheduler(session, sched_data, id_tags)
    }

    /////////////////////////////////////////////
    // start scheduler and main time recursion //
    /////////////////////////////////////////////
    /// start a scheduler, create scheduler data, etc ...
    #[allow(clippy::format_push_string)]
    fn start_scheduler(
        session: &Session<BUFSIZE, NCHAN>,
        sched_data: SchedulerData<BUFSIZE, NCHAN>,
        id_tags: BTreeSet<String>,
    ) {
        // otherwise, create new sched and data ...
        let mut sched = Scheduler::<BUFSIZE, NCHAN>::new();

        // assemble name for thread ...
        let mut thread_name: String = "".to_owned();
        for tag in id_tags.iter() {
            thread_name.push_str(&(format!("{tag} ")));
        }

        sched.start(
            thread_name.trim(),
            eval_loop,
            sched_data.clone(),
            session.clone(),
        );

        // get sched out of map, try to keep lock only shortly ...
        let sched_prox = if let Some((_, v)) = session.schedulers.remove(&id_tags) {
            Some(v)
        } else {
            None
        };

        session
            .schedulers
            .insert(id_tags.clone(), (sched, sched_data));

        // prepare for replacement
        if let Some((mut sched, _)) = sched_prox {
            thread::spawn(move || {
                sched.stop();
                sched.join();
                print!("replacing generator \'");
                for tag in id_tags.iter() {
                    print!("{tag} ");
                }
                println!("\'");
            });
        }
    }

    pub fn stop_generator(session: &Session<BUFSIZE, NCHAN>, gen_name: &BTreeSet<String>) {
        print!("--- stopping generator \'");
        for tag in gen_name.iter() {
            print!("{tag} ");
        }
        println!("\'");
        // get sched out of map, try to keep lock only shortly ...
        let sched_prox;
        {
            let sess = session;

            if let Some((_, v)) = sess.schedulers.remove(gen_name) {
                sched_prox = Some(v);
            } else {
                sched_prox = None;
            }

            {
                if let Some(cli) = session.osc_client.vis.try_read() {
                    if let Some(ref vc) = *cli {
                        vc.clear(gen_name);
                    }
                }
            }
        }

        if let Some((mut sched, data)) = sched_prox {
            sched.stop();
            sched.join();
            print!("stopped/removed generator \'");
            for tag in gen_name.iter() {
                print!("{tag} ");
            }
            println!("\'");
            {
                if let Some(cli) = session.osc_client.vis.try_read() {
                    if let Some(ref vc) = *cli {
                        for (_, proc) in data.generator.lock().processors.iter() {
                            proc.clear_visualization(&vc);
                        }
                    }
                }
            }
        }
    }

    pub fn stop_generators(session: Session<BUFSIZE, NCHAN>, gen_names: &[BTreeSet<String>]) {
        // get scheds out of map, try to keep lock only shortly ...
        let mut sched_proxies = Vec::new();

        for name in gen_names.iter() {
            if let Some((_, v)) = session.schedulers.remove(name) {
                sched_proxies.push(v);
            }

            {
                if let Some(cli) = session.osc_client.vis.try_read() {
                    if let Some(ref vc) = *cli {
                        vc.clear(name);
                    }
                }
            }
        }

        // stop
        let mut sched_proxies2 = Vec::new(); // sometimes rust is really annoying ...
        for (mut sched, data) in sched_proxies.drain(..) {
            sched.stop();
            sched_proxies2.push(sched);
            {
                if let Some(cli) = session.osc_client.vis.try_read() {
                    if let Some(ref vc) = *cli {
                        for (_, proc) in data.generator.lock().processors.iter() {
                            proc.clear_visualization(vc);
                        }
                    }
                }
            }
        }

        // join
        for mut sched in sched_proxies2.drain(..) {
            sched.join();
        }
    }

    pub fn clear_session(session: Session<BUFSIZE, NCHAN>) {
        for mut sc in session.schedulers.iter_mut() {
            let (sched, _) = sc.value_mut();
            sched.stop();
        }

        for mut sc in session.schedulers.iter_mut() {
            let (k, (sched, _)) = sc.pair_mut();
            sched.join();
            print!("stopped/removed generator \'");
            for tag in k.iter() {
                print!("{tag} ");
            }
            println!("\'");
        }

        {
            if let Some(cli) = session.osc_client.vis.try_read() {
                if let Some(ref vc) = *cli {
                    for sc in session.schedulers.iter() {
                        let (k, (_, data)) = sc.pair();
                        vc.clear(k);
                        for (_, proc) in data.generator.lock().processors.iter() {
                            proc.clear_visualization(&vc);
                        }
                    }
                }
            }
        }

        session.schedulers.clear();
        session.contexts.clear();
    }
}
