use std::{sync, collections::HashSet};
use parking_lot::Mutex;

use ruffbox_synth::ruffbox::Ruffbox;

use crate::builtin_types::*;
use crate::session::Session;

pub fn interpret<const BUFSIZE:usize, const NCHAN:usize>(session: &mut Session<BUFSIZE, NCHAN>,
							 sample_set: &mut SampleSet,
							 parsed_in: Expr,
							 ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>) {
    match parsed_in {
	Expr::Constant(Atom::Generator(g)) => {	    
	    print!("a generator called \'");
	    for tag in g.id_tags.iter() {
		print!("{} ", tag);
	    }
	    println!("\'");
	},
	Expr::Constant(Atom::Parameter(_)) => {	    
	    println!("a parameter");
	},
	Expr::Constant(Atom::Event(_)) => {	    
	    println!("an event");
	},
	Expr::Constant(Atom::GeneratorModifierFunction(_)) => {	    
	    println!("a gen mod fun");
	},
	Expr::Constant(Atom::GeneratorProcessor(_)) => {	    
	    println!("a gen proc");
	},
	Expr::Constant(Atom::GeneratorList(gl)) => {	    
	    println!("a gen list");
	    for gen in gl.iter() {
		print!("--- a generator called \'");
		for tag in gen.id_tags.iter() {
		    print!("{} ", tag);
		}
		println!("\'");
	    }	    	    
	}
	Expr::Constant(Atom::SyncContext(mut s)) => {
	    let name = s.name.clone();
	    for c in s.generators.drain(..){
		session.start_generator(Box::new(c), sync::Arc::clone(&ruffbox));
	    }				
	    println!("a context called \'{}\'", name);
	},
	Expr::Constant(Atom::Command(c)) => {
	    match c {
		Command::Clear => {
		    session.clear_session();
		    println!("a command (stop session)");
		},
		Command::LoadSample((set, mut keywords, path)) => {
		    
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
	Expr::Constant(Atom::Float(f)) => {
	    println!("a number: {}", f)
	},		    
	_ => println!("unknown")
    }
}
