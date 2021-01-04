use std::time::{Instant, Duration};
use std::{sync, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::generator::Generator;
use ruffbox_synth::ruffbox::Ruffbox;
use parking_lot::Mutex;
use crate::session::{OutputMode, Session};
use crate::builtin_types::*;

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
    pub shift: f64,
    pub generator: Box<Generator>,
    pub ruffbox: sync::Arc<Mutex<Ruffbox<BUFSIZE,NCHAN>>>,
    pub session: sync::Arc<Mutex<Session<BUFSIZE,NCHAN>>>,
    pub global_parameters: sync::Arc<GlobalParameters>,
    pub mode: OutputMode
}

impl <const BUFSIZE:usize, const NCHAN:usize> SchedulerData<BUFSIZE, NCHAN> {

    pub fn from_previous(old: &SchedulerData<BUFSIZE, NCHAN>,
			 shift: f64,
			 mut data: Box<Generator>,
			 ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
			 session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>) -> Self {
	let shift_diff = shift - old.shift;
	data.transfer_state(&old.generator);
	// keep scheduling, retain data
	SchedulerData {
	    start_time: old.start_time,
	    stream_time: old.stream_time + shift_diff,
	    logical_time: old.logical_time + shift_diff,
	    shift: shift,
	    last_diff: 0.0,
	    generator: data,
	    ruffbox: sync::Arc::clone(ruffbox),
	    session: sync::Arc::clone(session),
	    global_parameters: sync::Arc::clone(&old.global_parameters),
	    mode: old.mode
	}
    }

    pub fn from_time_data(old: &SchedulerData<BUFSIZE, NCHAN>,
			  shift: f64,
			  data: Box<Generator>,
			  ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
			  session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>) -> Self {
	let shift_diff = shift - old.shift;
	
	// keep scheduling, retain time
	SchedulerData {
	    start_time: old.start_time,
	    stream_time: old.stream_time + shift_diff,
	    logical_time: old.logical_time + shift_diff,
	    shift: shift,
	    last_diff: 0.0,
	    generator: data,
	    ruffbox: sync::Arc::clone(ruffbox),
	    session: sync::Arc::clone(session),
	    global_parameters: sync::Arc::clone(&old.global_parameters),
	    mode: old.mode
	}
    }
    
    pub fn from_data(data: Box<Generator>,
		     shift: f64,
		     session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
		     ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
		     global_parameters: &sync::Arc<GlobalParameters>,
		     mode: OutputMode) -> Self {
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
	    shift: shift,
	    generator: data,
	    ruffbox: sync::Arc::clone(ruffbox),
	    session: sync::Arc::clone(session),
	    global_parameters: sync::Arc::clone(global_parameters),
	    mode: mode
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
    pub fn start(&mut self,
		 fun: fn(&mut SchedulerData<BUFSIZE, NCHAN>) -> f64,
		 data: sync::Arc<Mutex<SchedulerData<BUFSIZE, NCHAN>>>) {
	self.running.store(true, Ordering::SeqCst);
	let running = self.running.clone();
        self.handle = Some(thread::spawn(move || {            	    
	    while running.load(Ordering::SeqCst) {
		let next:f64;
		let ldif:f64;
		{
		    let mut sched_data = data.lock();
		    let cur = sched_data.start_time.elapsed().as_secs_f64();		    
                    sched_data.last_diff = cur - sched_data.logical_time;
		    next = (fun)(&mut sched_data);
		    ldif = sched_data.last_diff ;		    
		    sched_data.logical_time += next;				
		    sched_data.stream_time += next;
		}
		// compensate for eventual lateness ...
		thread::sleep(Duration::from_secs_f64(next - ldif)); 
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
