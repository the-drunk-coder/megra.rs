use std::collections::{BTreeSet, HashMap};

use ruffbox_synth::ruffbox::synth::SynthParameter;

use crate::builtin_types::*;
use crate::session::OutputMode;
use crate::parameter::Parameter;
use crate::event::{Event, EventOperation};
use crate::generator_processor::PearProcessor;


pub fn handle(mul: &BuiltInMultiplexer, tail: &mut Vec<Expr>, parts_store: &PartsStore, out_mode: OutputMode) -> Atom {
    let last = tail.pop(); // generator or generator list ...

    let mut gen_proc_list_list = Vec::new();
    
    let mut tail_drain = tail.drain(..);
    while let Some(Expr::Constant(c)) = tail_drain.next() {
	match c {
	    Atom::GeneratorProcessorList(gpl) => {
		gen_proc_list_list.push(gpl);
	    },
	    Atom::GeneratorProcessor(gp) => {
		let mut gpl = Vec::new();
		gpl.push(gp);
		gen_proc_list_list.push(gpl);
	    },
	    Atom::GeneratorModifierFunction(_gm) => {
		//???
	    },
	    _ => { println!("can't multiplex this ..."); },
	}
    }

    let mut gens = Vec::new();

    match last {
	Some(Expr::Constant(Atom::Symbol(s))) => {
	    if let Some(kl) = parts_store.get(&s) {
		let mut klc = kl.clone();
		// collect tags ... make sure the multiplexing process leaves
		// each generator individually, but deterministically tagged ...
		let mut all_tags:BTreeSet<String> = BTreeSet::new();
		
		for gen in klc.iter() {
		    all_tags.append(&mut gen.id_tags.clone());
		}

		let mut idx:usize = 0;
		for gen in klc.drain(..) {
		    // multiplex into duplicates by cloning ...		
		    for gpl in gen_proc_list_list.iter() {
			let mut pclone = gen.clone();
			
			// this isn't super elegant but hey ... 
			for i in idx..100 {
			    let tag = format!("mpx-{}", i);
			    if !all_tags.contains(&tag) {
				pclone.id_tags.insert(tag);
				idx = i + 1;
				break;
			    } 		    
			}
			
			pclone.processors.append(&mut gpl.clone());
			gens.push(pclone);
		    }
		    gens.push(gen);		
		}
	    } else {
		println!("warning: '{} not defined!", s);
	    }
	},
	Some(Expr::Constant(Atom::Generator(g))) => {	    
	    // multiplex into duplicates by cloning ...
	    let mut idx:usize = 0;
	    for mut gpl in gen_proc_list_list.drain(..) {
		let mut pclone = g.clone();

		// this isn't super elegant but hey ... 
		for i in idx..100 {
		    let tag = format!("mpx-{}", i);
		    if !pclone.id_tags.contains(&tag) {
			pclone.id_tags.insert(tag);
			idx = i + 1;
			break;
		    } 		    
		}
		
		pclone.processors.append(&mut gpl);
		gens.push(pclone);
	    }	    	    
  	    gens.push(g);
	 },
	Some(Expr::Constant(Atom::GeneratorList(mut gl))) => {
	    // collect tags ... make sure the multiplexing process leaves
	    // each generator individually, but deterministically tagged ...
	    let mut all_tags:BTreeSet<String> = BTreeSet::new();
	    
	    for gen in gl.iter() {
		all_tags.append(&mut gen.id_tags.clone());
	    }

	    let mut idx:usize = 0;
	    for gen in gl.drain(..) {
		// multiplex into duplicates by cloning ...		
		for gpl in gen_proc_list_list.iter() {
		    let mut pclone = gen.clone();

		    // this isn't super elegant but hey ... 
		    for i in idx..100 {
			let tag = format!("mpx-{}", i);
			if !all_tags.contains(&tag) {
			    pclone.id_tags.insert(tag);
			    idx = i + 1;
			    break;
			} 		    
		    }
		    
		    pclone.processors.append(&mut gpl.clone());
		    gens.push(pclone);
		}
		gens.push(gen);
	    }	    
	},
	_ => {}	
    }
            	
    // for xdup, this would be enough ... for xspread etc, we need to prepend another processor ...
    match mul {
	BuiltInMultiplexer::XSpread => {
	    let positions = match out_mode {
		OutputMode::Stereo => {
		    if gens.len() == 1 {
			vec![0.0]
		    } else {
			let mut p = Vec::new();
			for i in 0..gens.len() {
			    let val = (i as f32 * (2.0 / (gens.len() as f32 - 1.0))) - 1.0;
			    p.push(val);			    
			}
			p		
		    }
		},
		OutputMode::FourChannel => {
		    if gens.len() == 1 {
			vec![0.0]
		    } else {
			let mut p = Vec::new();
			for i in 0..gens.len() {
			    let val = 1.0 + (i as f32 * (3.0 / (gens.len() as f32 - 1.0)));
			    p.push(val);			    
			}
			p
		    }
		},
		OutputMode::EightChannel => {
		    if gens.len() == 1 {
			vec![0.0]
		    } else {
			let mut p = Vec::new();
			for i in 0..gens.len() {
			    let val = 1.0 + (i as f32 * (7.0 / (gens.len() as f32 - 1.0)));
			    p.push(val);			    
			}
			p
		    }
		},
	    };

	    //println!("{:?}", positions);
	    for i in 0..gens.len() {
		let mut p = PearProcessor::new();
		let mut ev = Event::with_name_and_operation("pos".to_string(), EventOperation::Replace);
		ev.params.insert(SynthParameter::ChannelPosition, Box::new(Parameter::with_value(positions[i])));
		let mut filtered_events = HashMap::new();
		filtered_events.insert(vec!["".to_string()], vec![ev]);
		p.events_to_be_applied.push((Parameter::with_value(100.0), filtered_events));
		gens[i].processors.push(Box::new(p));
	    }
	},
	_ => {}
    }
                
    Atom::GeneratorList(gens)
}