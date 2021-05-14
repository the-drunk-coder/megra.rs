use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

use crate::{
    builtin_types::ConfigParameter,
    generator::{modifier_functions_raw::*, TimeMod},
    markov_sequence_generator::MarkovSequenceGenerator,
    parameter::Parameter,
};

pub type GenModFun = fn(
    &mut MarkovSequenceGenerator,
    &mut Vec<TimeMod>,
    &[ConfigParameter],
    &HashMap<String, ConfigParameter>,
);

pub fn haste(
    _: &mut MarkovSequenceGenerator,
    time_mods: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    // sanity check, otherwise nothing happens ...
    if let Some(ConfigParameter::Numeric(n)) = pos_args.get(0) {
	if let Some(ConfigParameter::Numeric(v)) = pos_args.get(1) {
            haste_raw(time_mods, *v, *n as usize);
        }
    }
}

pub fn relax(
    _: &mut MarkovSequenceGenerator,
    time_mods: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(n)) = pos_args.get(0) {
        if let Some(ConfigParameter::Numeric(v)) = pos_args.get(1) {
            relax_raw(time_mods, *v, *n as usize);
        }
    }
}

pub fn grow(
    gen: &mut MarkovSequenceGenerator,
    _: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    named_args: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        // get method or use default ...
        let m = if let Some(ConfigParameter::Symbolic(s)) = named_args.get("method") {
            s.clone()
        } else {
            "flower".to_string()
        };

        grow_raw(gen, &m, *f, &HashSet::new(), &Vec::<Parameter>::new());
    }
}

pub fn shrink(
    gen: &mut MarkovSequenceGenerator,
    _: &mut Vec<TimeMod>,
    _: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(random_symbol) = gen.generator.alphabet.choose(&mut rand::thread_rng()) {
        let r2 = *random_symbol;
        shrink_raw(gen, r2, true);
    }
}

pub fn shake(
    gen: &mut MarkovSequenceGenerator,
    _: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        shake_raw(gen, &HashSet::new(), *f);
    }
}

pub fn sharpen(
    gen: &mut MarkovSequenceGenerator,
    _: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        sharpen_raw(gen, *f);
    }
}

pub fn blur(
    gen: &mut MarkovSequenceGenerator,
    _: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        blur_raw(gen, *f);
    }
}

pub fn skip(
    gen: &mut MarkovSequenceGenerator,
    _: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        skip_raw(gen, *f as usize);
    }
}

pub fn rewind(
    gen: &mut MarkovSequenceGenerator,
    _: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {

    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        rewind_raw(gen, *f as usize);
    }
}

pub fn rnd(
    gen: &mut MarkovSequenceGenerator,
    _: &mut Vec<TimeMod>,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {

    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        rnd_raw(gen, f / 100.0);
    }
}
