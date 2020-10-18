use std::time::{Instant, Duration};
use std::{sync, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::generator::Generator;

/// A simple time-recursion event scheduler running at a fixed time interval.
pub struct Scheduler {
    pub handle: Option<thread::JoinHandle<()>>,
    pub running: sync::Arc<AtomicBool>,
}

pub struct SchedulerData {
    pub start_time: Instant,
    pub logical_time: f64,
    pub last_diff: f64,
    pub generator: Box<Generator>,
}

impl SchedulerData {
    pub fn from_data(data: Box<Generator>) -> Self {
	SchedulerData {
	    start_time: Instant::now(),
	    logical_time: 0.0,
	    last_diff: 0.0,
	    generator: data,
	}
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            handle: None,
            running: sync::Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Start this scheduler.
    pub fn start(&mut self, fun: fn(&mut SchedulerData) -> f64, data: Box<Generator>) {
	self.running.store(true, Ordering::SeqCst);
	let running = self.running.clone();
        self.handle = Some(thread::spawn(move || {
            
	    let mut sched_data:SchedulerData = SchedulerData::from_data(data);
	    
	    while running.load(Ordering::SeqCst) {
		let cur = sched_data.start_time.elapsed().as_secs_f64();
                sched_data.last_diff = cur - sched_data.logical_time;
		let next = (fun)(&mut sched_data);		
		sched_data.logical_time += next;
		thread::sleep(Duration::from_secs_f64(next - sched_data.last_diff)); // compensate for eventual lateness ...
            }
        }));
    }

    /// Stop this scheduler.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.handle
            .take().expect("Called stop on non-running thread")
            .join().expect("Could not join spawned thread");
    }
}
