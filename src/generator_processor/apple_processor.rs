use rand::*;
use std::sync::*;

use crate::{
    builtin_types::VariableStore, generator::Generator, generator_processor::*, parameter::DynVal,
};

/// Apple-ys modifiers to the underlying processors
#[derive(Clone)]
pub struct AppleProcessor {
    pub modifiers_to_be_applied: Vec<(DynVal, GenModFunsAndArgs)>,
}

impl AppleProcessor {
    pub fn new() -> Self {
        AppleProcessor {
            modifiers_to_be_applied: Vec::new(),
        }
    }
}

impl GeneratorProcessor for AppleProcessor {
    // this one only processes generators ... for the event stream processor,
    // see "pear"
    fn process_generator(&mut self, gen: &mut Generator, globals: &Arc<VariableStore>) {
        let mut rng = rand::thread_rng();
        for (prob, gen_mods) in self.modifiers_to_be_applied.iter_mut() {
            let cur_prob: usize = (prob.evaluate_numerical() as usize) % 101; // make sure prob is always between 0 and 100
            for (gen_mod_fun, pos_args, named_args) in gen_mods.iter() {
                if rng.gen_range(0..100) < cur_prob {
                    gen_mod_fun(gen, pos_args, named_args, globals)
                }
            }
        }
    }
}
