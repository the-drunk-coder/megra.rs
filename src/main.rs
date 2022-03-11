// take care of these later ...
#![allow(clippy::new_without_default)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::type_complexity)]

pub mod builtin_types;
pub mod commands;
pub mod cyc_parser;
pub mod editor;
pub mod event;
pub mod event_helpers;
pub mod generator;
pub mod generator_processor;
pub mod interpreter;
pub mod markov_sequence_generator;
pub mod music_theory;
pub mod parameter;
pub mod parser;
pub mod pfa_growth;
pub mod pfa_reverse;
pub mod real_time_streaming;
pub mod repl;
pub mod sample_set;
pub mod scheduler;
pub mod session;

#[rustfmt::skip]
mod standard_library;

mod visualizer_client;

use crate::builtin_types::*;
use crate::sample_set::SampleSet;
use crate::session::{OutputMode, Session};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use directories_next::ProjectDirs;
use getopts::Options;
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::{init_ruffbox, ReverbMode};
use standard_library::define_standard_library;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, sync, thread};

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

// change this if you need to compile for really unreliable systems that need
// a lot of latency
// will be ignored if ringbuffer feature isn't activated
#[cfg(feature = "ringbuffer")]
const RINGBUFFER_SIZE: usize = 8000;
#[cfg(feature = "ringbuffer")]
const BLOCKSIZE: usize = 128;

