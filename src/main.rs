pub mod parser;
pub mod parameter;
pub mod event;
pub mod event_helpers;
pub mod markov_sequence_generator;
pub mod event_processor;
pub mod generator;
pub mod session;
pub mod scheduler;

extern crate anyhow;
extern crate cpal;
extern crate claxon;
extern crate ruffbox_synth;

use std::sync::Arc;
use std::collections::HashSet;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::Ruffbox;
use crate::generator::Generator;
use crate::session::Session;
use crate::parser::SampleSet;
use rustyline::error::ReadlineError;
use rustyline::Editor;

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
    let _sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let ruffbox = Arc::new(Mutex::new(Ruffbox::<512>::new()));

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

    let mut session = Session::new();
    let mut sample_set = SampleSet::new();
    
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("megra>> ");
        match readline {
            Ok(line) => {
		
		let pfa_in = parser::eval_from_str(&line.as_str(), &sample_set);

		match pfa_in {
		    Err(e) => {
			println!("parser error {}", e);
		    },
		    Ok(pfa) => {
			match pfa {
			    parser::Expr::Constant(parser::Atom::MarkovSequenceGenerator(p)) => {
				let name = p.name.clone();
				let gen = Box::new(Generator {
				    name: p.name.clone(),
				    root_generator: p,
				    processors: Vec::new()
				});
				
				session.start_generator(gen, Arc::clone(&ruffbox));
				println!("a generator called \'{}\'", name);
			    },
			    parser::Expr::Constant(parser::Atom::Command(c)) => {
				match c {
				    parser::Command::Clear => {
					session.clear_session();
					println!("a command (stop session)");
				    },
				    parser::Command::LoadSample((set, mut keywords, path)) => {
					
					let mut sample_buffer:Vec<f32> = Vec::new();
					let mut reader = claxon::FlacReader::open(path.clone()).unwrap();

					println!("sample path: {} channels: {}", path, reader.streaminfo().channels);

					// decode to f32
					let max_val = (i32::MAX >> (32 - reader.streaminfo().bits_per_sample)) as f32;
					for sample in reader.samples() {
					    let s = sample.unwrap() as f32 / max_val;
					    sample_buffer.push(s);				    
					}
					
					let mut ruff = ruffbox.lock();
					let bufnum = ruff.load_sample(&sample_buffer);

					let mut keyword_set = HashSet::new();
					for k in keywords.drain(..) {
					    keyword_set.insert(k);
					}
					
					sample_set.entry(set).or_insert(Vec::new()).push((keyword_set, bufnum));
					
					println!("a command (load sample)");
				    }
				};
				
			    },
			    parser::Expr::Constant(parser::Atom::Float(f)) => {
				println!("a number: {}", f)
			    },		    
			    _ => println!("unknown")
			}
			
			rl.add_history_entry(line.as_str());						
		    }
		}
	    },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    rl.save_history("history.txt").unwrap();
    Ok(())
}
