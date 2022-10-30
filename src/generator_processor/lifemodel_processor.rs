use rand::seq::SliceRandom;
use rand::Rng;
use std::{collections::HashSet, sync::*};

use ruffbox_synth::building_blocks::SynthParameterLabel;

use crate::{
    builtin_types::{BuiltinGlobalParameters, GlobalParameters},
    event::{InterpretableEvent, StaticEvent},
    generator::modifier_functions_raw::*,
    generator::Generator,
    generator_processor::*,
    parameter::*,
};

struct LifemodelDefaults;

impl LifemodelDefaults {
    const GLOBAL_INIT_RESOURCES: f32 = 200.0;
    const GROWTH_COST: f32 = 1.0;
    const AUTOPHAGIA_REGAIN: f32 = 0.7;
    const APOPTOSIS_REGAIN: f32 = 0.5;
    const LOCAL_RESOURCES: f32 = 8.0;
    const NODE_LIFESPAN: usize = 21;
    const GROWTH_CYCLE: usize = 20;
    const GROWTH_METHOD: &'static str = "flower";
    const VARIANCE: f32 = 0.2;
    const NODE_LIFESPAN_VARIANCE: f32 = 0.1;
    const RND_CHANCE: f32 = 0.0;
    const SOLIDIFY_CHANCE: f32 = 0.01;
    const SOLIDIFY_LEN: usize = 3;
}

/// Apple-ys events to the throughcoming ones
#[derive(Clone)]
pub struct LifemodelProcessor {
    pub step_count: usize,
    pub growth_cycle: usize,
    pub growth_method: String,
    pub variance: f32,
    pub node_lifespan: usize,
    pub node_lifespan_variance: f32,
    pub apoptosis: bool,
    pub autophagia: bool,
    pub local_resources: f32,
    pub growth_cost: f32,
    pub apoptosis_regain: f32,
    pub autophagia_regain: f32,
    pub durations: Vec<DynVal>,
    pub dont_let_die: bool,
    pub keep_param: HashSet<SynthParameterLabel>,
    pub global_contrib: bool,
    pub solidify_chance: f32,
    pub solidify_len: usize,
    pub rnd_chance: f32,
}

impl LifemodelProcessor {
    pub fn new() -> Self {
        LifemodelProcessor {
            step_count: 0,
            growth_cycle: LifemodelDefaults::GROWTH_CYCLE,
            growth_method: LifemodelDefaults::GROWTH_METHOD.to_string(),
            variance: LifemodelDefaults::VARIANCE,
            node_lifespan: LifemodelDefaults::NODE_LIFESPAN,
            node_lifespan_variance: LifemodelDefaults::NODE_LIFESPAN_VARIANCE,
            apoptosis: true,
            autophagia: true,
            local_resources: LifemodelDefaults::LOCAL_RESOURCES,
            growth_cost: LifemodelDefaults::GROWTH_COST,
            apoptosis_regain: LifemodelDefaults::APOPTOSIS_REGAIN,
            autophagia_regain: LifemodelDefaults::AUTOPHAGIA_REGAIN,
            durations: Vec::new(),
            dont_let_die: true,
            keep_param: HashSet::new(),
            global_contrib: false,
            solidify_chance: LifemodelDefaults::SOLIDIFY_CHANCE,
            solidify_len: LifemodelDefaults::SOLIDIFY_LEN,
            rnd_chance: LifemodelDefaults::RND_CHANCE,
        }
    }
}

impl GeneratorProcessor for LifemodelProcessor {

    // I'm a bit surprises this one's stateless ...
    
