use rand::*;
use std::sync::*;

use crate::{ builtin_types::GlobalParameters,
             event::{StaticEvent, InterpretableEvent},
	     parameter::Parameter,
	     generator::TimeMod,
	     generator_processor::*,
	     markov_sequence_generator::MarkovSequenceGenerator};

/// Apple-ys modifiers to the underlying processors
#[derive(Clone)]
pub struct AppleProcessor {
    pub modifiers_to_be_applied: Vec<(Parameter, GenModFunsAndArgs)>
}

impl AppleProcessor {
    pub fn new() -> Self {
	AppleProcessor {
	    modifiers_to_be_applied: Vec::new(),	    
	}	    
    }
}

impl GeneratorProcessor for AppleProcessor {    
    fn process_events(&mut self, _: &mut Vec<InterpretableEvent>, _: &Arc<GlobalParameters>) { /* pass */ }
    fn process_transition(&mut self, _: &mut StaticEvent, _: &Arc<GlobalParameters>) { /* pass */ }
    
    fn process_generator(&mut self, gen: &mut MarkovSequenceGenerator, _: &Arc<GlobalParameters>, time_mods: &mut Vec<TimeMod>) {	
	let mut rng = rand::thread_rng();
	for (prob, gen_mods) in self.modifiers_to_be_applied.iter_mut() {	    
	    let cur_prob:usize = (prob.evaluate() as usize) % 101; // make sure prob is always between 0 and 100
	    for (gen_mod_fun, pos_args, named_args) in gen_mods.iter() {						
		if rng.gen_range(0, 100) < cur_prob {
		    gen_mod_fun(gen, time_mods, pos_args, named_args)
		}				
	    }	    
	}	    	
    }    
}