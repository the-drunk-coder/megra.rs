use crate::builtin_types::*;
use crate::event_helpers::*;
use crate::music_theory;
use crate::parameter::*;

use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::HashMap;

pub fn get_float_from_expr(e: &Expr) -> Option<f32> {
    match e {
        Expr::Constant(Atom::Float(n)) => Some(*n),
        _ => None,
    }
}

pub fn get_float_from_expr_opt(e: &Option<Expr>) -> Option<f32> {
    match e {
        Some(Expr::Constant(Atom::Float(n))) => Some(*n),
        _ => None,
    }
}

pub fn get_bool_from_expr(e: &Expr) -> Option<bool> {
    match e {
        Expr::Constant(Atom::Boolean(b)) => Some(*b),
        _ => None,
    }
}

pub fn get_bool_from_expr_opt(e: &Option<Expr>) -> Option<bool> {
    match e {
        Some(Expr::Constant(Atom::Boolean(b))) => Some(*b),
        _ => None,
    }
}

pub fn get_string_from_expr(e: &Expr) -> Option<String> {
    match e {
        Expr::Constant(Atom::Description(s)) => Some(s.to_string()),
        Expr::Constant(Atom::Symbol(s)) => Some(s.to_string()),
        _ => None,
    }
}

pub fn get_keyword_params(
    params: &mut HashMap<SynthParameter, Box<Parameter>>,
    tail_drain: &mut std::vec::Drain<Expr>,
) {
    while let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {
        params.insert(map_parameter(&k), Box::new(get_next_param(tail_drain, 0.0)));
    }
}

pub fn get_raw_keyword_params(tail_drain: &mut std::vec::Drain<Expr>) -> HashMap<String, Atom> {
    let mut params = HashMap::new();
    while let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {
        if let Some(Expr::Constant(c)) = tail_drain.next() {
            // only bool, string, float, symbol
            match c {
                Atom::Float(f) => params.insert(k, Atom::Float(f)),
                Atom::Boolean(b) => params.insert(k, Atom::Boolean(b)),
                Atom::Description(d) => params.insert(k, Atom::Description(d)),
                Atom::Symbol(d) => params.insert(k, Atom::Symbol(d)),
                _ => params.insert(k, Atom::Nothing),
            };
        }
    }
    params
}

pub fn find_keyword_float_param(
    raw_params: &HashMap<String, Atom>,
    key: String,
    default: f32,
) -> Parameter {
    Parameter::with_value(if let Some(Atom::Float(f)) = raw_params.get(&key) {
        *f
    } else {
        default
    })
}

pub fn find_keyword_float_value(
    raw_params: &HashMap<String, Atom>,
    key: String,
    default: f32,
) -> f32 {
    if let Some(Atom::Float(f)) = raw_params.get(&key) {
        *f
    } else {
        default
    }
}

pub fn find_keyword_bool_value(
    raw_params: &HashMap<String, Atom>,
    key: String,
    default: bool,
) -> bool {
    if let Some(Atom::Boolean(b)) = raw_params.get(&key) {
        *b
    } else {
        default
    }
}

pub fn get_next_keyword_param(
    key: String,
    tail_drain: &mut std::vec::Drain<Expr>,
    default: f32,
) -> Parameter {
    if let Some(Expr::Constant(Atom::Keyword(k))) = tail_drain.next() {
        if k == key {
            get_next_param(tail_drain, default)
        } else {
            Parameter::with_value(default)
        }
    } else {
        Parameter::with_value(default)
    }
}

pub fn get_next_param(tail_drain: &mut std::vec::Drain<Expr>, default: f32) -> Parameter {
    match tail_drain.next() {
        Some(Expr::Constant(Atom::Float(n))) => Parameter::with_value(n),
        Some(Expr::Constant(Atom::Parameter(pl))) => pl,
        _ => Parameter::with_value(default),
    }
}

pub fn get_next_pitch_param(tail_drain: &mut std::vec::Drain<Expr>, default: f32) -> Parameter {
    match tail_drain.next() {
        Some(Expr::Constant(Atom::Float(n))) => Parameter::with_value(n),
        Some(Expr::Constant(Atom::Parameter(pl))) => pl,
        Some(Expr::Constant(Atom::Symbol(s))) => Parameter::with_value(music_theory::to_freq(
            music_theory::from_string(&s),
            music_theory::Tuning::EqualTemperament,
        )),
        _ => Parameter::with_value(default),
    }
}
