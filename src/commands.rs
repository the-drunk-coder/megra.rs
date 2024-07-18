use dashmap::DashMap;
use parking_lot::Mutex;
use rosc::OscType;

use std::env::temp_dir;
use std::fs::File;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
    sync,
};

// not sure why the author deprecated the sync version,
// as I really don't see the point of using async here at
// the moment ...
#[allow(deprecated)]
use sha256::try_digest;

use vom_rs::pfa;

use ruffbox_synth::{
    building_blocks::SynthParameterLabel, building_blocks::SynthParameterValue,
    helpers::wavetableize::*, ruffbox::RuffboxControls,
};

use crate::builtin_types::*;
use crate::commands;
use crate::event::*;
use crate::event_helpers::*;
use crate::generator::*;
use crate::load_audio_file;
use crate::osc_sender::OscSender;
use crate::parameter::*;
use crate::parser::eval::{self};
use crate::parser::FunctionMap;
use crate::real_time_streaming;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::*;
use chrono::Local;
use directories_next::ProjectDirs;
use std::io::{prelude::*, BufReader, Cursor};
use std::sync::atomic::Ordering;

use std::io;

// helper method ...
fn fetch_url(url: String, file_name: String) -> anyhow::Result<()> {
    let response = reqwest::blocking::get(url)?;
    let mut file = std::fs::File::create(file_name)?;
    let mut content = Cursor::new(response.bytes()?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

#[allow(deprecated)]
pub fn fetch_sample_set<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: SampleAndWavematrixSet,
    base_dir: String,
    resource: SampleResource,
) {
    let (fname, checksum) = match resource {
        SampleResource::File(fpath, cs) => (std::path::Path::new(&fpath).to_path_buf(), cs),
        SampleResource::Url(url, cs) => {
            println!("downlading sample set from {url}");
            // tmp file for download ...
            let tmp_dl = temp_dir().join("download.zip");
            fetch_url(url, tmp_dl.display().to_string()).unwrap();
            (tmp_dl, cs)
        }
    };

    if let Some(cs) = checksum {
        let val = try_digest(fname.as_path()).unwrap();
        if val != cs {
            // not loading anything, checksum error ...
            println!("sample set archive has invalid checksum, deleting ...");
            std::fs::remove_file(fname.as_path()).unwrap();
            return;
        } else {
            println!("sample set archive has valid checksum, loading ...");
        }
    }

    let file = fs::File::open(fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        let sample_path = Path::new(&base_dir).join("samples");

        // file name without enclosing zip folder ...
        let mut file_comp = file.enclosed_name().unwrap().components();
        file_comp.next();
        let file_name = file_comp.as_path();
        let file_path = sample_path.join(file_name);

        if (*file.name()).ends_with('/') {
            if !file_path.exists() {
                println!("Folder {} extracted to \"{}\"", i, file_path.display());
                fs::create_dir_all(&file_path).unwrap();
            } else {
                println!("Folder {} already exists ...", file_path.display());
            }
        } else if !file_path.exists() {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                file_path.display(),
                file.size()
            );
            if let Some(p) = file_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }

            // load sample
            let mut cmp = file_name.components();
            // only load wav or flac ...
            // will fail if zip file isn't in the right format ...
            let load_data = if (*file.name()).ends_with("flac") || (*file.name()).ends_with("wav") {
                let set: String = cmp
                    .next()
                    .unwrap()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string();
                let keyword: Vec<String> = vec![cmp
                    .as_path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()];
                Some((set, keyword))
            } else {
                None
            };

            let mut outfile = fs::File::create(&file_path).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
            if let Some((set, mut keyword)) = load_data {
                load_sample(
                    function_map,
                    ruffbox,
                    sample_set.clone(),
                    set,
                    &mut keyword,
                    file_path.display().to_string(),
                    false,
                );
            }
        } else {
            // don't overwrite files ...
            println!("can't extract file, probably already exists ...");
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&file_path, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }
}

pub fn freeze_buffer<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    freezbuf: usize,
    inbuf: usize,
) {
    ruffbox.freeze_buffer(freezbuf, inbuf);
}

pub fn load_sample_as_wavematrix(
    mut sample_set: SampleAndWavematrixSet,
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
            wavematrix[x].push(DynVal::with_value(wavematrix_raw[x][y]));
        }
    }

    sample_set.insert_wavematrix(key, wavematrix);
}

