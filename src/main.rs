pub mod builtin_types;
pub mod parser;
pub mod interpreter;
pub mod parameter;
pub mod event;
pub mod event_helpers;
pub mod markov_sequence_generator;
pub mod generator_processor;
pub mod generator;
pub mod session;
pub mod scheduler;
pub mod repl;
pub mod editor;
pub mod commands;
pub mod sample_set;
pub mod cyc_parser;

use directories_next::ProjectDirs;
use getopts::Options;
use std::{env, sync, thread};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::Ruffbox;
use crate::session::{Session, OutputMode};
use crate::sample_set::SampleSet;
use crate::builtin_types::*;


fn print_help(program: &str, opts: Options) {
    let description = format!(
        "{prog}: a markov-chain music language

Mégra is a DSL to make music with markov chains.

Usage:
    {prog} [options] [FILES...]
      ",
        prog = program,
    );
    println!("{}", opts.usage(&description));
}

fn main() -> Result<(), anyhow::Error> {

    let mut argv = env::args();
    let program = argv.next().unwrap();

    let mut opts = Options::new();
    opts.optflag("v", "version", "Print version");
    opts.optflag("e", "editor", "Use integrated editor (experimental)");
    opts.optflag("h", "help", "Print this help");
    opts.optflag("n", "no-samples", "don't load default samples");
    opts.optopt("o", "output-mode", "output mode (stereo, 8ch)", "stereo");
    opts.optflag("l", "list-devices", "list available devices");
    opts.optopt("d", "device", "choose device", "default");
    

    let matches = match opts.parse(argv) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}. Please see --help for more details", e);
	    return Ok(());
        }
    };
    
    if matches.opt_present("v") {
        println!("{}", "0.0.1");
        return Ok(());
    }

    let editor:bool = matches.opt_present("e");
    let load_samples:bool = !matches.opt_present("n");

    if matches.opt_present("h") {
        print_help(&program, opts);
        return Ok(());
    }
    

    let out_mode = match matches.opt_str("o").as_deref() {
	Some("8ch") => OutputMode::EightChannel,
	Some("4ch") => OutputMode::FourChannel,
	Some("stereo") => OutputMode::Stereo,
	_ => {
	    println!("invalid output mode, assume stereo");
	    OutputMode::Stereo
	},
    };

    #[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"))]
    let host = cpal::host_from_id(cpal::available_hosts()
				  .into_iter()
				  .find(|id| *id == cpal::HostId::Jack)
				  .expect(
				      "make sure --features jack is specified. only works on OSes where jack is available",
				  )).expect("jack host unavailable");

    #[cfg(not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")))]
    let host = cpal::default_host();

    if matches.opt_present("l") {
	for dev in host.output_devices()? {
	    println!("{:?}", dev.name());
	}        
        return Ok(());
    }

    let out_device = if let Some(dev) = matches.opt_str("d") {
	dev
    } else {
	"default".to_string()
    };

    let device = if out_device == "default" {
        host.default_output_device()
    } else {
        host.output_devices()?
            .find(|x| x.name().map(|y| y == out_device).unwrap_or(false))
    }
    .expect("failed to find output device");

    let config:cpal::SupportedStreamConfig = device.default_output_config().unwrap();
    let sample_format = config.sample_format();
        
    match out_mode {
	OutputMode::Stereo => {
	    let mut conf: cpal::StreamConfig = config.into();
	    conf.channels = 2;
	    match sample_format  {		
		cpal::SampleFormat::F32 => run::<f32, 2>(&device, &conf, out_mode, editor, load_samples)?,
		cpal::SampleFormat::I16 => run::<i16, 2>(&device, &conf, out_mode, editor, load_samples)?,
		cpal::SampleFormat::U16 => run::<u16, 2>(&device, &conf, out_mode, editor, load_samples)?,
	    }
	},
	OutputMode::FourChannel => {
	    let mut conf: cpal::StreamConfig = config.into();
	    conf.channels = 4;
	    match sample_format  {
		cpal::SampleFormat::F32 => run::<f32, 4>(&device, &conf, out_mode, editor, load_samples)?,
		cpal::SampleFormat::I16 => run::<i16, 4>(&device, &conf, out_mode, editor, load_samples)?,
		cpal::SampleFormat::U16 => run::<u16, 4>(&device, &conf, out_mode, editor, load_samples)?,
	    }
	},
	OutputMode::EightChannel => {
	    let mut conf: cpal::StreamConfig = config.into();
	    conf.channels = 8;
	    match sample_format  {
		cpal::SampleFormat::F32 => run::<f32, 8>(&device, &conf, out_mode, editor, load_samples)?,
		cpal::SampleFormat::I16 => run::<i16, 8>(&device, &conf, out_mode, editor, load_samples)?,
		cpal::SampleFormat::U16 => run::<u16, 8>(&device, &conf, out_mode, editor, load_samples)?,
	    }
	}
    }
    
    Ok(())
}

