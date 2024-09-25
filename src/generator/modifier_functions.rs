use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

use crate::{
    builtin_types::ConfigParameter, generator::modifier_functions_raw::*, generator::Generator,
    parameter::DynVal, GlobalVariables,
};

pub type GenModFun = fn(
    &mut Generator,
    &[ConfigParameter],
    &HashMap<String, ConfigParameter>,
    &std::sync::Arc<GlobalVariables>,
);

pub fn haste(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    // sanity check, otherwise nothing happens ...
    if let Some(ConfigParameter::Numeric(n)) = pos_args.first() {
        if let Some(ConfigParameter::Numeric(v)) = pos_args.get(1) {
            haste_raw(&mut gen.time_mods, *v, *n as usize);
        }
    }
}

pub fn reverse(
    gen: &mut Generator,
    _: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    reverse_raw(&mut gen.root_generator);
}

pub fn relax(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(n)) = pos_args.first() {
        if let Some(ConfigParameter::Numeric(v)) = pos_args.get(1) {
            relax_raw(&mut gen.time_mods, *v, *n as usize);
        }
    }
}

pub fn grow(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    named_args: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        // get method or use default ...
        let m = if let Some(ConfigParameter::Symbolic(s)) = named_args.get("method") {
            s.clone()
        } else {
            "flower".to_string()
        };

        grow_raw(
            &mut gen.root_generator,
            &m,
            *f,
            &HashSet::new(),
            &Vec::<DynVal>::new(),
        );
    }
}

pub fn grown(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    named_args: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(n)) = pos_args.first() {
        if let Some(ConfigParameter::Numeric(f)) = pos_args.get(1) {
            // get method or use default ...
            let m = if let Some(ConfigParameter::Symbolic(s)) = named_args.get("method") {
                s.clone()
            } else {
                "flower".to_string()
            };

            grown_raw(
                &mut gen.root_generator,
                &m,
                *f,
                &HashSet::new(),
                &Vec::<DynVal>::new(),
                *n as usize,
            );
        }
    }
}

pub fn shrink(
    gen: &mut Generator,
    _: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(random_symbol) = gen
        .root_generator
        .generator
        .alphabet
        .choose(&mut rand::thread_rng())
    {
        let r2 = *random_symbol;
        shrink_raw(&mut gen.root_generator, r2, true);
    }
}

pub fn shake(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        shake_raw(&mut gen.root_generator, &HashSet::new(), *f);
    }
}

pub fn sharpen(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        sharpen_raw(&mut gen.root_generator, *f);
    }
}

pub fn blur(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        blur_raw(&mut gen.root_generator, *f);
    }
}

pub fn skip(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        skip_raw(&mut gen.root_generator, *f as usize, globals);
    }
}

pub fn rewind(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        rewind_raw(&mut gen.root_generator, *f as usize);
    }
}

pub fn solidify(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        solidify_raw(&mut gen.root_generator, *f as usize);
    }
}

pub fn rnd(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        rnd_raw(&mut gen.root_generator, f / 100.0);
    }
}

pub fn shift(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.first() {
        gen.time_shift = *f as i32;
    }
}

pub fn keep(
    gen: &mut Generator,
    _: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    gen.keep_root = true;
}

pub fn rep(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
    _globals: &std::sync::Arc<GlobalVariables>,
) {
    if let Some(ConfigParameter::Numeric(r)) = pos_args.first() {
        if let Some(ConfigParameter::Numeric(m)) = pos_args.get(1) {
            rep_raw(&mut gen.root_generator, r / 100.0, *m as usize);
        }
    }
}
