use std::collections::HashMap;
use crate::builtin_types::*;
use crate::event_helpers::*;
use crate::parameter::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;

pub fn get_float_from_expr(e: &Expr) -> Option<f32> {
    match e {
	Expr::Constant(Atom::Float(n)) => Some(*n),
	_ => None
    }  
}

pub fn get_bool_from_expr(e: &Expr) -> Option<bool> {
    match e {
	Expr::Constant(Atom::Boolean(b)) => Some(*b),
	_ => None
    }	    
}

pub fn get_string_from_expr(e: &Expr) -> Option<String> {
    match e {
	Expr::Constant(Atom::Description(s)) => Some(s.to_string()),
	Expr::Constant(Atom::Symbol(s)) => Some(s.to_string()),
	_ => None
    }     
}

pub fn get_keyword_params(params: &mut HashMap<SynthParameter, Box<Parameter>>, tail_drain: &mut std::vec::Drain<Expr>) {
    while let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {
	params.insert(map_parameter(&k), Box::new(get_next_param(tail_drain, 0.0)));
    }
}

pub fn get_next_param(tail_drain: &mut std::vec::Drain<Expr>, default: f32) -> Parameter {
    match tail_drain.next() {
	Some(Expr::Constant(Atom::Float(n))) => Parameter::with_value(n),
	Some(Expr::Constant(Atom::Parameter(pl))) => pl,
	_ => Parameter::with_value(default)
    }
}
