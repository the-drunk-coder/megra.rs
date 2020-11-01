pub mod parser;
pub mod parameter;
pub mod event;
pub mod markov_sequence_generator;
pub mod event_processor;
pub mod generator;
pub mod session;
pub mod scheduler;

extern crate anyhow;
extern crate cpal;
extern crate ruffbox_synth;

use std::sync::Arc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::time::{Instant, Duration};
use vom_rs::pfa::Pfa;
use ruffbox_synth::ruffbox::Ruffbox;
use ruffbox_synth::ruffbox::synth::{SynthParameter, SourceType};
use std::{sync, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;
use crate::scheduler::{Scheduler, SchedulerData};
use crate::generator::Generator;

fn main() -> Result<(), anyhow::Error> {
    
    let host = cpal::host_from_id(cpal::available_hosts()
		       .into_iter()
		       .find(|id| *id == cpal::HostId::Jack)
		       .expect(
			   "make sure --features jack is specified. only works on OSes where jack is available",
		       )).expect("jack host unavailable");
	    
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");

    let config:cpal::SupportedStreamConfig = device.default_output_config().unwrap();
    let sample_format = config.sample_format();
    
    match sample_format  {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into())?,
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into())?,
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into())?,
    }

    Ok(())
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let ruffbox = Arc::new(Mutex::new(Ruffbox::new()));

    let ruffbox2 = Arc::clone(&ruffbox);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], cbinfo: &cpal::OutputCallbackInfo| {
	    let mut ruff = ruffbox2.lock();
            let ruff_out = ruff.process(cbinfo.timestamp().playback.as_secs());
	    let mut frame_count = 0;
	    for frame in data.chunks_mut(channels) {
		frame[0] = ruff_out[0][frame_count];
		frame[1] = ruff_out[1][frame_count];
		frame_count = frame_count + 1;
	    }
        },
        err_fn,
    )?;
    stream.play()?;

    let mut input = String::new();

    loop {
	io::stdin().read_line(&mut input)?;

	let mut pfa_in = parser::eval_from_str(&input).unwrap();

	match pfa_in {
	    parser::Expr::Constant(parser::Atom::MarkovSequenceGenerator(p)) => {

		let mut sched = Scheduler::new();
		let mut gen = Box::new(Generator {
		    name: "ho".to_string(),
		    root_generator: p,
		    processors: Vec::new()		
		});
						
		sched.start(move |data: &mut SchedulerData| -> f64 {
		    
		    //println!{"diff: {0}", data.last_diff};		    
		    match data.generator.root_generator.generator.next_symbol() {
			Some(sym) => {
			    let freq = match sym {
				'a' => 330.0,
				'b' => 440.0,
				'c' => 550.0,
				'd' => 660.0,
				'f' => 2.0 * 334.0,
				'r' => 2.0 * 444.0,
				't' => 2.0 * 554.0,
				'v' => 2.0 * 664.0,
				_ => 1000.0
			    };
			    
			    let mut ruff = data.ruffbox.lock();
			    let inst = ruff.prepare_instance(SourceType::LFSawSynth, 2.0, 0);
			    ruff.set_instance_parameter(inst, SynthParameter::PitchFrequency, freq);
			    ruff.set_instance_parameter(inst, SynthParameter::StereoPosition, -1.0);
			    ruff.set_instance_parameter(inst, SynthParameter::Level, 1.0);
			    ruff.set_instance_parameter(inst, SynthParameter::Attack, 0.01);
			    ruff.set_instance_parameter(inst, SynthParameter::Sustain, 0.1);
			    ruff.set_instance_parameter(inst, SynthParameter::Release, 0.89);
			    
			    ruff.trigger(inst);						    
			},
			None => println!(" NIL"),
		    };
		    
		    1.0
		}, gen, Arc::clone(&ruffbox));		
	    },
	    _ => println!("unknown")		
	}
    }
    
    //Ok(())                    
}
