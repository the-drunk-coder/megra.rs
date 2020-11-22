use std::{sync, collections::HashSet};
use parking_lot::Mutex;

use ruffbox_synth::ruffbox::Ruffbox;

use crate::parser;
use crate::session::Session;

pub fn interpret(session: &mut Session, sample_set: &mut parser::SampleSet, parsed_in: parser::Expr, ruffbox: &sync::Arc<Mutex<Ruffbox<512>>>) {
    match parsed_in {
	parser::Expr::Constant(parser::Atom::Generator(g)) => {
	    let name = g.name.clone();
	    session.start_generator(Box::new(g), sync::Arc::clone(&ruffbox));
	    println!("a generator called \'{}\'", name);
	},
	parser::Expr::Constant(parser::Atom::SyncContext(mut s)) => {
	    let name = s.name.clone();
	    for c in s.generators.drain(..){
		session.start_generator(Box::new(c), sync::Arc::clone(&ruffbox));
	    }				
	    println!("a context called \'{}\'", name);
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
}
