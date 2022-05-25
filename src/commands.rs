use parking_lot::Mutex;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
    sync,
};
use vom_rs::pfa;

use ruffbox_synth::{
    building_blocks::SynthParameterLabel, building_blocks::SynthParameterValue,
    helpers::wavetableize::*, ruffbox::RuffboxControls,
};

use crate::builtin_types::*;
use crate::event::*;
use crate::event_helpers::*;
use crate::generator::*;
use crate::load_audio_file;
use crate::parameter::*;
use crate::parser::eval;
use crate::parser::FunctionMap;
use crate::real_time_streaming;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::*;
use chrono::Local;
use directories_next::ProjectDirs;
use std::sync::atomic::Ordering;

pub fn freeze_buffer<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    freezbuf: usize,
    inbuf: usize,
) {
    ruffbox.freeze_buffer(freezbuf, inbuf);
}

pub fn load_sample_as_wavematrix(
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    key: String,
    path: String,
    method: &str,
    matrix_size: (usize, usize),
    start: f32,
) {
    let mut sample_buffer: Vec<f32> = Vec::new();
    let mut reader = claxon::FlacReader::open(path).unwrap();

    // decode to f32
    let max_val = (i32::MAX >> (32 - reader.streaminfo().bits_per_sample)) as f32;
    for sample in reader.samples() {
        let s = sample.unwrap() as f32 / max_val;
        sample_buffer.push(s);
    }

    let wavematrix_raw = match method {
        "raw" => wavetableize(&sample_buffer, matrix_size, start, WavetableizeMethod::Raw),
        "smooth" => wavetableize(
            &sample_buffer,
            matrix_size,
            start,
            WavetableizeMethod::Smooth,
        ),
        "supersmooth" => wavetableize(
            &sample_buffer,
            matrix_size,
            start,
            WavetableizeMethod::Supersmooth,
        ),
        _ => wavetableize(
            &sample_buffer,
            matrix_size,
            start,
            WavetableizeMethod::Supersmooth,
        ),
    };

    let mut wavematrix = Vec::new();

    // turn into parameters
    for x in 0..wavematrix_raw.len() {
        wavematrix.push(Vec::new());
        for y in 0..wavematrix_raw[x].len() {
            wavematrix[x].push(Parameter::with_value(wavematrix_raw[x][y]));
        }
    }

    sample_set.lock().insert_wavematrix(key, wavematrix);
}

pub fn load_sample<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    set: String,
    keywords: &mut Vec<String>,
    path: String,
) {
    let (mut duration, samplerate, channels, mut sample_buffer) =
        load_audio_file::load_flac(&path, ruffbox.samplerate);

    // max ten seconds
    if duration > 10000 {
        duration = 10000;
    }

    // adds interpolation samples to sample buffer, don't use afterwards
    let bufnum = ruffbox.load_sample(&mut sample_buffer, true, samplerate);

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

    println!(
        "sample path: {} channels: {} dur: {} orig sr: {} ruf sr: {} resampled: {}",
        path,
        channels,
        duration,
        samplerate,
        ruffbox.samplerate,
        samplerate != ruffbox.samplerate
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
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
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
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    samples_path: String,
) {
    let path = Path::new(&samples_path);
    load_sample_set(function_map, ruffbox, sample_set, path);
}

pub fn load_sample_sets<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    folder_path: String,
) {
    let root_path = Path::new(&folder_path);
    load_sample_sets_path(function_map, ruffbox, sample_set, root_path);
}

pub fn load_sample_sets_path<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
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

/// start a recording of the output
pub fn start_recording<const BUFSIZE: usize, const NCHAN: usize>(
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    prefix: Option<String>,
    rec_input: bool,
) {
    let maybe_rec_ctrl = session.lock().rec_control.take();
    if let Some(mut rec_ctrl) = maybe_rec_ctrl {
        // OUTPUT RECORDING
        if rec_ctrl.is_recording_output.load(Ordering::SeqCst) {
            println!("there's already a recording in progress, please stop first !");
        } else {
            let maybe_catch = rec_ctrl.catch_out.take();
            if let Some(catch_out) = maybe_catch {
                // place in recordings folder
                if let Some(proj_dirs) = ProjectDirs::from("de", "parkellipsen", "megra") {
                    let id = if let Some(p) = prefix.clone() {
                        format!("{}_{}_output.wav", p, Local::now().format("%Y%m%d_%H%M_%S"))
                    } else {
                        format!(
                            "megra_recording_{}_output.wav",
                            Local::now().format("%Y%m%d_%H%M_%S")
                        )
                    };

                    let recordings_path = proj_dirs.config_dir().join("recordings");

                    let file_path = if recordings_path.exists() {
                        let path = recordings_path.join(id).into_os_string().into_string();
                        path.unwrap()
                    } else {
                        id
                    };

                    rec_ctrl.catch_out_handle = Some(real_time_streaming::start_writer_thread(
                        catch_out,
                        rec_ctrl.samplerate,
                        file_path,
                    ));
                    rec_ctrl.is_recording_output.store(true, Ordering::SeqCst);
                }
            }
        }
        // INPUT RECORDING
        // record input if desired ...
        if rec_input && !rec_ctrl.is_recording_input.load(Ordering::SeqCst) {
            let maybe_catch = rec_ctrl.catch_in.take();
            if let Some(catch_in) = maybe_catch {
                // place in recordings folder
                if let Some(proj_dirs) = ProjectDirs::from("de", "parkellipsen", "megra") {
                    let id = if let Some(p) = prefix {
                        format!("{}_{}_input.wav", p, Local::now().format("%Y%m%d_%H%M_%S"))
                    } else {
                        format!(
                            "megra_recording_{}_input.wav",
                            Local::now().format("%Y%m%d_%H%M_%S")
                        )
                    };

                    let recordings_path = proj_dirs.config_dir().join("recordings");

                    let file_path = if recordings_path.exists() {
                        let path = recordings_path.join(id).into_os_string().into_string();
                        path.unwrap()
                    } else {
                        id
                    };

                    rec_ctrl.catch_in_handle = Some(real_time_streaming::start_writer_thread(
                        catch_in,
                        rec_ctrl.samplerate,
                        file_path,
                    ));
                    rec_ctrl.is_recording_input.store(true, Ordering::SeqCst);
                }
            }
        }
        session.lock().rec_control = Some(rec_ctrl);
    }
}

