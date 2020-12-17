use std::collections::HashMap;

use crate::builtin_types::*;

use crate::parameter::Parameter;
use crate::generator_processor::*;
use crate::parser::parser_helpers::*;

fn collect_every(tail: &mut Vec<Expr>) -> Box<EveryProcessor> {
    let mut tail_drain = tail.drain(..); 
    let mut proc = EveryProcessor::new();

    let mut last_filters = Vec::new();
    last_filters.push("".to_string());
    
    let mut cur_step = Parameter::with_value(1.0); // if nothing is specified, it's always applied
    let mut gen_mod_funs = Vec::new();
    let mut events = Vec::new();
    let mut collect_filters = false;
        
    while let Some(Expr::Constant(c)) = tail_drain.next() {				
	match c {
	    Atom::GeneratorModifierFunction(g) => {
		gen_mod_funs.push(g);
		collect_filters = false;
	    }
	    Atom::SoundEvent(e) => {
		events.push(e);
		collect_filters = false;
	    },
	    Atom::Symbol(s) => {
		if collect_filters {
		    last_filters.push(s)
		}
	    },
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "for" => {
			if !events.is_empty() || !gen_mod_funs.is_empty() {
			    let mut n_mods = Vec::new();
			    n_mods.append(&mut gen_mod_funs);
			    
			    let mut filtered_events = HashMap::new();
			    let mut n_evs = Vec::new();
			    let mut n_filters = Vec::new();
			    n_evs.append(&mut events);
			    n_filters.append(&mut last_filters);
			    filtered_events.insert(n_filters, n_evs);
			    
			    proc.things_to_be_applied.push((cur_step.clone(), filtered_events, n_mods));
			}
			// collect new filters
			collect_filters = true;
		    },
		    "n" => {
			if !events.is_empty() || !gen_mod_funs.is_empty() {
			    let mut n_mods = Vec::new();
			    n_mods.append(&mut gen_mod_funs);
			    
			    let mut filtered_events = HashMap::new();
			    let mut n_evs = Vec::new();
			    let mut n_filters = Vec::new();
			    n_evs.append(&mut events);
			    n_filters.append(&mut last_filters);
			    filtered_events.insert(n_filters, n_evs);
			    
			    proc.things_to_be_applied.push((cur_step.clone(), filtered_events, n_mods));
			}
			// grab new probability
			cur_step = get_next_param(&mut tail_drain, 1.0);
			collect_filters = false;
		    },		    
		    _ => {}
		}
	    },	    
	    _ => {}
	}
    }

    // save last context
    if !events.is_empty() || !gen_mod_funs.is_empty() {			
	let mut filtered_events = HashMap::new();	
	filtered_events.insert(last_filters, events);	
	proc.things_to_be_applied.push((cur_step, filtered_events, gen_mod_funs));
    }
    
    Box::new(proc)
}

fn collect_pear (tail: &mut Vec<Expr>) -> Box<PearProcessor> {
    let mut tail_drain = tail.drain(..);
    let mut proc = PearProcessor::new();

    let mut last_filters = Vec::new();
    last_filters.push("".to_string());
    
    let mut evs = Vec::new();
    let mut collect_filters = false;
    let mut cur_prob = Parameter::with_value(100.0); // if nothing is specified, it's always or prob 100
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {				
	match c {
	    Atom::SoundEvent(e) => {
		evs.push(e);
		if collect_filters {
		    collect_filters = false;
		}
	    },
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "p" => {
			// save current context, if something has been found
			if !evs.is_empty() {
			    let mut filtered_events = HashMap::new();
			    let mut n_evs = Vec::new();
			    let mut n_filters = Vec::new();
			    n_evs.append(&mut evs);
			    n_filters.extend_from_slice(&last_filters);
			    filtered_events.insert(n_filters, n_evs);
			    proc.events_to_be_applied.push((cur_prob.clone(), filtered_events));
			}				
			// grab new probability
			cur_prob = get_next_param(&mut tail_drain, 100.0);
			collect_filters = false;
		    },
		    "for" => {
			if !evs.is_empty() {
			    let mut filtered_events = HashMap::new();
			    let mut n_evs = Vec::new();
			    let mut n_filters = Vec::new();
			    n_evs.append(&mut evs);
			    n_filters.append(&mut last_filters);
			    filtered_events.insert(n_filters, n_evs);
			    proc.events_to_be_applied.push((cur_prob.clone(), filtered_events));
			}
			// collect new filters
			collect_filters = true;
		    },
		    _ => {}
		}
	    },
	    Atom::Symbol(s) => {
		if collect_filters {
		    last_filters.push(s)
		}
	    },
	    _ => {}
	}
    }

    // save last context
    if !evs.is_empty() {
	let mut filtered_events = HashMap::new();
	filtered_events.insert(last_filters, evs);
	proc.events_to_be_applied.push((cur_prob, filtered_events));
    }	    	    
    Box::new(proc)
}


