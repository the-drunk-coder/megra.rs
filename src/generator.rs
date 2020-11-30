use std::boxed::Box;
use crate::{event::{StaticEvent, EventOperation},
	    generator_processor::GeneratorProcessor,
	    markov_sequence_generator::MarkovSequenceGenerator};
use ruffbox_synth::ruffbox::synth::SynthParameter;

// little helper struct for fixed time operations
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

pub struct Generator {
    pub name: String,
    pub root_generator: MarkovSequenceGenerator,
    pub processors: Vec<Box<dyn GeneratorProcessor + Send>>,
    pub time_mods: Vec<TimeMod>
}

impl Generator {

    pub fn current_events(&mut self) -> Vec<StaticEvent> {
	let mut events = self.root_generator.current_events();
	for proc in self.processors.iter_mut() {
	    proc.process_events(&mut events);
	    proc.process_generator(&mut self.root_generator);
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

// to bind, i need a holder, would that work in an enum ?
pub fn haste(gen: &mut Generator, n: usize, factor: f32) {
    for _ in 0..n {
	gen.time_mods.push(TimeMod{
	    val: factor,
	    op: EventOperation::Multiply		
	});
    }    
}

// to bind, i need a holder, would that work in an enum ?
pub fn relax(gen: &mut Generator, n: usize, factor: f32) {
    for _ in 0..n {
	gen.time_mods.push(TimeMod{
	    val: factor,
	    op: EventOperation::Divide		
	});
    }
}
