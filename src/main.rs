// take care of these later ...
#![allow(clippy::new_without_default)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::type_complexity)]

// types to represent the AST
pub mod ast_types;
// types to represent the evaluated megra language ...
pub mod builtin_types;
pub mod commands;
pub mod cyc_parser;
pub mod duration_tree;
pub mod editor;
pub mod eval;
pub mod event;
pub mod event_helpers;
pub mod file_interpreter;
pub mod generator;
pub mod generator_processor;
pub mod interpreter;
pub mod load_audio_file;
pub mod markov_sequence_generator;
pub mod midi_input;
pub mod music_theory;
pub mod osc_client;
pub mod parameter;
pub mod parser;
pub mod pfa_growth;
pub mod pfa_reverse;
pub mod real_time_streaming;
pub mod repl;
pub mod sample_set;
pub mod scheduler;
pub mod session;
pub mod synth_parameter_value_arithmetic;

#[rustfmt::skip]
mod standard_library;

mod osc_receiver;
mod osc_sender;
mod visualizer_client;

use crate::builtin_types::*;
use crate::osc_client::OscClient;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::{OutputMode, Session};
use anyhow::anyhow;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use dashmap::DashMap;
use directories_next::ProjectDirs;
use getopts::Options;
use parking_lot::Mutex;
use real_time_streaming::Throw;
use ruffbox_synth::ruffbox::{init_ruffbox, ReverbMode, RuffboxPlayhead};
use standard_library::define_standard_library;

use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, sync, thread};

fn print_help(program: &str, opts: Options) {
    let description = format!(
        "{program}: a markov-chain music language

Mégra is a DSL to make music with markov chains.

Version: 0.0.15

Usage:
    {program} [options] [FILES...]
      ",
    );
    println!("{}", opts.usage(&description));
}

// change this if you need to compile for really unreliable systems that need
// a lot of latency
// will be ignored if ringbuffer feature isn't activated
#[cfg(feature = "ringbuffer")]
const RINGBUFFER_SIZE: usize = 8000;

#[cfg(any(feature = "ringbuffer", feature = "low_latency"))]
const BLOCKSIZE: usize = 128;

#[cfg(any(feature = "ringbuffer", feature = "low_latency"))]
const BLOCKSIZE_FLOAT: f32 = 128.0;

#[cfg(not(any(feature = "ringbuffer", feature = "low_latency")))]
const BLOCKSIZE: usize = 512;

#[cfg(not(any(feature = "ringbuffer", feature = "low_latency")))]
const BLOCKSIZE_FLOAT: f32 = 512.0;