/// stop a running recording
pub fn stop_recording<const BUFSIZE: usize, const NCHAN: usize>(
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
) {
    let maybe_rec_ctrl = session.lock().rec_control.take();
    if let Some(mut rec_ctrl) = maybe_rec_ctrl {
        if rec_ctrl.is_recording_output.load(Ordering::SeqCst) {
            let maybe_catch_handle = rec_ctrl.catch_out_handle.take();
            if let Some(catch_handle) = maybe_catch_handle {
                rec_ctrl.is_recording_output.store(false, Ordering::SeqCst);
                real_time_streaming::stop_writer_thread(catch_handle);
            }
        } else {
            println!("can't stop output recording that isn't running !");
        }
        if rec_ctrl.is_recording_input.load(Ordering::SeqCst) {
            let maybe_catch_handle = rec_ctrl.catch_in_handle.take();
            if let Some(catch_handle) = maybe_catch_handle {
                rec_ctrl.is_recording_input.store(false, Ordering::SeqCst);
                real_time_streaming::stop_writer_thread(catch_handle);
            }
        } else {
            println!("can't stop input recording that isn't running !");
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
    params: &mut HashMap<SynthParameterLabel, ParameterValue>,
) {
    for (k, v) in params.iter_mut() {
        match v {
            ParameterValue::Scalar(p) => {
                ruffbox.set_master_parameter(*k, p.evaluate_val_f32());
            }
            ParameterValue::Lfo(init, freq, range, op) => {
                ruffbox.set_master_parameter(
                    *k,
                    SynthParameterValue::Lfo(
                        init.evaluate_numerical(),
                        freq.evaluate_numerical(),
                        range.evaluate_numerical(),
                        *op,
                    ),
                );
            }
            _ => {}
        }
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
    sound_events: &mut [StaticEvent],
    control_events: &mut [ControlEvent],
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
        if let Some(SynthParameterValue::ScalarUsize(b)) =
            s.params.get(&SynthParameterLabel::SampleBufferNumber)
        {
            bufnum = *b;
        }

        // latency 0.05, should be made configurable later ...
        if let Some(mut inst) = ruffbox.prepare_instance(map_name(&s.name), 0.0, bufnum) {
            // set parameters and trigger instance
            for (k, v) in s.params.iter() {
                // special handling for stereo param
                match k {
                    SynthParameterLabel::ChannelPosition => match v {
                        SynthParameterValue::ScalarF32(p) => {
                            if output_mode == OutputMode::Stereo {
                                let pos = (*p + 1.0) * 0.5;
                                inst.set_instance_parameter(
                                    *k,
                                    &SynthParameterValue::ScalarF32(pos),
                                );
                            } else {
                                inst.set_instance_parameter(
                                    *k,
                                    &SynthParameterValue::ScalarF32(*p),
                                );
                            }
                        }
                        SynthParameterValue::Lfo(init, freq, range, op) => {
                            if output_mode == OutputMode::Stereo {
                                let pos = (*init + 1.0) * 0.5;
                                inst.set_instance_parameter(
                                    *k,
                                    &SynthParameterValue::Lfo(pos, *freq, *range, *op),
                                );
                            } else {
                                inst.set_instance_parameter(
                                    *k,
                                    &SynthParameterValue::Lfo(*init, *freq, *range, *op),
                                );
                            }
                        }
                        _ => {}
                    },
                    // convert milliseconds to seconds
                    SynthParameterLabel::Duration => {
                        if let SynthParameterValue::ScalarF32(val) = v {
                            inst.set_instance_parameter(
                                *k,
                                &SynthParameterValue::ScalarF32(*val * 0.001),
                            )
                        }
                    }
                    SynthParameterLabel::Attack => {
                        if let SynthParameterValue::ScalarF32(val) = v {
                            inst.set_instance_parameter(
                                *k,
                                &SynthParameterValue::ScalarF32(*val * 0.001),
                            )
                        }
                    }
                    SynthParameterLabel::Sustain => {
                        if let SynthParameterValue::ScalarF32(val) = v {
                            inst.set_instance_parameter(
                                *k,
                                &SynthParameterValue::ScalarF32(*val * 0.001),
                            )
                        }
                    }
                    SynthParameterLabel::Release => {
                        if let SynthParameterValue::ScalarF32(val) = v {
                            inst.set_instance_parameter(
                                *k,
                                &SynthParameterValue::ScalarF32(*val * 0.001),
                            )
                        }
                    }
                    _ => inst.set_instance_parameter(*k, v),
                }
            }
            ruffbox.trigger(inst);
        } else {
            println!("can't prepare this instance !");
        }
    }
}