pub fn load_sample<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    mut sample_set: SampleAndWavematrixSet,
    set: String,
    keywords: &mut Vec<String>,
    path: String,
    downmix_stereo: bool,
) {
    if let Some((mut duration, samplerate, channels, mut sample_buffer)) = if path
        .as_str()
        .to_lowercase()
        .as_str()
        .trim()
        .ends_with(".flac")
    {
        load_audio_file::load_flac(&path, ruffbox.samplerate)
    } else if path
        .as_str()
        .to_lowercase()
        .as_str()
        .trim()
        .ends_with(".wav")
    {
        load_audio_file::load_wav(&path, ruffbox.samplerate)
    } else {
        None
    } {
        // max duration ten seconds
        if duration > 10000 {
            duration = 10000;
        }

        // downmix
        let bufnum = if channels != 1 {
            if channels == 2 && !downmix_stereo {
                // load stereo sample
                let mut left = Vec::new();
                let mut right = Vec::new();
                let mut frames = sample_buffer.chunks_exact(channels.try_into().unwrap());
                while let Some([l, r]) = frames.next() {
                    left.push(*l);
                    right.push(*r);
                }
                ruffbox.load_stereo_sample(&mut left, &mut right, true, samplerate)
            } else {
                // downmix to mono (default case)
                let mut downmix_buffer = sample_buffer
                    .chunks(channels.try_into().unwrap())
                    .map(|x| x.iter().sum::<f32>() / channels as f32)
                    .collect();
                ruffbox.load_mono_sample(&mut downmix_buffer, true, samplerate)
            }
        } else {
            // load mono as-is
            ruffbox.load_mono_sample(&mut sample_buffer, true, samplerate)
        };

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

        sample_set.insert(set.clone(), keyword_set, bufnum, duration);
        function_map
            .lock()
            .std_lib // add sample functions to std lib for now ...
            .insert(set, eval::events::sound::sound);
    } else {
        println!("can't load sample {path}");
    }
}

