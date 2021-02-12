use std::sync::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{ builtin_types::{BuiltinGlobalParameters, GlobalParameters},
             event::{StaticEvent, InterpretableEvent},
	     generator::{TimeMod, modifier_functions_raw::*},
	     generator_processor::*,
	     parameter::*,
	     markov_sequence_generator::MarkovSequenceGenerator};

struct LifemodelDefaults;

impl LifemodelDefaults {
    const GLOBAL_INIT_RESOURCES: f32		= 200.0;    
    const GROWTH_COST: f32		        = 1.0;
    const AUTOPHAGIA_REGAIN: f32                = 0.7;
    const APOPTOSIS_REGAIN: f32                 = 0.5;
    const LOCAL_RESOURCES: f32                  = 8.0;
    const NODE_LIFESPAN: usize                  = 21;
    const GROWTH_CYCLE: usize                   = 20;
    const GROWTH_METHOD: &'static str           = "flower";
    const VARIANCE: f32                         = 0.2;
    const NODE_LIFESPAN_VARIANCE: f32           = 0.1;
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
    pub durations: Vec<Parameter>,
    pub dont_let_die: bool,
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
	}	    
    }
}

impl GeneratorProcessor for LifemodelProcessor {    
    
    fn process_events(&mut self, _: &mut Vec<InterpretableEvent>, _: &Arc<GlobalParameters>) { /* pass */ }
    fn process_transition(&mut self, _: &mut StaticEvent, _: &Arc<GlobalParameters>) { /* pass */ }
    
    fn process_generator(&mut self,
			 gen: &mut MarkovSequenceGenerator,
			 global_parameters: &Arc<GlobalParameters>,
			 _: &mut Vec<TimeMod>) {
	
	// check if we need to grow ...
	let mut something_happened = false;
	if self.step_count >= self.growth_cycle {
	    // reset step count 
	    self.step_count = 0;

	    let grow = if self.local_resources >= self.growth_cost {		
		// first, draw from local resources if possible
		self.local_resources -= self.growth_cost;
		true		
	    } else if let ConfigParameter::Numeric(global_resources) = global_parameters	    
		.entry(BuiltinGlobalParameters::LifemodelGlobalResources)
		.or_insert(ConfigParameter::Numeric(LifemodelDefaults::GLOBAL_INIT_RESOURCES)) // init on first attempt 
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
		grow_raw(gen, &self.growth_method, self.variance, &self.durations);
		//println!("lm grow {:?}", gen.generator.alphabet);
		something_happened = true;
	    } else if self.autophagia {
		// remove random symbol to allow further growth
		// in the future ...
		if gen.generator.alphabet.len() > 1 || !self.dont_let_die {
		    // if there's something left to prune ...
		    let mut rand = None;
		    
		    // sometimes the borrow checker makes you do really strange things ...
		    if let Some(random_symbol) = gen.generator.alphabet.choose(&mut rand::thread_rng()) {
			rand = Some(random_symbol.clone());			
		    }
		    		    
		    if let Some(random_symbol) = rand {
			//println!("lm auto {} {:?}", random_symbol, gen.generator.alphabet);
			// don't rebalance yet ...
			shrink_raw(gen, random_symbol, false);
			self.local_resources += self.autophagia_regain;
			something_happened = true;
		    }		    
		}		
	    }	    
	}
	// now check if the current symbol is ready to make room for new ones ..
	// well, first, check if we need to do that ...
	if self.apoptosis && (gen.generator.alphabet.len() > 1 || !self.dont_let_die) {
	    // NOW check if we have a symbol
	    let mut sym = None;
	    if let Some(res) = &gen.last_transition {
		// helper to add some variance to the age ...
		let add_var = |orig: f32, var: f32| -> usize {
		    let mut rng = rand::thread_rng();
		    let rand = (var * (1000.0 - rng.gen_range(0.0, 2000.0))) * (orig / 1000.0); 
		    (orig + rand).floor() as usize
		};

		let relevant_age = add_var(*gen.symbol_ages.get(&res.last_symbol).unwrap() as f32, self.node_lifespan_variance);
		if relevant_age >= self.node_lifespan {
		    sym = Some(res.last_symbol.clone())		    
		} 		
	    };

	    if let Some(symbol_to_remove) = sym {
		//println!("lm apop {} {:?}", symbol_to_remove, gen.generator.alphabet);
		shrink_raw(gen, symbol_to_remove, false);						
		something_happened = true;
	    }
	}
	
	// now rebalance
	if something_happened {
	    gen.generator.rebalance();	    
	}
	
	self.step_count += 1;
    }        
}
