use parking_lot::Mutex;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
    sync,
};
use vom_rs::pfa;

use ruffbox_synth::ruffbox::{synth::SynthParameter, RuffboxControls};

use crate::builtin_types::*;
use crate::event::*;
use crate::event_helpers::*;
use crate::generator::*;
use crate::parameter::*;
use crate::parser::eval;
use crate::parser::FunctionMap;
use crate::real_time_streaming;
use crate::sample_set::SampleSet;
use crate::session::*;
use std::sync::atomic::Ordering;

pub fn freeze_buffer<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    freezbuf: usize,
    inbuf: usize,
) {
    ruffbox.freeze_buffer(freezbuf, inbuf);
}

pub fn load_sample<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    set: String,
    keywords: &mut Vec<String>,
    path: String,
) {
    let mut sample_buffer: Vec<f32> = Vec::new();
    let mut reader = claxon::FlacReader::open(path.clone()).unwrap();

    let mut duration = if let Some(samples) = reader.streaminfo().samples {
        let tmp_dur = 1000.0
            * ((samples as f32 / reader.streaminfo().channels as f32)
                / reader.streaminfo().sample_rate as f32);
        tmp_dur as usize
    } else {
        200
    };

    // max ten seconds
    if duration > 10000 {
        duration = 10000;
    }

    // decode to f32
    let max_val = (i32::MAX >> (32 - reader.streaminfo().bits_per_sample)) as f32;
    for sample in reader.samples() {
        let s = sample.unwrap() as f32 / max_val;
        sample_buffer.push(s);
    }

    // adds interpolation samples to sample buffer, don't use afterwards
    let bufnum = ruffbox.load_sample(
        &mut sample_buffer,
        true,
        reader.streaminfo().sample_rate as f32,
    );

    let mut keyword_set = HashSet::new();
    for k in keywords.drain(..) {
        keyword_set.insert(k);
    }

    let path2 = Path::new(&path);
    if let Some(os_filename) = path2.file_stem() {
        if let Some(str_filename) = os_filename.to_str() {
            let tokens = str_filename.split(|c| c == ' ' || c == '_' || c == '-' || c == '.');
            for token in tokens {
                keyword_set.insert(token.to_lowercase().to_string());
            }
        }
    }

    if reader.streaminfo().sample_rate != ruffbox.samplerate as u32 {
        println!("adapt duration");
        duration = (duration as f32 * (reader.streaminfo().sample_rate as f32 / ruffbox.samplerate))
            as usize;
    }

    println!(
        "sample path: {} channels: {} dur: {} orig sr: {} ruf sr: {} resampled: {}",
        path,
        reader.streaminfo().channels,
        duration,
        reader.streaminfo().sample_rate,
        ruffbox.samplerate,
        reader.streaminfo().sample_rate != ruffbox.samplerate as u32
    );

    sample_set
        .lock()
        .insert(set.clone(), keyword_set, bufnum, duration);
    function_map
        .lock()
        .fmap
        .insert(set, eval::events::sound::sound);
}

pub fn load_sample_set<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    samples_path: &Path,
) {
    // determine set name or use default
    let set_name = if let Some(os_filename) = samples_path.file_stem() {
        if let Some(str_filename) = os_filename.to_str() {
            str_filename.to_string()
        } else {
            "default".to_string()
        }
    } else {
        "default".to_string()
    };

    if let Ok(entries) = fs::read_dir(samples_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            // only consider files here ...
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "flac" {
                        load_sample(
                            function_map,
                            ruffbox,
                            sample_set,
                            set_name.clone(),
                            &mut Vec::new(),
                            path.to_str().unwrap().to_string(),
                        );
                    }
                }
            }
        }
    }
}

pub fn load_sample_set_string<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    samples_path: String,
) {
    let path = Path::new(&samples_path);
    load_sample_set(function_map, ruffbox, sample_set, path);
}

pub fn load_sample_sets<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    folder_path: String,
) {
    let root_path = Path::new(&folder_path);
    load_sample_sets_path(function_map, ruffbox, sample_set, root_path);
}

pub fn load_sample_sets_path<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    root_path: &Path,
) {
    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                load_sample_set(function_map, ruffbox, sample_set, &path);
            }
        }
    }
}