pub fn load_sample_set<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: SampleAndWavematrixSet,
    samples_path: &Path,
    downmix_stereo: bool,
) {
    // determine set name or use default
    let set_name = if let Some(os_filename) = samples_path.file_stem() {
        if let Some(str_filename) = os_filename.to_str() {
            if str_filename.chars().next().unwrap().is_numeric() {
                let mut owned_string: String = "_".to_owned();
                owned_string.push_str(str_filename);
                owned_string
            } else {
                str_filename.to_string()
            }
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
                    if let Ok(ext_str) = ext.to_os_string().into_string() {
                        let ext_str_lc = ext_str.as_str().to_lowercase();
                        if ext_str_lc == "flac" || ext_str_lc == "wav" {
                            load_sample(
                                function_map,
                                ruffbox,
                                sample_set.clone(),
                                set_name.clone(),
                                &mut Vec::new(),
                                path.to_str().unwrap().to_string(),
                                downmix_stereo,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn load_sample_set_string<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: SampleAndWavematrixSet,
    samples_path: String,
    downmix_stereo: bool,
) {
    let path = Path::new(&samples_path);
    load_sample_set(function_map, ruffbox, sample_set, path, downmix_stereo);
}

pub fn load_sample_sets<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: SampleAndWavematrixSet,
    folder_path: String,
    downmix_stereo: bool,
) {
    let root_path = Path::new(&folder_path);
    load_sample_sets_path(function_map, ruffbox, sample_set, root_path, downmix_stereo);
}

pub fn load_sample_sets_path<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    sample_set: SampleAndWavematrixSet,
    root_path: &Path,
    downmix_stereo: bool,
) {
    let mut exclude_file_path_buf = root_path.to_path_buf();
    exclude_file_path_buf.push("exclude");
    exclude_file_path_buf.set_extension("txt");

    let exclude_path = exclude_file_path_buf.as_path();

    let mut excludes: HashSet<String> = HashSet::new();

    if exclude_path.exists() && exclude_path.is_file() {
        let f = File::open(exclude_path).unwrap();
        let mut reader = BufReader::new(f);
        let mut line = String::new(); // may also use with_capacity if you can guess
        while reader.read_line(&mut line).unwrap() > 0 {
            // do something with line
            if !line.starts_with('#') {
                excludes.insert(line.trim().to_string());
            }

            line.clear(); // clear to reuse the buffer
        }
    }

    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let foldername = path.file_name().unwrap().to_str().unwrap().to_string();
            if path.is_dir() && !excludes.contains(&foldername) {
                load_sample_set(
                    function_map,
                    ruffbox,
                    sample_set.clone(),
                    &path,
                    downmix_stereo,
                );
            }
        }
    };
}

/// start a recording of the output
pub fn start_recording<const BUFSIZE: usize, const NCHAN: usize>(
    session: &Session<BUFSIZE, NCHAN>,
    prefix: Option<String>,
    base_dir: String,
    rec_input: bool,
) {
    let maybe_rec_ctrl = session.rec_control.lock().take();
    if let Some(mut rec_ctrl) = maybe_rec_ctrl {
        //println!("rec state {} {}", rec_ctrl.is_recording_output.load(Ordering::SeqCst) ,rec_ctrl.is_recording_input.load(Ordering::SeqCst));

        // OUTPUT RECORDING
        if rec_ctrl.is_recording_output.load(Ordering::SeqCst) {
            println!("there's already a recording in progress, please stop first !");
        } else {
            let maybe_catch = rec_ctrl.catch_out.take();
            //println!("catch none ? {}", maybe_catch.is_none());
            if let Some(catch_out) = maybe_catch {
                // place in recordings folder

                let id = if let Some(p) = prefix.clone() {
                    format!("{}_{}_output.wav", p, Local::now().format("%Y%m%d_%H%M_%S"))
                } else {
                    format!(
                        "megra_recording_{}_output.wav",
                        Local::now().format("%Y%m%d_%H%M_%S")
                    )
                };

                let recordings_path = Path::new(&base_dir).join("recordings");

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
            } else {
                println!("can't get catch");
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

        *session.rec_control.lock() = Some(rec_ctrl);
    }
}

/// stop a running recording
pub fn stop_recording<const BUFSIZE: usize, const NCHAN: usize>(session: &Session<BUFSIZE, NCHAN>) {
    let maybe_rec_ctrl = session.rec_control.lock().take();
    if let Some(mut rec_ctrl) = maybe_rec_ctrl {
        //println!("rec state {} {}", rec_ctrl.is_recording_output.load(Ordering::SeqCst), rec_ctrl.is_recording_input.load(Ordering::SeqCst));
        if rec_ctrl.is_recording_output.load(Ordering::SeqCst) {
            let maybe_catch_handle = rec_ctrl.catch_out_handle.take();
            if let Some(catch_handle) = maybe_catch_handle {
                rec_ctrl.is_recording_output.store(false, Ordering::SeqCst);
                rec_ctrl.catch_out = Some(real_time_streaming::stop_writer_thread(catch_handle));
            }
        } else {
            println!("can't stop output recording that isn't running !");
        }
        if rec_ctrl.is_recording_input.load(Ordering::SeqCst) {
            let maybe_catch_handle = rec_ctrl.catch_in_handle.take();
            if let Some(catch_handle) = maybe_catch_handle {
                rec_ctrl.is_recording_input.store(false, Ordering::SeqCst);
                rec_ctrl.catch_in = Some(real_time_streaming::stop_writer_thread(catch_handle));
            }
        } else {
            println!("can't stop input recording that isn't running !");
        }

        *session.rec_control.lock() = Some(rec_ctrl);
    }
}

/// execute a pre-defined part step by step
pub fn step_part<const BUFSIZE: usize, const NCHAN: usize>(
    session: &Session<BUFSIZE, NCHAN>,
    part_name: String,
) {
    let mut sound_events = Vec::new();
    let mut control_events = Vec::new();
    if let Some(mut thing) = session.globals.get_mut(&VariableId::Custom(part_name)) {
        if let TypedEntity::GeneratorList(ref mut gens) = thing.value_mut() {
            for gen in gens.iter_mut() {
                gen.current_transition(&session.globals);
                let mut current_events = gen.current_events(&session.globals);
                for ev in current_events.drain(..) {
                    match ev {
                        InterpretableEvent::Control(c) => control_events.push(c),
                        InterpretableEvent::Sound(s) => sound_events.push(s),
                    }
                }
            }
        } else if let TypedEntity::Generator(ref mut gen) = thing.value_mut() {
            gen.current_transition(&session.globals);
            let mut current_events = gen.current_events(&session.globals);
            for ev in current_events.drain(..) {
                match ev {
                    InterpretableEvent::Control(c) => control_events.push(c),
                    InterpretableEvent::Sound(s) => sound_events.push(s),
                }
            }
        }
    }

    // execute retrieved events
    once(session, &mut sound_events, &control_events);
}

pub fn set_global_tmod(globals: &sync::Arc<GlobalVariables>, p: DynVal) {
    globals.insert(
        VariableId::GlobalTimeModifier,
        TypedEntity::ConfigParameter(ConfigParameter::Dynamic(p)),
    ); // init on first attempt
}

pub fn set_global_latency(globals: &sync::Arc<GlobalVariables>, p: DynVal) {
    globals.insert(
        VariableId::GlobalLatency,
        TypedEntity::ConfigParameter(ConfigParameter::Dynamic(p)),
    ); // init on first attempt
}

pub fn set_default_duration(globals: &sync::Arc<GlobalVariables>, n: f32) {
    globals.insert(
        VariableId::DefaultDuration,
        TypedEntity::ConfigParameter(ConfigParameter::Numeric(n)),
    ); // init on first attempt
}

pub fn set_global_lifemodel_resources(globals: &sync::Arc<GlobalVariables>, val: f32) {
    globals.insert(
        VariableId::LifemodelGlobalResources,
        TypedEntity::ConfigParameter(ConfigParameter::Numeric(val)),
    ); // init on first attempt
}

pub fn set_global_ruffbox_parameters<const BUFSIZE: usize, const NCHAN: usize>(
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    globals: &sync::Arc<GlobalVariables>,
    params: &mut HashMap<SynthParameterLabel, ParameterValue>,
) {
    for (k, v) in params.iter_mut() {
        ruffbox.set_master_parameter(*k, resolve_parameter(*k, v, globals))
    }
}

pub fn export_dot_static(filename: &str, generator: &Generator) {
    let dot_string = pfa::to_dot::<char>(&generator.root_generator.generator);
    println!("export to {filename}");
    fs::write(filename, dot_string).expect("Unable to write file");
}

pub fn export_dot_running<const BUFSIZE: usize, const NCHAN: usize>(
    filename: &str,
    tags: &BTreeSet<String>,
    session: &Session<BUFSIZE, NCHAN>,
) {
    let mut gens = Vec::new();

    for sc in session.schedulers.iter() {
        let (id_tags, (_, data)) = sc.pair();

        if !tags.is_disjoint(id_tags) {
            // get a snapshot of the generator in it's current state
            gens.push((id_tags.clone(), data.generator.lock().clone()));
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
        println!("export to {filename_tagged}");
        fs::write(filename_tagged, dot_string).expect("Unable to write file");
    }
}

pub fn once<const BUFSIZE: usize, const NCHAN: usize>(
    session: &Session<BUFSIZE, NCHAN>,
    sound_events: &mut [StaticEvent],
    control_events: &[ControlEvent],
) {
    for cev in control_events.iter() {
        if let Some(mut contexts) = cev.ctx.clone() {
            // this is the worst clone ....
            for mut sx in contexts.drain(..) {
                Session::handle_context(&mut sx, session);
            }
        }
        if let Some(mut commands) = cev.cmd.clone() {
            // this is the worst clone ....
            // some code duplication from session.rs,
            // should be unified at some point ...
            for c in commands.drain(..) {
                match c {
                    Command::FreezeBuffer(freezbuf, inbuf) => {
                        commands::freeze_buffer(&session.ruffbox, freezbuf, inbuf);
                        //println!("freeze buffer");
                    }
                    Command::Tmod(p) => {
                        commands::set_global_tmod(&session.globals, p);
                    }
                    Command::Bpm(b) => {
                        commands::set_default_duration(&session.globals, b);
                    }
                    Command::StepPart(p) => {
                        commands::step_part(session, p);
                    }
                    Command::GlobRes(v) => {
                        commands::set_global_lifemodel_resources(&session.globals, v);
                    }
                    Command::GlobalRuffboxParams(mut m) => {
                        commands::set_global_ruffbox_parameters(
                            &session.ruffbox,
                            &session.globals,
                            &mut m,
                        );
                    }
                    Command::Clear => {
                        let session2 = session.clone();
                        Session::clear_session(session2);
                        println!("a command (stop session)");
                    }
                    Command::Once(mut s, c) => {
                        //println!("handle once from gen");
                        commands::once(session, &mut s, &c);
                    }
                    Command::OscSendMessage(client_name, osc_addr, args) => {
                        let mut osc_args = Vec::new();
                        for arg in args.iter() {
                            match arg {
                                TypedEntity::Comparable(Comparable::Float(n)) => {
                                    osc_args.push(OscType::Float(*n))
                                }
                                TypedEntity::Comparable(Comparable::Double(n)) => {
                                    osc_args.push(OscType::Double(*n))
                                }
                                TypedEntity::Comparable(Comparable::Int32(n)) => {
                                    osc_args.push(OscType::Int(*n))
                                }
                                TypedEntity::Comparable(Comparable::Int64(n)) => {
                                    osc_args.push(OscType::Long(*n))
                                }
                                TypedEntity::Comparable(Comparable::String(s)) => {
                                    osc_args.push(OscType::String(s.to_string()))
                                }
                                TypedEntity::Comparable(Comparable::Symbol(s)) => {
                                    osc_args.push(OscType::String(s.to_string()))
                                }
                                _ => {}
                            }
                        }
                        if let Some(thing) = &session.osc_client.custom.get(&client_name) {
                            let _ = thing.value().send_message(osc_addr, osc_args);
                        }
                    }
                    Command::Print(te) => {
                        println!("{te:#?}");
                    }
                    _ => {
                        println!("ignore command")
                    }
                };
            }
        }
    }

    for s in sound_events.iter_mut() {
        if s.name == "silence" {
            continue;
        }

        // if this is a sampler event and contains a sample lookup,
        // resolve it NOW ... at the very end, finally ...
        let mut bufnum: usize = 0;
        if let Some(lookup) = s.sample_lookup.as_ref() {
            if let Some((res_bufnum, duration)) = session.sample_set.resolve_lookup(lookup) {
                bufnum = res_bufnum;
                // is this really needed ??
                s.params.insert(
                    SynthParameterLabel::SampleBufferNumber.into(),
                    SynthParameterValue::ScalarUsize(bufnum),
                );

                s.params
                    .entry(SynthParameterLabel::Sustain.into())
                    .or_insert_with(|| SynthParameterValue::ScalarF32((duration - 2) as f32));
            }
        }

        // prepare a single, self-contained envelope from
        // the available information ...
        s.build_envelope();

        // latency 0.05, should be made configurable later ...
        if let Some(mut inst) =
            session
                .ruffbox
                .prepare_instance(map_synth_type(&s.name, &s.params), 0.0, bufnum)
        {
            // set parameters and trigger instance
            for (addr, v) in s.params.iter() {
                // special handling for stereo param
                match addr.label {
                    SynthParameterLabel::ChannelPosition => {
                        if session.output_mode == OutputMode::Stereo {
                            inst.set_instance_parameter(*addr, &translate_stereo(v.clone()));
                        } else {
                            inst.set_instance_parameter(*addr, v);
                        }
                    }
                    // convert milliseconds to seconds
                    SynthParameterLabel::Duration => {
                        if let SynthParameterValue::ScalarF32(val) = v {
                            inst.set_instance_parameter(
                                *addr,
                                &SynthParameterValue::ScalarF32(*val * 0.001),
                            )
                        }
                    }
                    SynthParameterLabel::Attack => {
                        if let SynthParameterValue::ScalarF32(val) = v {
                            inst.set_instance_parameter(
                                *addr,
                                &SynthParameterValue::ScalarF32(*val * 0.001),
                            )
                        }
                    }
                    SynthParameterLabel::Sustain => {
                        if let SynthParameterValue::ScalarF32(val) = v {
                            inst.set_instance_parameter(
                                *addr,
                                &SynthParameterValue::ScalarF32(*val * 0.001),
                            )
                        }
                    }
                    SynthParameterLabel::Release => {
                        if let SynthParameterValue::ScalarF32(val) = v {
                            inst.set_instance_parameter(
                                *addr,
                                &SynthParameterValue::ScalarF32(*val * 0.001),
                            )
                        }
                    }
                    _ => inst.set_instance_parameter(*addr, v),
                }
            }
            session.ruffbox.trigger(inst);
        } else {
            println!("can't prepare this instance !");
        }
    }
}

pub fn define_osc_client(
    name: String,
    target: String,
    host: String,
    clients: &sync::Arc<DashMap<String, OscSender>>,
) {
    if let Ok(cl) = OscSender::start(target, host) {
        clients.insert(name, cl);
    }
}

pub fn push(id: VariableId, value: TypedEntity, globals: &sync::Arc<GlobalVariables>) {
    if let Some(mut thing) = globals.get_mut(&id) {
        if let TypedEntity::Vec(v) = thing.value_mut() {
            v.push(Box::new(value));
        }
    }
}

pub fn insert(
    id: VariableId,
    key: VariableId,
    value: TypedEntity,
    globals: &sync::Arc<GlobalVariables>,
) {
    if let Some(mut thing) = globals.get_mut(&id) {
        if let TypedEntity::Map(m) = thing.value_mut() {
            m.insert(key, value);
        }
    }
}