struct RunOptions {
    mode: OutputMode,
    num_live_buffers: usize,
    live_buffer_time: f32,
    max_sample_buffers: usize,
    editor: bool,
    create_sketch: bool,
    load_samples: bool,
    sample_folder: Option<String>,
    base_folder: Option<String>,
    reverb_mode: ReverbMode,
    font: Option<String>,
    font_size: f32,
    downmix_stereo: bool,
    ambisonic_binaural: bool,
    karl_yerkes_mode: bool,
}

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

    opts.optflag("", "nosketch", "don't create new sketch in editor mode");
    opts.optflag(
        "",
        "karl-yerkes-mode",
        "immediately execute current expression on any edit",
    );
    opts.optflag("", "ambisonic-binaural", "enable ambisonic-binaural mode");
    opts.optflag(
        "",
        "use-stereo-samples",
        "don't downmix stereo samples to mono (which is the default behaviour)",
    );

    opts.optflag("h", "help", "Print this help");
    opts.optflag("n", "no-samples", "don't load default samples");
    opts.optopt("o", "output-mode", "output mode (stereo, 8ch)", "stereo");
    opts.optflag("l", "list-devices", "list available audio devices");
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

    opts.optopt("", "sample-folder", "folder to a collection of samples", "");
    opts.optopt(
        "",
        "base",
        "base folder including samples, a sketchbook, and a recordings folder",
        "",
    );

    opts.optopt(
        "",
        "live-buffers",
        "number of live input buffers (creates one input channels per live buffer)",
        "1",
    );

    opts.optopt(
        "",
        "max-sample-buffers",
        "maximum number of sample buffers you can load",
        "3000",
    );

    opts.optopt(
        "",
        "live-buffer-time",
        "the capacity of the live input buffers in seconds",
        "3.0",
    );

    opts.optopt("", "font-size", "editor font size", "15.0");

    let matches = match opts.parse(argv) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {e}. Please see --help for more details");
            return Ok(());
        }
    };

    if matches.opt_present("v") {
        println!("0.0.15");
        return Ok(());
    }

    let editor: bool = !matches.opt_present("r");
    let create_sketch: bool = !matches.opt_present("nosketch");
    let load_samples: bool = !matches.opt_present("n");
    let downmix_stereo: bool = !matches.opt_present("use-stereo-samples");
    let ambisonic_binaural: bool = matches.opt_present("ambisonic-binaural");
    let karl_yerkes_mode: bool = matches.opt_present("karl-yerkes-mode");

    if matches.opt_present("h") {
        print_help(&program, opts);
        return Ok(());
    }

    let out_mode = match matches.opt_str("o").as_deref() {
        Some("16ch") => OutputMode::SixteenChannel,
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
        s.parse().unwrap_or(1)
    } else {
        1
    };

    let max_sample_buffers: usize = if let Some(s) = matches.opt_str("max-sample-buffers") {
        s.parse().unwrap_or(3000)
    } else {
        3000
    };

    let live_buffer_time: f32 = if let Some(s) = matches.opt_str("live-buffer-time") {
        s.parse().unwrap_or(3.0)
    } else {
        3.0
    };

    let font_size: f32 = if let Some(s) = matches.opt_str("font-size") {
        s.parse().unwrap_or(15.0)
    } else {
        15.0
    };

    println!("using a live buffer time of: {live_buffer_time}");

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
            println!("out {:?}", dev.name());
        }
        for dev in host.input_devices()? {
            println!("in {:?}", dev.name());
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
    };

    let input_device = if out_device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == out_device).unwrap_or(false))
    };

    let run_opts = RunOptions {
        mode: out_mode,
        num_live_buffers: num_live_buffers as usize,
        live_buffer_time,
        max_sample_buffers,
        editor,
        create_sketch,
        load_samples,
        sample_folder: matches.opt_str("sample-folder"),
        base_folder: matches.opt_str("base"),
        reverb_mode,
        font: matches.opt_str("font"),
        font_size,
        downmix_stereo,
        ambisonic_binaural,
        karl_yerkes_mode,
    };

    match out_mode {
        OutputMode::Stereo => run::<2>(input_device, output_device, run_opts)?,
        OutputMode::FourChannel => run::<4>(input_device, output_device, run_opts)?,
        OutputMode::EightChannel => run::<8>(input_device, output_device, run_opts)?,
        OutputMode::SixteenChannel => run::<16>(input_device, output_device, run_opts)?,
    }

    Ok(())
}