fn main() -> Result<(), anyhow::Error> {
    let mut argv = env::args();
    let program = argv.next().unwrap();

    let mut opts = Options::new();
    opts.optflag("v", "version", "Print version");
    opts.optflag(
        "r",
        "repl",
        "no editor, repl only (i.e. for integration with other editors)",
    );
    opts.optflag("h", "help", "Print this help");
    opts.optflag("n", "no-samples", "don't load default samples");
    opts.optopt("o", "output-mode", "output mode (stereo, 8ch)", "stereo");
    opts.optflag("l", "list-devices", "list available devices");
    opts.optopt("d", "device", "choose device", "default");
    opts.optopt(
        "",
        "reverb-mode",
        "reverb mode (freeverb or convolution)",
        "freeverb",
    );
    opts.optopt(
        "",
        "font",
        "editor font (ComicMono, mononoki or custom path)",
        "mononoki",
    );
    opts.optopt("", "reverb-ir", "reverb impulse response (file)", "");

    opts.optopt(
        "",
        "live-buffers",
        "number of live input buffers (creates one input channels per live buffer)",
        "1",
    );

    opts.optopt(
        "",
        "live-buffer-time",
        "the capacity of the live input buffers in seconds",
        "3.0",
    );

    let matches = match opts.parse(argv) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}. Please see --help for more details", e);
            return Ok(());
        }
    };

    if matches.opt_present("v") {
        println!("0.0.4");
        return Ok(());
    }

    let editor: bool = !matches.opt_present("r");
    let load_samples: bool = !matches.opt_present("n");

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
        }
    };

    let reverb_mode = match matches.opt_str("reverb-mode").as_deref() {
        Some("freeverb") => ReverbMode::FreeVerb,
        Some("convolution") => {
            match matches.opt_str("reverb-ir").as_deref() {
                Some(filepath) => {
                    if let Ok(mut reader) = claxon::FlacReader::open(filepath) {
                        let mut sample_buffer: Vec<f32> = Vec::new();
                        // decode to f32
                        let max_val =
                            (i32::MAX >> (32 - reader.streaminfo().bits_per_sample)) as f32;

                        for sample in reader.samples() {
                            let s = sample.unwrap() as f32 / max_val;
                            sample_buffer.push(s);
                        }
                        ReverbMode::Convolution(
                            sample_buffer,
                            reader.streaminfo().sample_rate as f32,
                        )
                    } else {
                        println!("reverb ir path invalid, fall back to freeverb");
                        ReverbMode::FreeVerb
                    }
                }
                None => {
                    println!("no reverb ir provided, fall back to freeverb");
                    ReverbMode::FreeVerb
                }
            }
        }
        _ => ReverbMode::FreeVerb,
    };

    let num_live_buffers: u16 = if let Some(s) = matches.opt_str("live-buffers") {
        if let Ok(f) = s.parse() {
            f
        } else {
            1
        }
    } else {
        1
    };

    let live_buffer_time: f32 = if let Some(s) = matches.opt_str("live-buffer-time") {
        if let Ok(f) = s.parse() {
            f
        } else {
            3.0
        }
    } else {
        3.0
    };

    println!("using a live buffer time of: {}", live_buffer_time);

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

    let output_device = if out_device == "default" {
        host.default_output_device()
    } else {
        host.output_devices()?
            .find(|x| x.name().map(|y| y == out_device).unwrap_or(false))
    }
    .expect("failed to find output device");

    let input_device = if out_device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == out_device).unwrap_or(false))
    }
    .expect("failed to find input device");

    println!("odev {}", output_device.name().unwrap());

    let out_config: cpal::SupportedStreamConfig = output_device.default_output_config().unwrap();
    let in_config: cpal::SupportedStreamConfig = input_device.default_input_config().unwrap();

    // let's assume it's the same for both ...
    let sample_format = out_config.sample_format();

    match out_mode {
        OutputMode::Stereo => {
            let mut out_conf: cpal::StreamConfig = out_config.into();
            let mut in_conf: cpal::StreamConfig = in_config.into();
            in_conf.channels = num_live_buffers;
            out_conf.channels = 2;
            match sample_format {
                cpal::SampleFormat::F32 => run::<f32, 2>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
                cpal::SampleFormat::I16 => run::<i16, 2>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
                cpal::SampleFormat::U16 => run::<u16, 2>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
            }
        }
        OutputMode::FourChannel => {
            let mut out_conf: cpal::StreamConfig = out_config.into();
            let mut in_conf: cpal::StreamConfig = in_config.into();
            in_conf.channels = num_live_buffers;
            out_conf.channels = 4;
            match sample_format {
                cpal::SampleFormat::F32 => run::<f32, 4>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
                cpal::SampleFormat::I16 => run::<i16, 4>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
                cpal::SampleFormat::U16 => run::<u16, 4>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
            }
        }
        OutputMode::EightChannel => {
            let mut out_conf: cpal::StreamConfig = out_config.into();
            let mut in_conf: cpal::StreamConfig = in_config.into();
            in_conf.channels = num_live_buffers;
            out_conf.channels = 8;
            match sample_format {
                cpal::SampleFormat::F32 => run::<f32, 8>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
                cpal::SampleFormat::I16 => run::<i16, 8>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
                cpal::SampleFormat::U16 => run::<u16, 8>(
                    &input_device,
                    &output_device,
                    &out_conf,
                    &in_conf,
                    out_mode,
                    num_live_buffers as usize,
                    live_buffer_time,
                    editor,
                    load_samples,
                    &reverb_mode,
                    matches.opt_str("font").as_deref(),
                )?,
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run<T, const NCHAN: usize>(
    input_device: &cpal::Device,
    output_device: &cpal::Device,
    out_config: &cpal::StreamConfig,
    in_config: &cpal::StreamConfig,
    mode: OutputMode,
    num_live_buffers: usize,
    live_buffer_time: f32,
    editor: bool,
    load_samples: bool,
    reverb_mode: &ReverbMode,
    font: Option<&str>,
) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    // at some point i'll need to implement more samplerates i suppose ...
    let sample_rate = out_config.sample_rate.0 as f32;
    let out_channels = out_config.channels as usize;
    let in_channels = in_config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    println!("in chan: {} out chan: {}", in_channels, out_channels);

    #[cfg(feature = "ringbuffer")]
    let (controls, playhead) = init_ruffbox::<128, NCHAN>(
        num_live_buffers,
        live_buffer_time.into(),
        reverb_mode,
        sample_rate.into(),
        3000,
        10,
    );

    #[cfg(not(feature = "ringbuffer"))]
    let (controls, playhead) = init_ruffbox::<512, NCHAN>(
        num_live_buffers,
        live_buffer_time.into(),
        reverb_mode,
        sample_rate.into(),
        3000,
        10,
    );

    let playhead_out = sync::Arc::new(Mutex::new(playhead)); // the one for the audio thread (out stream)...
    let playhead_in = sync::Arc::clone(&playhead_out); // the one for the audio thread (in stream)...
    let in_stream = input_device.build_input_stream(
        in_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // these are the only two locks that are left.
            // Maybe, in the future, I could get around them using
            // some interior mutability pattern in the ruffbox playhead,
            // but given that the only other point where the lock \
            // is called is the output callback, and they have to be called in
            // sequence anyway (or at least in a deterministic fashion, i hope),
            // the lock here shouldn't hurt much (in fact it worked nicely even before
            // it was possible to call the controls without a lock).
            // Unless I run into trouble, this might just stay the way it is for now.
            let mut ruff = playhead_in.lock();

            // there might be a faster way to de-interleave here ...
            // only use first input channel
            for (_, frame) in data.chunks(in_channels).enumerate() {
                for (ch, s) in frame.iter().enumerate() {
                    ruff.write_sample_to_live_buffer(ch, *s);
                }
            }
        },
        err_fn,
    )?;

    // write real time stream 4 times a sec ...
    #[cfg(not(feature = "ringbuffer"))]
    let (throw, catch) = real_time_streaming::init_real_time_stream::<512, NCHAN>(
        (512.0 / sample_rate) as f64,
        0.25,
    );

    #[cfg(feature = "ringbuffer")]
    let (throw, catch) = real_time_streaming::init_real_time_stream::<128, NCHAN>(
        (128.0 / sample_rate) as f64,
        0.25,
    );

    let is_recording = sync::Arc::new(AtomicBool::new(false));

    let rec_control = real_time_streaming::RecordingControl {
        is_recording: sync::Arc::clone(&is_recording),
        catch: Some(catch),
        catch_handle: None,
        samplerate: sample_rate as u32,
    };

    // main audio callback (plain)
    // the plain audio callback for platforms where the blocksize
    // is static, configurable and a power of two (preferably 512)
    // (i.e. jack, coreaudio)
    #[cfg(not(feature = "ringbuffer"))]
    let out_stream = output_device.build_output_stream(
        out_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // about this lock, see note in the input stream ...
            let mut ruff = playhead_out.lock();

            // as the jack timing from cpal can't be trusted right now, the
            // ruffbox handles it's own logical time ...
            let ruff_out = ruff.process(0.0, true);

            if is_recording.load(Ordering::SeqCst) {
                throw.write_samples(&ruff_out, 512);
            }

            // there might be a faster way to de-interleave here ...
            for (frame_count, frame) in data.chunks_mut(out_channels).enumerate() {
                for ch in 0..out_channels {
                    frame[ch] = ruff_out[ch][frame_count];
                }
            }
        },
        err_fn,
    )?;

    // main audio callback (with )
    // this is the ringbuffer version that internally buffers the audio
    // stream to allow a fixed blocksize being used for the ruffbox synth
    // even if the system doesn't provide a fixed and/or configurable blocksize
    // might require a higher latency
    #[cfg(feature = "ringbuffer")]
    println!("using ringbuffer to adapt blocksize, you might need to use a higher latency");

    #[cfg(feature = "ringbuffer")]
    let mut ringbuffer: [[f32; RINGBUFFER_SIZE]; NCHAN] = [[0.0; RINGBUFFER_SIZE]; NCHAN];
    #[cfg(feature = "ringbuffer")]
    let mut write_idx: usize = 0;
    #[cfg(feature = "ringbuffer")]
    let mut read_idx: usize = 0;
    #[cfg(feature = "ringbuffer")]
    let out_stream = output_device.build_output_stream(
        out_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // about this lock, see note in the input stream ...
            let mut ruff = playhead_out.lock();

            let samples_available = if write_idx < read_idx {
                write_idx + RINGBUFFER_SIZE - read_idx
            } else if write_idx > read_idx {
                write_idx - read_idx
            } else {
                0
            };
            //let mut produced:usize = 0;

            let current_blocksize = data.len() / out_channels;

            //println!(
            //   "av {} need {} read {} write {}",
            //   samples_available, current_blocksize, read_idx, write_idx
            //);

            if samples_available < current_blocksize {
                let mut samples_actually_needed = current_blocksize - samples_available;

                while samples_actually_needed > 0 {
                    let ruff_out = ruff.process(0.0, true);

                    if is_recording.load(Ordering::SeqCst) {
                        throw.write_samples(&ruff_out, 128);
                    }

                    //produced += BLOCKSIZE;
                    for ch in 0..out_channels {
                        let mut tmp_write_idx = write_idx;
                        for s in 0..BLOCKSIZE {
                            ringbuffer[ch][tmp_write_idx] = ruff_out[ch][s];
                            tmp_write_idx += 1;
                            if tmp_write_idx >= RINGBUFFER_SIZE {
                                tmp_write_idx = 0;
                            }
                        }
                    }

                    write_idx += BLOCKSIZE;
                    if write_idx >= RINGBUFFER_SIZE {
                        write_idx = write_idx - RINGBUFFER_SIZE;
                    }

                    samples_actually_needed = if samples_actually_needed > BLOCKSIZE {
                        samples_actually_needed - BLOCKSIZE
                    } else {
                        0
                    }
                }
            }

            // there might be a faster way to de-interleave here ...
            for (_, frame) in data.chunks_mut(out_channels).enumerate() {
                for ch in 0..out_channels {
                    frame[ch] = ringbuffer[ch][read_idx];
                }
                read_idx += 1;
                if read_idx >= RINGBUFFER_SIZE {
                    read_idx = 0;
                }
            }
            /*
                println!(
                "POST BLOCK av {} bs {} to prod {} prod {} r idx {} w idx {} NOW {}",
                samples_available,
                current_blocksize,
                current_blocksize - samples_available,
                produced,
                read_idx,
                write_idx,
                ruff.get_now()
            );
                 */
        },
        err_fn,
    )?;

    in_stream.play()?;
    out_stream.play()?;

    // global data
    let mut raw_session = Session::new();
    raw_session.rec_control = Some(rec_control);
    let session = sync::Arc::new(Mutex::new(raw_session));

    let global_parameters = sync::Arc::new(GlobalParameters::with_capacity(1));
    let sample_set = sync::Arc::new(Mutex::new(SampleSet::new()));
    let parts_store = sync::Arc::new(Mutex::new(PartsStore::new()));
    // define the "standard library"
    let stdlib = sync::Arc::new(Mutex::new(define_standard_library()));
    let controls_arc = sync::Arc::new(controls);

    if let Some(proj_dirs) = ProjectDirs::from("de", "parkellipsen", "megra") {
        if !proj_dirs.config_dir().exists() {
            println!(
                "create megra resource directory {:?}",
                proj_dirs.config_dir()
            );
            std::fs::create_dir_all(proj_dirs.config_dir().to_str().unwrap())?;
        }

        let samples_path = proj_dirs.config_dir().join("samples");
        if !samples_path.exists() {
            println!("create megra samples directory {:?}", samples_path);
            std::fs::create_dir_all(samples_path.to_str().unwrap())?;
        }

        let sketchbook_path = proj_dirs.config_dir().join("sketchbook");
        if !sketchbook_path.exists() {
            println!("create megra sketchbook directory {:?}", sketchbook_path);
            std::fs::create_dir_all(sketchbook_path.to_str().unwrap())?;
        }

        let recordings_path = proj_dirs.config_dir().join("recordings");
        if !recordings_path.exists() {
            println!("create megra recordings directory {:?}", recordings_path);
            std::fs::create_dir_all(recordings_path.to_str().unwrap())?;
        }
        // load the default sample set ...
        if load_samples {
            println!("load samples from path: {:?}", samples_path);
            let controls_arc2 = sync::Arc::clone(&controls_arc);
            let sample_set2 = sync::Arc::clone(&sample_set);
            let stdlib2 = sync::Arc::clone(&stdlib);
            thread::spawn(move || {
                commands::load_sample_sets_path(
                    &stdlib2,
                    &controls_arc2,
                    &sample_set2,
                    &samples_path,
                );
                println!("a command (load default sample sets)");
            });
        }
    }

    if editor {
        editor::run_editor(
            &stdlib,
            &session,
            &controls_arc,
            &global_parameters,
            &sample_set,
            &parts_store,
            mode,
            font,
        );
        Ok(())
    } else {
        // start the megra repl
        repl::start_repl(
            &stdlib,
            &session,
            &controls_arc,
            &global_parameters,
            &sample_set,
            &parts_store,
            mode,
        )
    }
}
