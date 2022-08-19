use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

use crate::{
    builtin_types::ConfigParameter, generator::modifier_functions_raw::*, generator::Generator,
    parameter::DynVal,
};

pub type GenModFun = fn(&mut Generator, &[ConfigParameter], &HashMap<String, ConfigParameter>);

pub fn haste(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    // sanity check, otherwise nothing happens ...
    if let Some(ConfigParameter::Numeric(n)) = pos_args.get(0) {
        if let Some(ConfigParameter::Numeric(v)) = pos_args.get(1) {
            haste_raw(&mut gen.time_mods, *v, *n as usize);
        }
    }
}

pub fn reverse(gen: &mut Generator, _: &[ConfigParameter], _: &HashMap<String, ConfigParameter>) {
    reverse_raw(&mut gen.root_generator);
}

pub fn relax(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(n)) = pos_args.get(0) {
        if let Some(ConfigParameter::Numeric(v)) = pos_args.get(1) {
            relax_raw(&mut gen.time_mods, *v, *n as usize);
        }
    }
}

pub fn grow(
    gen: &mut Generator,
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
) {
    if let Some(ConfigParameter::Numeric(n)) = pos_args.get(0) {
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

pub fn shrink(gen: &mut Generator, _: &[ConfigParameter], _: &HashMap<String, ConfigParameter>) {
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
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        shake_raw(&mut gen.root_generator, &HashSet::new(), *f);
    }
}

pub fn sharpen(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        sharpen_raw(&mut gen.root_generator, *f);
    }
}

pub fn blur(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        blur_raw(&mut gen.root_generator, *f);
    }
}

pub fn skip(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        skip_raw(&mut gen.root_generator, *f as usize);
    }
}

pub fn rewind(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        rewind_raw(&mut gen.root_generator, *f as usize);
    }
}

pub fn solidify(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        solidify_raw(&mut gen.root_generator, *f as usize);
    }
}

pub fn rnd(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(f)) = pos_args.get(0) {
        rnd_raw(&mut gen.root_generator, f / 100.0);
    }
}

pub fn keep(gen: &mut Generator, _: &[ConfigParameter], _: &HashMap<String, ConfigParameter>) {
    gen.keep_root = true;
}

pub fn rep(
    gen: &mut Generator,
    pos_args: &[ConfigParameter],
    _: &HashMap<String, ConfigParameter>,
) {
    if let Some(ConfigParameter::Numeric(r)) = pos_args.get(0) {
        if let Some(ConfigParameter::Numeric(m)) = pos_args.get(1) {
            rep_raw(&mut gen.root_generator, r / 100.0, *m as usize);
        }
    }
}
