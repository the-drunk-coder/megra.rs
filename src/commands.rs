use std::{fs, sync, path::Path, collections::{HashSet, HashMap}};
use parking_lot::Mutex;

use ruffbox_synth::ruffbox::{Ruffbox, synth::SynthParameter};

use crate::sample_set::SampleSet;
use crate::builtin_types::*;
use crate::parameter::*;

pub fn load_sample<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
							   sample_set: &sync::Arc<Mutex<SampleSet>>,
							   set:String,
							   keywords: &mut Vec<String>,
							   path: String) {

    let mut sample_buffer:Vec<f32> = Vec::new();
    let mut reader = claxon::FlacReader::open(path.clone()).unwrap();
    
    let mut duration = if let Some(samples) = reader.streaminfo().samples {
	let tmp_dur = 1000.0 * ((samples as f32 / reader.streaminfo().channels as f32) / reader.streaminfo().sample_rate as f32);
	tmp_dur as usize	    	    
    } else {
	200
    };
    
    // max ten seconds
    if duration > 10000 {
	duration = 10000;
    }

    println!("sample path: {} channels: {} dur: {}", path, reader.streaminfo().channels, duration);
        
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

pub fn load_sample_set<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
							       sample_set: &sync::Arc<Mutex<SampleSet>>,
							       samples_path: &Path) {

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
	for entry in entries {
	    if let Ok(entry) = entry {
		let path = entry.path();
		// only consider files here ...
		if path.is_file() {
		    if let Some(ext) = path.extension() {
			if ext == "flac" {
			    load_sample(ruffbox, sample_set, set_name.clone(), &mut Vec::new(), path.to_str().unwrap().to_string());
			}
		    }
		}
	    }	    
	}
    }
}
pub fn load_sample_set_string<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
								      sample_set: &sync::Arc<Mutex<SampleSet>>,
								      samples_path: String) {
    let path = Path::new(&samples_path);
    load_sample_set(ruffbox, sample_set, path);
}

pub fn load_sample_sets<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
								sample_set: &sync::Arc<Mutex<SampleSet>>,
								folder_path: String) {

    let root_path = Path::new(&folder_path);
    load_sample_sets_path(ruffbox, sample_set, root_path);
}

pub fn load_sample_sets_path<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
								     sample_set: &sync::Arc<Mutex<SampleSet>>,
								     root_path: &Path) {

    if let Ok(entries) = fs::read_dir(root_path) {
	for entry in entries {
	    if let Ok(entry) = entry {
		let path = entry.path();
		if path.is_dir() {
		    load_sample_set(ruffbox, sample_set, &path);
		}
	    }	    
	}
    }
}

pub fn load_part(parts_store: &sync::Arc<Mutex<PartsStore>>, name: String, part: Part) {    
    let mut ps = parts_store.lock();
    ps.insert(name, part);
}

pub fn set_global_tmod(global_parameters: &sync::Arc<GlobalParameters>, p: Parameter) {    
    global_parameters.insert(BuiltinGlobalParameters::GlobalTimeModifier,
			     ConfigParameter::Dynamic(p)); // init on first attempt 
}

pub fn set_global_lifemodel_resources(global_parameters: &sync::Arc<GlobalParameters>, val: f32) {    
    global_parameters.insert(BuiltinGlobalParameters::LifemodelGlobalResources,
			     ConfigParameter::Numeric(val)); // init on first attempt 
}

pub fn set_global_ruffbox_parameters<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
									     params: &HashMap<SynthParameter, f32>) {
    let mut rb = ruffbox.lock();
    for (k,v) in params.iter() {
	rb.set_master_parameter(*k,*v);
    }
}
