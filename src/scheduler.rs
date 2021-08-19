use crate::builtin_types::*;
use crate::generator::Generator;
use crate::session::{OutputMode, Session, SyncMode};
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::Ruffbox;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::{sync, thread};

/// A simple time-recursion event scheduler running at a fixed time interval.
pub struct Scheduler<const BUFSIZE: usize, const NCHAN: usize> {
    pub handle: Option<thread::JoinHandle<()>>,
    pub running: sync::Arc<AtomicBool>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> Default for Scheduler<BUFSIZE, NCHAN> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SchedulerData<const BUFSIZE: usize, const NCHAN: usize> {
    pub start_time: Instant,
    pub stream_time: f64,
    pub logical_time: f64,
    pub last_diff: f64,
    pub shift: f64,
    pub generator: Box<Generator>,
    pub synced_generators: Vec<(Box<Generator>, f64)>,
    pub ruffbox: sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
    pub session: sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    pub parts_store: sync::Arc<Mutex<PartsStore>>,
    pub global_parameters: sync::Arc<GlobalParameters>,
    pub output_mode: OutputMode,
    pub sync_mode: SyncMode,
}

impl<const BUFSIZE: usize, const NCHAN: usize> SchedulerData<BUFSIZE, NCHAN> {
    pub fn from_previous(
        old: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
        mut data: Box<Generator>,
        ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
    ) -> Self {
        let shift_diff = shift - old.shift;
        data.transfer_state(&old.generator);
        // keep scheduling, retain data
        SchedulerData {
            start_time: old.start_time,
            stream_time: old.stream_time + shift_diff,
            logical_time: old.logical_time + shift_diff,
            shift,
            last_diff: old.last_diff,
            generator: data,
            synced_generators: old.synced_generators.clone(), // carry over synced gens ...
            ruffbox: sync::Arc::clone(ruffbox),
            session: sync::Arc::clone(session),
            parts_store: sync::Arc::clone(parts_store),
            global_parameters: sync::Arc::clone(&old.global_parameters),
            output_mode: old.output_mode,
            sync_mode: old.sync_mode,
        }
    }

    pub fn from_time_data(
        old: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
        data: Box<Generator>,
        ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
    ) -> Self {
        let shift_diff = shift - old.shift;

        // keep scheduling, retain time
        SchedulerData {
            start_time: old.start_time,
            stream_time: old.stream_time + shift_diff,
            logical_time: old.logical_time + shift_diff,
            shift,
            last_diff: 0.0,
            generator: data,
            synced_generators: Vec::new(),
            ruffbox: sync::Arc::clone(ruffbox),
            session: sync::Arc::clone(session),
            parts_store: sync::Arc::clone(parts_store),
            global_parameters: sync::Arc::clone(&old.global_parameters),
            output_mode: old.output_mode,
            sync_mode: old.sync_mode,
        }
    }

    pub fn from_data(
        data: Box<Generator>,
        shift: f64,
        session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
        ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
        parts_store: &sync::Arc<Mutex<PartsStore>>,
        global_parameters: &sync::Arc<GlobalParameters>,
        output_mode: OutputMode,
        sync_mode: SyncMode,
    ) -> Self {
        // get logical time since start from ruffbox
        let stream_time;
        {
            let ruff = ruffbox.lock();
            stream_time = ruff.get_now();
        }
        SchedulerData {
            start_time: Instant::now(),
            stream_time: stream_time + shift,
            logical_time: shift,
            last_diff: 0.0,
            shift,
            generator: data,
            synced_generators: Vec::new(),
            ruffbox: sync::Arc::clone(ruffbox),
            session: sync::Arc::clone(session),
            parts_store: sync::Arc::clone(parts_store),
            global_parameters: sync::Arc::clone(global_parameters),
            output_mode: output_mode,
            sync_mode: sync_mode,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Scheduler<BUFSIZE, NCHAN> {
    pub fn new() -> Self {
        Scheduler {
            handle: None,
            running: sync::Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start this scheduler.
    pub fn start(
        &mut self,
        name: &str,
        fun: fn(&mut SchedulerData<BUFSIZE, NCHAN>) -> (f64, bool),
        data: sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>,
    ) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let builder = thread::Builder::new().name(name.into());

        self.handle = Some(
            builder
                .spawn(move || {
                    while running.load(Ordering::SeqCst) {
                        let next: f64;
                        let ldif: f64;
                        let cur: f64;
                        {
                            let mut sched_data = data.lock();
                            // call event processing function that'll return
                            // the sync flag and
                            let sched_result = (fun)(&mut sched_data);
                            let sync = sched_result.1;
                            if sync {
                                let mut syncs = sched_data.synced_generators.clone();
                                for (g, s) in syncs.drain(..) {
                                    Session::start_generator_no_sync(
                                        g,
                                        &sched_data.session,
                                        &sched_data.ruffbox,
                                        &sched_data.parts_store,
                                        &sched_data.global_parameters,
                                        sched_data.output_mode,
                                        s,
                                    );
                                }
                                sched_data.synced_generators.clear();
                            }

			    cur = sched_data.start_time.elapsed().as_secs_f64();
                            sched_data.last_diff = cur - sched_data.logical_time;
                            next = sched_result.0;
                            ldif = sched_data.last_diff;
                            // compensate for eventual lateness ...
                            if (next - ldif) < 0.0 {
                                let handle = thread::current();
				println!(
                                    "{} negative duration found: cur before {} cur after {} {} {} {}",
                                    handle.name().unwrap(),
                                    cur,
				    sched_data.start_time.elapsed().as_secs_f64(),
                                    sched_data.logical_time,
                                    next,
                                    ldif
                                );
                            }
                            sched_data.logical_time += next;
                            sched_data.stream_time += next;
                        }

                        thread::sleep(Duration::from_secs_f64(next - ldif));
                    }
                })
                .unwrap(),
        );
    }

    /// Stop this scheduler.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Join this scheduler thread.
    pub fn join(&mut self) {
        if let Some(h) = self.handle.take() {
            if let Ok(_) = h.join() {
                println!("joined!");
            } else {
                println!("Could not join spawned thread");
            }
        } else {
            println!("Called stop on non-running thread");
        }
    }
}
