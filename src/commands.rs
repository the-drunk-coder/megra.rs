use std::{sync, path::Path, collections::HashSet};
use parking_lot::Mutex;

use ruffbox_synth::ruffbox::Ruffbox;
use crate::builtin_types::*;
use crate::generator::Generator;

pub fn load_sample<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
							   sample_set: &mut SampleSet,
							   set:String,
							   keywords: &mut Vec<String>,
							   path: String) {

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

    let path = Path::new(&path);
    if let Some(os_filename) = path.file_stem() {
	if let Some(str_filename) = os_filename.to_str() {
	    keyword_set.insert(str_filename.to_string());
	}
    }
    
    sample_set.entry(set).or_insert(Vec::new()).push((keyword_set, bufnum));    
}

pub fn load_part(parts_store: &mut PartsStore, name: String, mut generators: Vec<Generator> ) {
    for gen in generators.iter_mut() {
	gen.id_tags.insert(name.clone());
    }
    parts_store.insert(name, generators);
}
    
