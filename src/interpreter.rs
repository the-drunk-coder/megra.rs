use std::sync;
use parking_lot::Mutex;
use std::thread;

use ruffbox_synth::ruffbox::Ruffbox;

use crate::builtin_types::*;
use crate::sample_set::SampleSet;
use crate::session::Session;
use crate::commands;

pub fn interpret<const BUFSIZE:usize, const NCHAN:usize>(parsed_in: Expr,							 
							 session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
							 ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>,
							 global_parameters: &sync::Arc<GlobalParameters>,
							 sample_set: &sync::Arc<Mutex<SampleSet>>,
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
		    let ruffbox2 = sync::Arc::clone(ruffbox);
		    let sample_set2 = sync::Arc::clone(sample_set);
		    thread::spawn(move || {
			commands::load_sample(&ruffbox2, &sample_set2, set, &mut keywords, path);
			println!("a command (load sample)");
		    });	   
		},
		Command::LoadSampleSets(path) => {
		    let ruffbox2 = sync::Arc::clone(ruffbox);
		    let sample_set2 = sync::Arc::clone(sample_set);
		    thread::spawn(move || {
			commands::load_sample_sets(&ruffbox2, &sample_set2, path);
			println!("a command (load sample sets)");
		    });		    
		},
		Command::LoadSampleSet(path) => {
		    let ruffbox2 = sync::Arc::clone(ruffbox);
		    let sample_set2 = sync::Arc::clone(sample_set);
		    thread::spawn(move || {
			commands::load_sample_set_string(&ruffbox2, &sample_set2, path);
			println!("a command (load sample sets)");
		    });		    
		    println!("a command (load sample set)");
		},
		Command::LoadPart((name, generators)) => {
		    commands::load_part(parts_store, name, generators);
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
