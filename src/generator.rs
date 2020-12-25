use std::boxed::Box;
use std::collections::{HashMap, BTreeSet};
use crate::{builtin_types::ConfigParameter,
	    event::{StaticEvent, InterpretableEvent, EventOperation, SourceEvent},
	    generator_processor::GeneratorProcessor,
	    markov_sequence_generator::MarkovSequenceGenerator};
use ruffbox_synth::ruffbox::synth::SynthParameter;

// little helper struct for fixed time operations
#[derive(Clone)]
pub struct TimeMod {
    val: f32,
    op: EventOperation,
}

impl TimeMod {
    fn apply_to(&self, ev: &mut StaticEvent) {
	let old_val = ev.params[&SynthParameter::Duration];
	let new_val = match self.op {
	    EventOperation::Multiply => old_val * self.val,
	    EventOperation::Divide => old_val / self.val,
	    EventOperation::Add => old_val + self.val,
	    EventOperation::Subtract => old_val - self.val,
	    EventOperation::Replace => self.val,	    
	};
	ev.params.insert(SynthParameter::Duration, new_val);
    }
}

#[derive(Clone)]
pub struct Generator {
    pub id_tags: BTreeSet<String>,
    pub root_generator: MarkovSequenceGenerator,
    pub processors: Vec<Box<dyn GeneratorProcessor + Send>>,
    pub time_mods: Vec<TimeMod>
}


impl Generator {

    pub fn transfer_state(&mut self, other: &Generator) {
	self.root_generator.transfer_state(&other.root_generator);
	// genprocs follow later ...
    }

    pub fn current_events(&mut self) -> Vec<InterpretableEvent> {
	let mut events = self.root_generator.current_events();
	for proc in self.processors.iter_mut() {
	    proc.process_events(&mut events);
	    proc.process_generator(&mut self.root_generator, &mut self.time_mods);
	}
	events
    }
    
    pub fn current_transition(&mut self) -> StaticEvent {
	let mut trans = self.root_generator.current_transition();
	for proc in self.processors.iter_mut() {
	    proc.process_transition(&mut trans);
	}
	if let Some(tmod) = self.time_mods.pop() {
	    tmod.apply_to(&mut trans);
	} 
	trans		
    }
}

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
	    for _ in 0..n as usize {
		time_mods.push(TimeMod{
		    val: v,
		    op: EventOperation::Multiply		
		});
	    }
	}
    }            
}

pub fn relax(_: &mut MarkovSequenceGenerator,
	     time_mods: &mut Vec<TimeMod>,
	     pos_args: &Vec<ConfigParameter>,
	     _: &HashMap<String, ConfigParameter>) {
    
    if let ConfigParameter::Numeric(n) = pos_args[0] {
	if let ConfigParameter::Numeric(v) = pos_args[1] {
	    for _ in 0..n as usize {
		time_mods.push(TimeMod{
		    val: v,
		    op: EventOperation::Divide		
		});
	    }
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
	
	if let Some(result) = match m.as_str() {
	    "flower" => gen.generator.grow_flower(),
	    "old" => gen.generator.grow_old(),
	    _ => gen.generator.grow_old(),
	} {
	    //println!("grow!");
	    let template_sym = result.template_symbol.unwrap();
	    let added_sym = result.added_symbol.unwrap();
	    if let Some(old_evs) = gen.event_mapping.get(&template_sym) {
		let mut new_evs = old_evs.clone();
		for ev in new_evs.iter_mut() {
		    match ev {
			SourceEvent::Sound(s) => s.shake(f),
			SourceEvent::Control(_) => {}
		    }
		}
		
		gen.event_mapping.insert(added_sym, new_evs);
		gen.symbol_ages.insert(added_sym, 0);
		// is this ok or should it rather follow the actually added transitions ??
		let mut dur_mapping_to_add = HashMap::new();
		for sym in gen.generator.alphabet.iter() {
		    if let Some(dur) = gen.duration_mapping.get(&(*sym, template_sym)) {
			dur_mapping_to_add.insert((*sym, added_sym), dur.clone());
		    }
		    if let Some(dur) = gen.duration_mapping.get(&(template_sym, *sym)) {
			dur_mapping_to_add.insert((added_sym, *sym), dur.clone());
		    }	   	    
		}
		for (k, v) in dur_mapping_to_add.drain() {
		    gen.duration_mapping.insert(k,v);
		}
	    }    	    	    
	} else {
	    println!("can't grow!");
	}
    }    
}