pub fn load_part(parts_store: &sync::Arc<Mutex<PartsStore>>, name: String, part: Part) {
    let mut ps = parts_store.lock();
    ps.insert(name, part);
}

/// execute a pre-defined part step by step
pub fn start_recording<const BUFSIZE: usize, const NCHAN: usize>(
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
) {
    let maybe_rec_ctrl = session.lock().rec_control.take();
    if let Some(mut rec_ctrl) = maybe_rec_ctrl {
	if rec_ctrl.is_recording.load(Ordering::SeqCst) {
	    println!("there's already a recording in progress, please stop first !");
	} else {
	    let maybe_catch = rec_ctrl.catch.take();
            if let Some(catch) = maybe_catch {
		rec_ctrl.catch_handle = Some(real_time_streaming::start_writer_thread(
                    catch,
                    44100,
                    "megra_foo.wav".to_string(),
		));
		rec_ctrl.is_recording.store(true, Ordering::SeqCst);
            }
	}	        
        session.lock().rec_control = Some(rec_ctrl);
    }
}

/// execute a pre-defined part step by step
pub fn stop_recording<const BUFSIZE: usize, const NCHAN: usize>(
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
) {
    let maybe_rec_ctrl = session.lock().rec_control.take();
    if let Some(mut rec_ctrl) = maybe_rec_ctrl {
	if rec_ctrl.is_recording.load(Ordering::SeqCst) {
	    let maybe_catch_handle = rec_ctrl.catch_handle.take();
            if let Some(catch_handle) = maybe_catch_handle {
		rec_ctrl.is_recording.store(false, Ordering::SeqCst);
		real_time_streaming::stop_writer_thread(catch_handle);
            }
	} else {
	    println!("can't stop recording that isn't running !");
	}        
        session.lock().rec_control = Some(rec_ctrl);
    }
}

/// execute a pre-defined part step by step
pub fn step_part<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
    global_parameters: &sync::Arc<GlobalParameters>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    output_mode: OutputMode,
    part_name: String,
) {
    let mut sound_events = Vec::new();
    let mut control_events = Vec::new();

    {
        let mut ps = parts_store.lock();
        if let Some(Part::Combined(gens, _)) = ps.get_mut(&part_name) {
            for gen in gens.iter_mut() {
                gen.current_transition(global_parameters);
                let mut current_events = gen.current_events(global_parameters);
                for ev in current_events.drain(..) {
                    match ev {
                        InterpretableEvent::Control(c) => control_events.push(c),
                        InterpretableEvent::Sound(s) => sound_events.push(s),
                    }
                }
            }
        }
    }

    // execute retrieved events
    once(
        ruffbox,
        parts_store,
        global_parameters,
        session,
        &mut sound_events,
        &mut control_events,
        output_mode,
    );
}

pub fn set_global_tmod(global_parameters: &sync::Arc<GlobalParameters>, p: Parameter) {
    global_parameters.insert(
        BuiltinGlobalParameters::GlobalTimeModifier,
        ConfigParameter::Dynamic(p),
    ); // init on first attempt
}

pub fn set_global_latency(global_parameters: &sync::Arc<GlobalParameters>, p: Parameter) {
    global_parameters.insert(
        BuiltinGlobalParameters::GlobalLatency,
        ConfigParameter::Dynamic(p),
    ); // init on first attempt
}

pub fn set_default_duration(global_parameters: &sync::Arc<GlobalParameters>, n: f32) {
    global_parameters.insert(
        BuiltinGlobalParameters::DefaultDuration,
        ConfigParameter::Numeric(n),
    ); // init on first attempt
}

pub fn set_global_lifemodel_resources(global_parameters: &sync::Arc<GlobalParameters>, val: f32) {
    global_parameters.insert(
        BuiltinGlobalParameters::LifemodelGlobalResources,
        ConfigParameter::Numeric(val),
    ); // init on first attempt
}

pub fn set_global_ruffbox_parameters<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    params: &HashMap<SynthParameter, f32>,
) {
    for (k, v) in params.iter() {
        ruffbox.set_master_parameter(*k, *v);
    }
}