fn run<T, const NCHAN:usize>(device: &cpal::Device,
			     config: &cpal::StreamConfig,
			     mode: OutputMode,
			     editor: bool,
			     load_samples: bool) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    // at some point i'll need to implement more samplerates i suppose ...
    let _sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let ruffbox = sync::Arc::new(Mutex::new(Ruffbox::<512, NCHAN>::new()));
    let ruffbox2 = sync::Arc::clone(&ruffbox); // the one for the audio thread ...
    
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
	    let mut ruff = ruffbox2.lock();

	    // as the jack timing from cpal can't be trusted right now, the
	    // ruffbox handles it's own logical time ...
            let ruff_out = ruff.process(0.0, true);
	    let mut frame_count = 0;

	    // there might be a faster way to de-interleave here ... 
	    for frame in data.chunks_mut(channels) {
		for ch in 0..channels {
		    frame[ch] = ruff_out[ch][frame_count];
		}				
		frame_count = frame_count + 1;
	    }
        },
        err_fn,
    )?;
    stream.play()?;

    // global data
    let session = sync::Arc::new(Mutex::new(Session::with_mode(mode)));
    let global_parameters = sync::Arc::new(GlobalParameters::with_capacity(1));
    let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));
    let parts_store = sync::Arc::new(Mutex::new(PartsStore::new()));

    // load the default sample set ...
    if load_samples {
		
	if let Some(proj_dirs) = ProjectDirs::from("de", "parkellipsen", "megra") {
	    if !proj_dirs.config_dir().exists() {
		println!("create megra resource directory {:?}", proj_dirs.config_dir());
		std::fs::create_dir(proj_dirs.config_dir().to_str().unwrap())?;
	    }
	    
	    let samples_path = proj_dirs.config_dir().join("samples");
	    if !samples_path.exists() {
		println!("create megra samples directory {:?}", samples_path);
		std::fs::create_dir(samples_path.to_str().unwrap())?;
	    }
	    
	    let sketchbook_path = proj_dirs.config_dir().join("sketchbook");
	    if !sketchbook_path.exists() {
		println!("create megra sketchbook directory {:?}", sketchbook_path);
		std::fs::create_dir(sketchbook_path.to_str().unwrap())?;
	    }
	    
	    println!("load samples from path: {:?}", samples_path);
	    let ruffbox2 = sync::Arc::clone(&ruffbox);
	    let sample_set2 = sync::Arc::clone(&sample_set);
	    thread::spawn(move || {
		commands::load_sample_sets_path(&ruffbox2, &sample_set2, &samples_path);
		println!("a command (load default sample sets)");
	    });		    
	}
    }
    
    if editor {
	editor::run_editor(&session, &ruffbox, &global_parameters, &sample_set, &parts_store, mode);	
	Ok(())		
    } else {
	// star1t the megra repl
	repl::start_repl(&session, &ruffbox, &global_parameters, &sample_set, &parts_store, mode)
    }    
}
