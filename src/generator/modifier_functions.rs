use std::collections::HashMap;
use crate::{builtin_types::ConfigParameter,	    
	    generator::{TimeMod, modifier_functions_raw::*},
	    markov_sequence_generator::MarkovSequenceGenerator};

pub type GenModFun = fn(&mut MarkovSequenceGenerator,
			&mut Vec::<TimeMod>,
			&Vec<ConfigParameter>,
			&HashMap<String, ConfigParameter>);

pub fn haste(_: &mut MarkovSequenceGenerator,
	     time_mods: &mut Vec<TimeMod>,
	     pos_args: &Vec<ConfigParameter>,
	     _: &HashMap<String, ConfigParameter>) {

    // sanity check, otherwise nothing happens ...    
    if let ConfigParameter::Numeric(n) = pos_args[0] {
	if let ConfigParameter::Numeric(v) = pos_args[1] {
	    haste_raw(time_mods, v, n as usize);
	}
    }            
}

pub fn relax(_: &mut MarkovSequenceGenerator,
	     time_mods: &mut Vec<TimeMod>,
	     pos_args: &Vec<ConfigParameter>,
	     _: &HashMap<String, ConfigParameter>) {
    
    if let ConfigParameter::Numeric(n) = pos_args[0] {
	if let ConfigParameter::Numeric(v) = pos_args[1] {
	    relax_raw(time_mods, v, n as usize);
	}
    }    
}

pub fn grow(gen: &mut MarkovSequenceGenerator,
	    _: &mut Vec<TimeMod>,
	    pos_args: &Vec<ConfigParameter>,
	    named_args: &HashMap<String, ConfigParameter>) {

    if let ConfigParameter::Numeric(f) = pos_args[0] {
	// get method or use default ...
	let m = if let Some(ConfigParameter::Symbolic(s)) = named_args.get("method") {
	    s.clone()
	} else {
	    "flower".to_string()
	};
	
	grow_raw(gen, &m, f);
    }
}
