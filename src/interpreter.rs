use std::{sync, collections::HashSet};
use parking_lot::Mutex;

use ruffbox_synth::ruffbox::Ruffbox;

use crate::builtin_types::*;
use crate::session::Session;

pub fn interpret<const BUFSIZE:usize, const NCHAN:usize>(parsed_in: Expr,							 
							 session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
							 ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
							 global_parameters: &sync::Arc<GlobalParameters>,
							 sample_set: &mut SampleSet,
							 parts_store: &mut PartsStore) {
    match parsed_in {
	Expr::Comment => {println!("a comment")},
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
	Expr::Constant(Atom::SoundEvent(_)) => {	    
	    println!("a sound event");
	},
	Expr::Constant(Atom::ControlEvent(_)) => {	    
	    println!("a control event");
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
	    println!("a context called \'{}\'", s.name);	    
	    Session::handle_context(&mut s, &session, &ruffbox, &global_parameters);	    
	},
	Expr::Constant(Atom::Command(c)) => {
	    match c {
		Command::Clear => {		    
		    Session::clear_session(session);
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
		},
		Command::LoadPart((name, mut generators)) => {
		    for gen in generators.iter_mut() {
			gen.id_tags.insert(name.clone());
		    }
		    parts_store.insert(name, generators);
		    println!("a command (load part)");
		}
	    };
	    
	},
	Expr::Constant(Atom::Float(f)) => {
	    println!("a number: {}", f)
	},		    
	_ => println!("unknown")
    }
}
