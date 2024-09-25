use crate::generator::Generator;
use crate::session::Session;
use crossbeam::atomic::AtomicCell;
use dashmap::DashSet;
use parking_lot::Mutex;

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

    pub block_tags: std::sync::Arc<DashSet<String>>,
    pub solo_tags: std::sync::Arc<DashSet<String>>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> SchedulerData<BUFSIZE, NCHAN> {
    /// update this scheduler data with a new generator and shift
    /// adjustments
    pub fn update(
        &mut self,
        shift: f64,
        mut data: Generator,
        block_tags: BTreeSet<String>,
        solo_tags: BTreeSet<String>,
    ) {
        if !data.keep_root {
            data.transfer_state(&self.generator.lock());
        } else {
            data.root_generator = self.generator.lock().root_generator.clone();
        }

        let shift_diff = shift - self.shift.load();
        self.stream_time.store(self.stream_time.load() + shift_diff);
        self.logical_time
            .store(self.logical_time.load() + shift_diff);
        self.shift.store(shift);
        *self.generator.lock() = data;
        self.finished.store(false, Ordering::SeqCst);

        self.block_tags.clear();
        for tag in block_tags {
            self.block_tags.insert(tag);
        }

        self.solo_tags.clear();
        for tag in solo_tags {
            self.solo_tags.insert(tag);
        }
    }

    /// new scheduler data, synchronized to another scheduler
    pub fn new_sync(
        old: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
        data: Generator,
        block_tags: &sync::Arc<DashSet<String>>,
        solo_tags: &sync::Arc<DashSet<String>>,
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
            block_tags: sync::Arc::clone(block_tags),
            solo_tags: sync::Arc::clone(solo_tags),
        }
    }

    /// update this scheduler data and synchronize with another scheduler ...
    pub fn update_sync(
        &mut self,
        old: &SchedulerData<BUFSIZE, NCHAN>,
        shift: f64,
        data: Generator,
        block_tags: BTreeSet<String>,
        solo_tags: BTreeSet<String>,
    ) {
        let shift_diff = shift - old.shift.load();
        self.start_time = old.start_time.clone();
        self.stream_time.store(old.stream_time.load() + shift_diff);
        self.logical_time
            .store(old.logical_time.load() + shift_diff);
        self.shift.store(shift);
        self.last_diff.store(0.0);
        *self.generator.lock() = data;
        self.finished.store(false, Ordering::SeqCst);

        self.block_tags.clear();
        for tag in block_tags {
            self.block_tags.insert(tag);
        }

        self.solo_tags.clear();
        for tag in solo_tags {
            self.solo_tags.insert(tag);
        }
    }

    /// create fresh data for a scheduler ...
    pub fn new(
        data: Generator,
        shift: f64,
        stream_time: f64,
        block_tags_in: BTreeSet<String>,
        solo_tags_in: BTreeSet<String>,
    ) -> Self {
        let block_tags = DashSet::new();
        for tag in block_tags_in {
            block_tags.insert(tag);
        }

        let solo_tags = DashSet::new();
        for tag in solo_tags_in {
            solo_tags.insert(tag);
        }

        SchedulerData {
            start_time: sync::Arc::new(Mutex::new(Instant::now())),
            stream_time: sync::Arc::new(AtomicCell::new(stream_time + shift)),
            logical_time: sync::Arc::new(AtomicCell::new(shift)),
            last_diff: sync::Arc::new(AtomicCell::new(0.0)),
            shift: sync::Arc::new(AtomicCell::new(shift)),
            generator: sync::Arc::new(Mutex::new(data)),
            finished: sync::Arc::new(AtomicBool::new(false)),
            synced_generators: sync::Arc::new(Mutex::new(Vec::new())),
            block_tags: sync::Arc::new(block_tags),
            solo_tags: sync::Arc::new(solo_tags),
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
				    let gen_shift = g.time_shift;
                                    Session::start_generator_data_sync(
                                        g,
					&session,
                                        &sched_data,
                                        s + (gen_shift as f64 * 0.001),
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
