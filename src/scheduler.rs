use std::time::{Instant, Duration};
use std::{sync, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::generator::Generator;
use ruffbox_synth::ruffbox::Ruffbox;
use parking_lot::Mutex;

/// A simple time-recursion event scheduler running at a fixed time interval.
pub struct Scheduler<const BUFSIZE:usize, const NCHAN:usize> {
    pub handle: Option<thread::JoinHandle<()>>,
    pub running: sync::Arc<AtomicBool>,
}

pub struct SchedulerData<const BUFSIZE:usize, const NCHAN:usize> {
    pub start_time: Instant,
    pub stream_time: f64,
    pub logical_time: f64,
    pub last_diff: f64,
    pub generator: Box<Generator>,
    pub ruffbox: sync::Arc<Mutex<Ruffbox<BUFSIZE,NCHAN>>>,
}

impl <const BUFSIZE:usize, const NCHAN:usize> SchedulerData<BUFSIZE, NCHAN> {
    pub fn from_data(data: Box<Generator>, ruffbox: sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>) -> Self {
	// get logical time since start from ruffbox
	let stream_time;
	{
	    let ruff = ruffbox.lock();
	    stream_time = ruff.get_now();
	}
	SchedulerData {
	    start_time: Instant::now(),
	    stream_time: stream_time,
	    logical_time: 0.0, 
	    last_diff: 0.0,
	    generator: data,
	    ruffbox: ruffbox,
	}
    }
}

impl <const BUFSIZE:usize, const NCHAN:usize> Scheduler<BUFSIZE, NCHAN> {
    pub fn new() -> Self {
        Scheduler {
            handle: None,
            running: sync::Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Start this scheduler.
    pub fn start(&mut self, fun: fn(&mut SchedulerData<BUFSIZE, NCHAN>) -> f64, data: Box<Generator>, ruffbox: sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>) {
	self.running.store(true, Ordering::SeqCst);
	let running = self.running.clone();
        self.handle = Some(thread::spawn(move || {
            
	    let mut sched_data:SchedulerData<BUFSIZE, NCHAN> = SchedulerData::<BUFSIZE, NCHAN>::from_data(data, ruffbox);	    
	    
	    while running.load(Ordering::SeqCst) {
		let cur = sched_data.start_time.elapsed().as_secs_f64();
		
                sched_data.last_diff = cur - sched_data.logical_time;
		let next = (fun)(&mut sched_data);
		//println!("cur: {} should: {} next diff: {} stream_diff: {}", cur, sched_data.logical_time,
		//next - sched_data.last_diff,
		//	 sched_data.stream_time - sched_data.logical_time);
		sched_data.logical_time += next;				
		sched_data.stream_time += next;
		// compensate for eventual lateness ...
		thread::sleep(Duration::from_secs_f64(next - sched_data.last_diff)); 
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