fn collect_apple (tail: &mut Vec<Expr>) -> Box<AppleProcessor> {
    let mut tail_drain = tail.drain(..); 
    let mut proc = AppleProcessor::new();
            
    let mut cur_prob = Parameter::with_value(100.0); // if nothing is specified, it's always or prob 100
    let mut gen_mod_funs = Vec::new();
        
    while let Some(Expr::Constant(c)) = tail_drain.next() {				
	match c {
	    Atom::GeneratorModifierFunction(g) => {
		gen_mod_funs.push(g);
	    }
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "p" => {
			if !gen_mod_funs.is_empty() {
			    let mut new_mods = Vec::new();
			    new_mods.append(&mut gen_mod_funs);			    
			    proc.modifiers_to_be_applied.push((cur_prob.clone(), new_mods));
			}
			// grab new probability
			cur_prob = get_next_param(&mut tail_drain, 100.0);
		    },		    
		    _ => {}
		}
	    },	    
	    _ => {}
	}
    }

    // save last context
    if !gen_mod_funs.is_empty() {	
	proc.modifiers_to_be_applied.push((cur_prob, gen_mod_funs));
    }
    
    Box::new(proc)
}

pub fn collect_generator_processor(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>) -> Box<dyn GeneratorProcessor + Send> {
    match proc_type {
	BuiltInGenProc::Pear => collect_pear(tail),
	BuiltInGenProc::Apple => collect_apple(tail),
	BuiltInGenProc::Every => collect_every(tail),
    }        
}

// store list of genProcs in a vec if there's no root gen ???
pub fn handle(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>, parts_store: &PartsStore) -> Atom {    
    let last = tail.pop();
    match last {
	Some(Expr::Constant(Atom::Generator(mut g))) => {
	    g.processors.push(collect_generator_processor(proc_type, tail));
	    Atom::Generator(g)
	},
	Some(Expr::Constant(Atom::Symbol(s))) => {
	    if let Some(gl) = parts_store.get(&s) {
		let gp = collect_generator_processor(proc_type, tail);
		let mut glc = gl.clone();
		for gen in glc.iter_mut() { // clone here
		    gen.processors.push(gp.clone());		    
		}	    
		Atom::GeneratorList(glc)
	    } else {
		println!("warning: '{} not defined!", s);
		Atom::GeneratorProcessor(collect_generator_processor(proc_type, tail)) // ignore symbol
	    }
	},
	Some(Expr::Constant(Atom::GeneratorList(mut gl))) => {
	    let gp = collect_generator_processor(proc_type, tail);
	    for gen in gl.iter_mut() {
		gen.processors.push(gp.clone());
	    }	    
	    Atom::GeneratorList(gl)
	},
	Some(Expr::Constant(Atom::GeneratorProcessor(gp)))=> {
	    let mut v = Vec::new();
	    v.push(gp);
	    v.push(collect_generator_processor(proc_type, tail));
	    Atom::GeneratorProcessorList(v)
	},
	Some(Expr::Constant(Atom::GeneratorProcessorList(mut l)))=> {
	    l.push(collect_generator_processor(proc_type, tail));
	    Atom::GeneratorProcessorList(l)
	},
	Some(l) => {
	    tail.push(l);
	    Atom::GeneratorProcessor(collect_generator_processor(proc_type, tail))
	},
	None => {
	    Atom::Nothing
	}
    }    
}
