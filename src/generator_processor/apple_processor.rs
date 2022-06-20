use rand::*;
use std::sync::*;

use crate::{
    builtin_types::GlobalParameters,
    event::{InterpretableEvent, StaticEvent},
    generator::Generator,
    generator_processor::*,
    parameter::DynVal,
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
    fn set_state(&mut self, _: GeneratorProcessorState) {}

    fn get_state(&self) -> GeneratorProcessorState {
        GeneratorProcessorState::None
    }

    fn process_events(&mut self, _: &mut Vec<InterpretableEvent>, _: &Arc<GlobalParameters>) {
        /* pass */
    }
    fn process_transition(&mut self, _: &mut StaticEvent, _: &Arc<GlobalParameters>) {
        /* pass */
    }

    fn process_generator(&mut self, gen: &mut Generator, _: &Arc<GlobalParameters>) {
        let mut rng = rand::thread_rng();
        for (prob, gen_mods) in self.modifiers_to_be_applied.iter_mut() {
            let cur_prob: usize = (prob.evaluate_numerical() as usize) % 101; // make sure prob is always between 0 and 100
            for (gen_mod_fun, pos_args, named_args) in gen_mods.iter() {
                if rng.gen_range(0..100) < cur_prob {
                    gen_mod_fun(gen, pos_args, named_args)
                }
            }
        }
    }

    fn visualize_if_possible(&mut self, _: &sync::Arc<VisualizerClient>) {
        // pass
    }

    fn clear_visualization(&self, _: &sync::Arc<VisualizerClient>) {
        // pass
    }
}