pub fn export_dot_static(filename: &str, generator: &Generator) {
    let dot_string = pfa::to_dot::<char>(&generator.root_generator.generator);
    println!("export to {}", filename);
    fs::write(filename, dot_string).expect("Unable to write file");
}

pub fn export_dot_part(
    filename: &str,
    part_name: &str,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
) {
    let ps = parts_store.lock();
    if let Some(Part::Combined(gens, _)) = ps.get(part_name) {
        // write generators to dot strings ...
        for gen in gens.iter() {
            let mut filename_tagged = filename.to_string();
            filename_tagged.push('_');
            filename_tagged.push_str(part_name);
            filename_tagged.push('_');
            for tag in gen.id_tags.iter() {
                filename_tagged.push_str(tag);
                filename_tagged.push('_');
            }
            // remove trailing _
            filename_tagged = filename_tagged[..filename_tagged.len() - 1].to_string();
            filename_tagged.push_str(".dot");
            let dot_string = pfa::to_dot::<char>(&gen.root_generator.generator);
            println!("export to {}", filename_tagged);
            fs::write(filename_tagged, dot_string).expect("Unable to write file");
        }
    }
}

pub fn export_dot_running<const BUFSIZE: usize, const NCHAN: usize>(
    filename: &str,
    tags: &BTreeSet<String>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
) {
    let mut gens = Vec::new();

    {
        let sess = session.lock();
        for (id_tags, (_, sched_data)) in sess.schedulers.iter() {
            if !tags.is_disjoint(id_tags) {
                let data = sched_data.lock();
                // get a snapshot of the generator in it's current state
                gens.push((id_tags.clone(), data.generator.clone()));
            }
        }
    }

    // write generators to dot strings ...
    for (tags, gen) in gens.iter() {
        let mut filename_tagged = filename.to_string();
        filename_tagged.push('_');
        for tag in tags.iter() {
            filename_tagged.push_str(tag);
            filename_tagged.push('_');
        }
        // remove trailing _
        filename_tagged = filename_tagged[..filename_tagged.len() - 1].to_string();
        filename_tagged.push_str(".dot");
        let dot_string = pfa::to_dot::<char>(&gen.root_generator.generator);
        println!("export to {}", filename_tagged);
        fs::write(filename_tagged, dot_string).expect("Unable to write file");
    }
}

pub fn once<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
    global_parameters: &sync::Arc<GlobalParameters>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    sound_events: &mut Vec<StaticEvent>,
    control_events: &mut Vec<ControlEvent>,
    output_mode: OutputMode,
) {
    for cev in control_events.iter() {
        if let Some(mut contexts) = cev.ctx.clone() {
            // this is the worst clone ....
            for mut sx in contexts.drain(..) {
                Session::handle_context(
                    &mut sx,
                    session,
                    ruffbox,
                    parts_store,
                    global_parameters,
                    output_mode,
                );
            }
        }
    }

    for s in sound_events.iter_mut() {
        if s.name == "silence" {
            continue;
        }

        let mut bufnum: usize = 0;
        if let Some(b) = s.params.get(&SynthParameter::SampleBufferNumber) {
            bufnum = *b as usize;
        }

        // latency 0.05, should be made configurable later ...
        if let Some(mut inst) = ruffbox.prepare_instance(map_name(&s.name), 0.0, bufnum) {
            // set parameters and trigger instance
            for (k, v) in s.params.iter() {
                // special handling for stereo param
                match k {
                    SynthParameter::ChannelPosition => {
                        if output_mode == OutputMode::Stereo {
                            let pos = (*v + 1.0) * 0.5;
                            inst.set_instance_parameter(*k, pos);
                        } else {
                            inst.set_instance_parameter(*k, *v);
                        }
                    }
                    // convert milliseconds to seconds
                    SynthParameter::Duration => inst.set_instance_parameter(*k, *v * 0.001),
                    SynthParameter::Attack => inst.set_instance_parameter(*k, *v * 0.001),
                    SynthParameter::Sustain => inst.set_instance_parameter(*k, *v * 0.001),
                    SynthParameter::Release => inst.set_instance_parameter(*k, *v * 0.001),
                    _ => inst.set_instance_parameter(*k, *v),
                }
            }
            ruffbox.trigger(inst);
        } else {
            println!("can't prepare this instance !");
        }
    }
}
