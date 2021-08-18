use parking_lot::Mutex;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync,
};
use vom_rs::pfa;

use ruffbox_synth::ruffbox::{synth::SynthParameter, Ruffbox};

use crate::builtin_types::*;
use crate::event::*;
use crate::event_helpers::*;
use crate::generator::*;
use crate::parameter::*;
use crate::sample_set::SampleSet;
use crate::session::*;

pub fn freeze_buffer<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
    freezbuf: usize,
) {
    let mut ruff = ruffbox.lock();
    ruff.freeze_buffer(freezbuf);
}

pub fn load_sample<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
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

    println!(
        "sample path: {} channels: {} dur: {}",
        path,
        reader.streaminfo().channels,
        duration
    );

    // decode to f32
    let max_val = (i32::MAX >> (32 - reader.streaminfo().bits_per_sample)) as f32;
    sample_buffer.push(0.0); // interpolation sample
    for sample in reader.samples() {
        let s = sample.unwrap() as f32 / max_val;
        sample_buffer.push(s);
    }
    sample_buffer.push(0.0); // interpolation sample
    sample_buffer.push(0.0); // interpolation sample

    let mut ruff = ruffbox.lock();
    let bufnum = ruff.load_sample(&sample_buffer);

    let mut keyword_set = HashSet::new();
    for k in keywords.drain(..) {
        keyword_set.insert(k);
    }

    let path = Path::new(&path);
    if let Some(os_filename) = path.file_stem() {
        if let Some(str_filename) = os_filename.to_str() {
            let tokens = str_filename.split(|c| c == ' ' || c == '_' || c == '-' || c == '.');
            for token in tokens {
                keyword_set.insert(token.to_lowercase().to_string());
            }
        }
    }

    sample_set.lock().insert(set, keyword_set, bufnum, duration);
}

pub fn load_sample_set<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
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
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    samples_path: String,
) {
    let path = Path::new(&samples_path);
    load_sample_set(ruffbox, sample_set, path);
}

pub fn load_sample_sets<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    folder_path: String,
) {
    let root_path = Path::new(&folder_path);
    load_sample_sets_path(ruffbox, sample_set, root_path);
}

pub fn load_sample_sets_path<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    root_path: &Path,
) {
    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                load_sample_set(ruffbox, sample_set, &path);
            }
        }
    }
}

pub fn load_part(parts_store: &sync::Arc<Mutex<PartsStore>>, name: String, part: Part) {
    let mut ps = parts_store.lock();
    ps.insert(name, part);
}

pub fn set_global_tmod(global_parameters: &sync::Arc<GlobalParameters>, p: Parameter) {
    global_parameters.insert(
        BuiltinGlobalParameters::GlobalTimeModifier,
        ConfigParameter::Dynamic(p),
    ); // init on first attempt
}

pub fn set_global_lifemodel_resources(global_parameters: &sync::Arc<GlobalParameters>, val: f32) {
    global_parameters.insert(
        BuiltinGlobalParameters::LifemodelGlobalResources,
        ConfigParameter::Numeric(val),
    ); // init on first attempt
}

pub fn set_global_ruffbox_parameters<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
    params: &HashMap<SynthParameter, f32>,
) {
    let mut rb = ruffbox.lock();
    for (k, v) in params.iter() {
        rb.set_master_parameter(*k, *v);
    }
}

pub fn export_dot(filename: &str, generator: &Generator) {
    let dot_string = pfa::to_dot::<char>(&generator.root_generator.generator);
    println!("export to {}", filename);
    fs::write(filename, dot_string).expect("Unable to write file");
}

pub fn once<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
    global_parameters: &sync::Arc<GlobalParameters>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    sound_events: &mut Vec<Event>,
    control_events: &mut Vec<ControlEvent>,
    output_mode: OutputMode,
) {
    for cev in control_events.iter() {
        if let Some(mut contexts) = cev.ctx.clone() {
            // this is the worst clone ....
            for mut sx in contexts.drain(..) {
                Session::handle_context(&mut sx, session, ruffbox, parts_store, global_parameters, output_mode);
            }
        }
    }

    for sev in sound_events.iter_mut() {
        let s = sev.get_static();

        if s.name == "silence" {
            continue;
        }

        let mut bufnum: usize = 0;
        if let Some(b) = s.params.get(&SynthParameter::SampleBufferNumber) {
            bufnum = *b as usize;
        }

        let mut ruff = ruffbox.lock();
        // latency 0.05, should be made configurable later ...
        let inst = ruff.prepare_instance(map_name(&s.name), 0.0, bufnum);
        // set parameters and trigger instance
        for (k, v) in s.params.iter() {
            // special handling for stereo param
            match k {
                SynthParameter::ChannelPosition => {
                    if output_mode == OutputMode::Stereo {
                        let pos = (*v + 1.0) * 0.5;
                        ruff.set_instance_parameter(inst, *k, pos);
                    } else {
                        ruff.set_instance_parameter(inst, *k, *v);
                    }
                }
                // convert milliseconds to seconds
                SynthParameter::Duration => ruff.set_instance_parameter(inst, *k, *v * 0.001),
                SynthParameter::Attack => ruff.set_instance_parameter(inst, *k, *v * 0.001),
                SynthParameter::Sustain => ruff.set_instance_parameter(inst, *k, *v * 0.001),
                SynthParameter::Release => ruff.set_instance_parameter(inst, *k, *v * 0.001),
                _ => ruff.set_instance_parameter(inst, *k, *v),
            }
        }
        ruff.trigger(inst);
    }
}