fn run_input<const NCHAN: usize>(
    input_device: &cpal::Device,
    playhead_in: sync::Arc<Mutex<RuffboxPlayhead<BLOCKSIZE, NCHAN>>>,
    is_recording_input: sync::Arc<AtomicBool>,
    throw_in: Throw<BLOCKSIZE, NCHAN>,
    options: &RunOptions,
) -> Result<Stream, anyhow::Error> {
    let mut in_config: StreamConfig = input_device.default_input_config()?.into();
    in_config.channels = options.num_live_buffers as u16;

    let in_channels = in_config.channels as usize;

    println!("[INPUT] start input stream with {in_channels} channels");

    let err_fn = |err| eprintln!("an error occurred on input stream: {err}");

    #[cfg(not(feature = "ringbuffer"))]
    let in_stream = input_device.build_input_stream(
        &in_config,
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

            if is_recording_input.load(Ordering::SeqCst) {
                let mut stream_item = throw_in.prep_next().unwrap();
                // there might be a faster way to de-interleave here ...
                for (f, frame) in data.chunks(in_channels).enumerate() {
                    for (ch, s) in frame.iter().enumerate() {
                        ruff.write_sample_to_live_buffer(ch, *s);
                        stream_item.buffer[ch][f] = *s;
                    }
                    stream_item.size += 1; // increment once per frame
                }
                throw_in.throw_next(stream_item);
            } else {
                // there might be a faster way to de-interleave here ...
                for frame in data.chunks(in_channels) {
                    for (ch, s) in frame.iter().enumerate() {
                        ruff.write_sample_to_live_buffer(ch, *s);
                    }
                }
            }
        },
        err_fn,
        None,
    )?;

    #[cfg(feature = "ringbuffer")]
    let in_stream = input_device.build_input_stream(
        &in_config,
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

            if is_recording_input.load(Ordering::SeqCst) {
                let current_blocksize = data.len() / in_channels;
                let num_blocks = current_blocksize / BLOCKSIZE;
                let leftover = current_blocksize - (num_blocks * BLOCKSIZE);

                for i in 0..num_blocks {
                    let mut stream_item = throw_in.prep_next().unwrap();
                    // there might be a faster way to de-interleave here ...
                    for (f, frame) in data
                        [i * BLOCKSIZE * in_channels..(i + 1) * BLOCKSIZE * in_channels]
                        .chunks(in_channels)
                        .enumerate()
                    {
                        for (ch, s) in frame.iter().enumerate() {
                            stream_item.buffer[ch][f] = *s;
                        }
                        stream_item.size += 1; // increment once per frame
                    }

                    throw_in.throw_next(stream_item);
                }
                if leftover > 0 {
                    let mut stream_item = throw_in.prep_next().unwrap();
                    // there might be a faster way to de-interleave here ...
                    for (f, frame) in data[num_blocks * BLOCKSIZE * in_channels..]
                        .chunks(in_channels)
                        .enumerate()
                    {
                        for (ch, s) in frame.iter().enumerate() {
                            stream_item.buffer[ch][f] = *s;
                        }
                        stream_item.size += 1; // increment once per frame
                    }
                    throw_in.throw_next(stream_item);
                }
            }

            // there might be a faster way to de-interleave here ...
            for frame in data.chunks(in_channels) {
                for (ch, s) in frame.iter().enumerate() {
                    ruff.write_sample_to_live_buffer(ch, *s);
                }
            }
        },
        err_fn,
        None,
    )?;

    in_stream.play()?;

    Ok(in_stream)
}