    // this one only processes the generators ...
    fn process_generator(
        &mut self,
        gen: &mut Generator,
        global_parameters: &Arc<GlobalParameters>,
    ) {
        // check if we need to grow ...
        let mut something_happened = false;
        if self.step_count >= self.growth_cycle {
            // println!("proc lm");
            // reset step count
            self.step_count = 0;

            let grow = if self.local_resources >= self.growth_cost {
                // first, draw from local resources if possible
                self.local_resources -= self.growth_cost;
                true
            } else if let ConfigParameter::Numeric(global_resources) = global_parameters
                .entry(BuiltinGlobalParameters::LifemodelGlobalResources)
                .or_insert(ConfigParameter::Numeric(
                    LifemodelDefaults::GLOBAL_INIT_RESOURCES,
                )) // init on first attempt
                .value_mut()
            {
                // get global resources, init value if it doesn't exist
                if *global_resources >= self.growth_cost {
                    *global_resources -= self.growth_cost;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if grow {
                grow_raw(
                    &mut gen.root_generator,
                    &self.growth_method,
                    self.variance,
                    &self.keep_param,
                    &self.durations,
                );
                //println!("lm grow {:?}", gen.root_generator.generator.alphabet);
                something_happened = true;
            } else if self.autophagia {
                // remove random symbol to allow further growth
                // in the future ...
                if gen.root_generator.generator.alphabet.len() > 1 || !self.dont_let_die {
                    // if there's something left to prune ...
                    if let Some(random_symbol) = gen
                        .root_generator
                        .generator
                        .alphabet
                        .choose(&mut rand::thread_rng())
                    {
                        //println!("lm auto {} {:?}", random_symbol, gen.root_generator.generator.alphabet);
                        // don't rebalance yet ...
                        let r2 = *random_symbol; // sometimes the borrow checker makes you do really strange things ...
                        shrink_raw(&mut gen.root_generator, r2, false);

                        if self.global_contrib {
                            if let ConfigParameter::Numeric(global_resources) = global_parameters
                                .entry(BuiltinGlobalParameters::LifemodelGlobalResources)
                                .or_insert(ConfigParameter::Numeric(
                                    LifemodelDefaults::GLOBAL_INIT_RESOURCES,
                                )) // init on first attempt
                                .value_mut()
                            {
                                // get global resources, init value if it doesn't exist
                                *global_resources += self.autophagia_regain;
                            }
                        } else {
                            self.local_resources += self.autophagia_regain;
                        }

                        something_happened = true;
                    }
                }
            }
        }
        // now check if the current symbol is ready to make room for new ones ..
        // well, first, check if we need to do that ...
        if self.apoptosis && (gen.root_generator.generator.alphabet.len() > 1 || !self.dont_let_die)
        {
            // NOW check if we have a symbol
            let mut sym = None;
            if let Some(res) = &gen.root_generator.last_transition {
                // helper to add some variance to the age ...
                let add_var = |orig: f32, var: f32| -> usize {
                    let mut rng = rand::thread_rng();
                    let rand = (var * (1000.0 - rng.gen_range(0.0..2000.0))) * (orig / 1000.0);
                    (orig + rand).floor() as usize
                };

                // sometimes the growth/shrink processed might have invalidated or deleted
                // the last symbol, so let's check just in case ...
                if let Some(age) = gen.root_generator.symbol_ages.get(&res.last_symbol) {
                    let relevant_age = add_var(*age as f32, self.node_lifespan_variance);
                    if relevant_age >= self.node_lifespan {
                        sym = Some(res.last_symbol)
                    }
                }
            };

            if let Some(symbol_to_remove) = sym {
                //println!("lm apop {} {:?}", symbol_to_remove, gen.root_generator.generator.alphabet);
                shrink_raw(&mut gen.root_generator, symbol_to_remove, false);
                if self.global_contrib {
                    if let ConfigParameter::Numeric(global_resources) = global_parameters
                        .entry(BuiltinGlobalParameters::LifemodelGlobalResources)
                        .or_insert(ConfigParameter::Numeric(
                            LifemodelDefaults::GLOBAL_INIT_RESOURCES,
                        )) // init on first attempt
                        .value_mut()
                    {
                        // get global resources, init value if it doesn't exist
                        *global_resources += self.apoptosis_regain;
                    }
                } else {
                    self.local_resources += self.apoptosis_regain;
                }
                something_happened = true;
            }
        }

        if something_happened && self.solidify_chance > 0.0 {
            let mut rng = rand::thread_rng();
            let rand = rng.gen_range(0.0..1000.0) / 1000.0;
            if rand < self.solidify_chance {
                gen.root_generator.generator.solidify(self.solidify_len);
            }
        }

        if something_happened && self.rnd_chance > 0.0 {
            gen.root_generator
                .generator
                .randomize_edges(self.rnd_chance, self.rnd_chance);
        }

        // now rebalance
        if something_happened {
            gen.root_generator.generator.rebalance();
        }

        self.step_count += 1;
    }    
}
