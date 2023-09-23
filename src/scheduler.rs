use crate::builtin_types::*;
use crate::generator::Generator;
use crate::osc_client::OscClient;
use crate::session::{OutputMode, Session, SyncMode};
use crossbeam::atomic::AtomicCell;
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::RuffboxControls;

use crate::SampleAndWavematrixSet;
use std::collections::BTreeSet;
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

#[derive(Clone)]
pub struct SchedulerData<const BUFSIZE: usize, const NCHAN: usize> {
    pub start_time: std::sync::Arc<Mutex<Instant>>,
    pub stream_time: std::sync::Arc<AtomicCell<f64>>,
    pub logical_time: std::sync::Arc<AtomicCell<f64>>,
    pub last_diff: std::sync::Arc<AtomicCell<f64>>,
    pub shift: std::sync::Arc<AtomicCell<f64>>,
    pub generator: std::sync::Arc<Mutex<Generator>>,
    pub finished: std::sync::Arc<AtomicBool>,
    pub synced_generators: std::sync::Arc<Mutex<Vec<(Generator, f64)>>>,

    pub block_tags: BTreeSet<String>,
    pub solo_tags: BTreeSet<String>,
    // the osc client reverence here might be a bit
    // redundant, as there's already a reference in the
    // session, but that way we can access the client without
    // having to lock the session
    // ONCE THE SCHED DATA IS LOCKFREE ALL OF THE BELOW CAN BE MOVED TO SESSION NOW ...
    pub osc_client: OscClient,
    pub sample_set: SampleAndWavematrixSet,
    pub output_mode: OutputMode,
    pub sync_mode: SyncMode,
    pub ruffbox: sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    pub globals: sync::Arc<GlobalVariables>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> SchedulerData<BUFSIZE, NCHAN> {
    #[allow(clippy::manual_map)]
    #[allow(clippy::too_many_arguments)]
    pub fn from_previous(
        old: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
        mut data: Generator,
        ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        globals: &sync::Arc<GlobalVariables>,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) -> Self {
        let shift_diff = shift - old.shift.load();

        println!("{:?} {}", data.id_tags, data.keep_root);

        if !data.keep_root {
            data.transfer_state(&old.generator.lock());
        } else {
            data.root_generator = old.generator.lock().root_generator.clone();
        }

        SchedulerData {
            start_time: old.start_time.clone(),
            stream_time: sync::Arc::new(AtomicCell::new(old.stream_time.load() + shift_diff)),
            logical_time: sync::Arc::new(AtomicCell::new(old.logical_time.load() + shift_diff)),
            shift: sync::Arc::new(AtomicCell::new(shift)),
            last_diff: old.last_diff.clone(),
            generator: sync::Arc::new(Mutex::new(data)),
            finished: sync::Arc::new(AtomicBool::new(false)),
            synced_generators: old.synced_generators.clone(), // carry over synced gens ...
            ruffbox: sync::Arc::clone(ruffbox),
            globals: sync::Arc::clone(globals),
            osc_client: old.osc_client.clone(),
            output_mode: old.output_mode,
            sample_set: old.sample_set.clone(),
            sync_mode: old.sync_mode,
            block_tags: block_tags.clone(),
            solo_tags: solo_tags.clone(),
        }
    }

    #[allow(clippy::manual_map)]
    #[allow(clippy::too_many_arguments)]
    pub fn from_time_data(
        old: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
        data: Generator,
        ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        globals: &sync::Arc<GlobalVariables>,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) -> Self {
        let shift_diff = shift - old.shift.load();

        // keep scheduling, retain time
        SchedulerData {
            start_time: old.start_time.clone(),
            stream_time: sync::Arc::new(AtomicCell::new(old.stream_time.load() + shift_diff)),
            logical_time: sync::Arc::new(AtomicCell::new(old.logical_time.load() + shift_diff)),
            shift: sync::Arc::new(AtomicCell::new(shift)),
            last_diff: sync::Arc::new(AtomicCell::new(0.0)),
            generator: sync::Arc::new(Mutex::new(data)),
            finished: sync::Arc::new(AtomicBool::new(false)),
            synced_generators: sync::Arc::new(Mutex::new(Vec::new())),
            ruffbox: sync::Arc::clone(ruffbox),
            globals: sync::Arc::clone(globals),
            osc_client: old.osc_client.clone(),
            sample_set: old.sample_set.clone(),
            output_mode: old.output_mode,
            sync_mode: old.sync_mode,
            block_tags: block_tags.clone(),
            solo_tags: solo_tags.clone(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::manual_map)]
    pub fn from_data(
        data: Generator,
        shift: f64,
        osc_client: OscClient,
        ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
        globals: &sync::Arc<GlobalVariables>,
        sample_set: SampleAndWavematrixSet,
        output_mode: OutputMode,
        sync_mode: SyncMode,
        block_tags: &BTreeSet<String>,
        solo_tags: &BTreeSet<String>,
    ) -> Self {
        // get logical time since start from ruffbox
        let stream_time = ruffbox.get_now();

        SchedulerData {
            start_time: sync::Arc::new(Mutex::new(Instant::now())),
            stream_time: sync::Arc::new(AtomicCell::new(stream_time + shift)),
            logical_time: sync::Arc::new(AtomicCell::new(shift)),
            last_diff: sync::Arc::new(AtomicCell::new(0.0)),
            shift: sync::Arc::new(AtomicCell::new(shift)),
            generator: sync::Arc::new(Mutex::new(data)),
            finished: sync::Arc::new(AtomicBool::new(false)),
            synced_generators: sync::Arc::new(Mutex::new(Vec::new())),
            ruffbox: sync::Arc::clone(ruffbox),
            globals: sync::Arc::clone(globals),
            osc_client,
            sample_set,
            output_mode,
            sync_mode,
            block_tags: block_tags.clone(),
            solo_tags: solo_tags.clone(),
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
        fun: fn(&mut SchedulerData<BUFSIZE, NCHAN>, &Session<BUFSIZE, NCHAN>) -> (f64, bool, bool),
        mut sched_data: SchedulerData<BUFSIZE, NCHAN>,
        session: Session<BUFSIZE, NCHAN>,
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
                            // call event processing function that'll return
                            // the sync flag and
                            let sched_result = (fun)(&mut sched_data, &session);
                            let sync = sched_result.1;
			    let end = sched_result.2;
                            if sync {
                                let mut syncs = sched_data.synced_generators.lock();
                                for (g, s) in syncs.drain(..) {
                                    Session::start_generator_data_sync(
                                        g,
					&session,
                                        &sched_data,
                                        s,
					&sched_data.block_tags,
					&sched_data.solo_tags,
                                    );
				}
			    }
			    if end {
				sched_data.finished.store(true, Ordering::SeqCst);
				running.store(false, Ordering::SeqCst);
				return;
			    }
			    cur = sched_data.start_time.lock().elapsed().as_secs_f64();
                            sched_data.last_diff.store(cur - sched_data.logical_time.load());
                            next = sched_result.0;
                            ldif = sched_data.last_diff.load();
                            // compensate for eventual lateness ...
                            if (next - ldif) < 0.0 {
                                let handle = thread::current();
				println!(
                                    "{} negative duration found: cur before {} cur after {} {} {} {}",
                                    handle.name().unwrap(),
                                    cur,
				    sched_data.start_time.lock().elapsed().as_secs_f64(),
                                    sched_data.logical_time.load(),
                                    next,
                                    ldif
                                );
				sched_data.finished.store(true, Ordering::SeqCst);
				running.store(false, Ordering::SeqCst);
				return;
                            }
                            sched_data.logical_time.store(sched_data.logical_time.load() + next);
                            sched_data.stream_time.store(sched_data.stream_time.load() + next);
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
            if h.join().is_ok() {
                println!("joined!");
            } else {
                println!("Could not join spawned thread");
            }
        } else {
            println!("Called stop on non-running thread");
        }
    }
}