fn run_output<const NCHAN: usize>(
    output_device: &cpal::Device,
    playhead_out: sync::Arc<Mutex<RuffboxPlayhead<BLOCKSIZE, NCHAN>>>,
    is_recording_output: sync::Arc<AtomicBool>,
    throw_out: Throw<BLOCKSIZE, NCHAN>,
) -> Result<Stream, anyhow::Error> {
    let mut out_config: StreamConfig = output_device.default_output_config()?.into();
    out_config.channels = NCHAN as u16;

    println!("[OUTPUT] start output stream with {NCHAN} channels");

    let err_fn = |err| eprintln!("an error occurred on input stream: {err}");

    // main audio callback (plain)
    // the plain audio callback for platforms where the blocksize
    // is static, configurable and a power of two (preferably 512)
    // (i.e. jack, coreaudio)
    #[cfg(not(feature = "ringbuffer"))]
    let out_stream = output_device.build_output_stream(
        &out_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // about this lock, see note in the input stream ...
            let mut ruff = playhead_out.lock();

            // as the jack timing from cpal can't be trusted right now, the
            // ruffbox handles it's own logical time ...
            let ruff_out = ruff.process(0.0, true);

            if is_recording_output.load(Ordering::SeqCst) {
                throw_out.write_samples(&ruff_out, BLOCKSIZE);
            }

            // there might be a faster way to de-interleave here ...
            for (frame_count, frame) in data.chunks_mut(NCHAN).enumerate() {
                for ch in 0..NCHAN {
                    frame[ch] = ruff_out[ch][frame_count];
                }
            }
        },
        err_fn,
        None,
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
        &out_config,
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

            let current_blocksize = data.len() / NCHAN;

            //println!(
            //   "av {} need {} read {} write {}",
            //   samples_available, current_blocksize, read_idx, write_idx
            //);

            if samples_available < current_blocksize {
                let mut samples_actually_needed = current_blocksize - samples_available;

                while samples_actually_needed > 0 {
                    let ruff_out = ruff.process(0.0, true);

                    if is_recording_output.load(Ordering::SeqCst) {
                        throw_out.write_samples(&ruff_out, BLOCKSIZE);
                    }

                    //produced += BLOCKSIZE;
                    for ch in 0..NCHAN {
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
            for (_, frame) in data.chunks_mut(NCHAN).enumerate() {
                for ch in 0..NCHAN {
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
        None,
    )?;

    out_stream.play()?;

    Ok(out_stream)
}

fn run<const NCHAN: usize>(
    input_device: Option<cpal::Device>,
    output_device: Option<cpal::Device>,
    options: RunOptions,
) -> Result<(), anyhow::Error> {
    // try getting samplerate
    let sample_rate = if let Some(ref dev) = output_device {
        if let Ok(cfg) = dev.default_output_config() {
            let scfg: StreamConfig = cfg.into();
            scfg.sample_rate.0 as f32
        } else {
            // just as a default guess
            44100.0
        }
    } else {
        // just as a default guess
        44100.0
    };

    let (controls, playhead) = init_ruffbox::<BLOCKSIZE, NCHAN>(
        options.num_live_buffers,
        options.live_buffer_time.into(),
        &options.reverb_mode,
        sample_rate.into(),
        options.max_sample_buffers,
        10,
        options.ambisonic_binaural,
    );

    // OUTPUT RECORDING
    let (throw_out, catch_out) = real_time_streaming::init_real_time_stream::<BLOCKSIZE, NCHAN>(
        (BLOCKSIZE_FLOAT / sample_rate) as f64,
        0.25,
    );

    // INPUT MONITOR RECORDING
    let (throw_in, catch_in) = real_time_streaming::init_real_time_stream::<BLOCKSIZE, NCHAN>(
        (BLOCKSIZE_FLOAT / sample_rate) as f64,
        0.25,
    );

    let is_recording_output = sync::Arc::new(AtomicBool::new(false));
    let is_recording_input = sync::Arc::new(AtomicBool::new(false));

    let rec_control = real_time_streaming::RecordingControl {
        is_recording_output: sync::Arc::clone(&is_recording_output),
        is_recording_input: sync::Arc::clone(&is_recording_input),
        catch_out: Some(catch_out),
        catch_out_handle: None,
        catch_in: Some(catch_in),
        catch_in_handle: None,
        samplerate: sample_rate as u32,
    };

    let playhead_out = sync::Arc::new(Mutex::new(playhead)); // the one for the audio thread (out stream)...

    // keep stream handles alive by
    // keeping them in scope
    let in_stream = if let Some(in_dev) = input_device {
        let playhead_in = sync::Arc::clone(&playhead_out); // the one for the audio thread (in stream)...
        run_input(&in_dev, playhead_in, is_recording_input, throw_in, &options)
    } else {
        Err(anyhow!("can't start input stream"))
    };

    if in_stream.is_ok() {
        println!("[INPUT] input started!");
    } else {
        eprintln!("[INPUT] error starting input!");
    }

    let out_stream = if let Some(out_dev) = output_device {
        run_output(&out_dev, playhead_out, is_recording_output, throw_out)
    } else {
        Err(anyhow!("can't start output stream"))
    };

    if out_stream.is_ok() {
        println!("[OUTPUT] output started!");
    } else {
        eprintln!("[OUTPUT] error starting output!");
    }

    // global data
    let session = Session {
        schedulers: sync::Arc::new(DashMap::new()),
        contexts: sync::Arc::new(DashMap::new()),
        osc_client: OscClient::new(),
        rec_control: sync::Arc::new(Mutex::new(Some(rec_control))),
        globals: sync::Arc::new(GlobalVariables::new()),
        sample_set: SampleAndWavematrixSet::new(),
        ruffbox: sync::Arc::new(controls),
        output_mode: options.mode,
        sync_mode: session::SyncMode::NotOnSilence,
        // define the "standard library"
        functions: sync::Arc::new(define_standard_library()),
    };

    let base_dir = if let Some(p) = options.base_folder {
        let bd = std::path::PathBuf::from(p);
        if !bd.exists() {
            println!("create custom megra resource directory {bd:?}");
            std::fs::create_dir_all(bd.to_str().unwrap())?;
        }
        bd
    } else if let Some(proj_dirs) = ProjectDirs::from("de", "parkellipsen", "megra") {
        if !proj_dirs.config_dir().exists() {
            println!(
                "create default megra resource directory {:?}",
                proj_dirs.config_dir()
            );
            std::fs::create_dir_all(proj_dirs.config_dir().to_str().unwrap())?;
        }
        proj_dirs.config_dir().to_path_buf()
    } else {
        // not the most elegant solution, hope this doesn't happen
        let bd = std::path::PathBuf::from("~/MEGRA_FALLBACK");
        if !bd.exists() {
            println!("create custom megra resource directory {bd:?}");
            std::fs::create_dir_all(bd.to_str().unwrap())?;
        }
        bd
    };

    println!("base dir is: {base_dir:?}");

    let samples_path = if let Some(folder) = options.sample_folder {
        std::path::PathBuf::from(folder)
    } else {
        base_dir.join("samples")
    };

    if !samples_path.exists() {
        println!("create megra samples directory {samples_path:?}");
        std::fs::create_dir_all(samples_path.to_str().unwrap())?;
    }

    let sketchbook_path = base_dir.join("sketchbook");
    if !sketchbook_path.exists() {
        println!("create megra sketchbook directory {sketchbook_path:?}");
        std::fs::create_dir_all(sketchbook_path.to_str().unwrap())?;
    }

    let recordings_path = base_dir.join("recordings");
    if !recordings_path.exists() {
        println!("create megra recordings directory {recordings_path:?}");
        std::fs::create_dir_all(recordings_path.to_str().unwrap())?;
    }

    let init_file_path = sketchbook_path.join("init.megra3");
    if !init_file_path.exists() {
        println!("no startup file found");
    } else {
        let init_path_string = init_file_path.to_str().unwrap().to_string();
        println!("loading init file {init_path_string}");
        file_interpreter::parse_file(
            init_path_string,
            &session,
            base_dir.to_str().unwrap().to_string(),
        );
    };

    // load the default sample set ...
    if options.load_samples {
        println!("load samples from path: {samples_path:?}");
        let controls_arc2 = sync::Arc::clone(&session.ruffbox);
        let stdlib2 = sync::Arc::clone(&session.functions);
        let sample_set2 = session.sample_set.clone();
        thread::spawn(move || {
            commands::load_sample_sets_path(
                &stdlib2,
                &controls_arc2,
                sample_set2,
                &samples_path,
                options.downmix_stereo,
            );
            println!("a command (load default sample sets)");
        });
    }

    if options.editor {
        editor::run_editor(
            session,
            base_dir.display().to_string(),
            options.create_sketch,
            options.font.as_deref(),
            options.font_size,
            options.karl_yerkes_mode,
        )
        .unwrap();
        Ok(())
    } else {
        // start the megra repl
        repl::start_repl(session, base_dir.display().to_string())
    }
}
