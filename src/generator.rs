use crate::{
    builtin_types::GlobalParameters,
    event::{EventOperation, InterpretableEvent, StaticEvent},
    generator_processor::GeneratorProcessor,
    markov_sequence_generator::MarkovSequenceGenerator,
};
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::boxed::Box;
use std::collections::BTreeSet;
use std::sync::*;

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
    pub time_mods: Vec<TimeMod>,
}

impl Generator {
    pub fn transfer_state(&mut self, other: &Generator) {
        self.root_generator.transfer_state(&other.root_generator);
        // genprocs follow later ...
    }

    pub fn current_events(
        &mut self,
        global_parameters: &Arc<GlobalParameters>,
    ) -> Vec<InterpretableEvent> {
        let mut events = self.root_generator.current_events();
        for proc in self.processors.iter_mut() {
            proc.process_events(&mut events, global_parameters);
            proc.process_generator(
                &mut self.root_generator,
                global_parameters,
                &mut self.time_mods,
            );
        }
        if events.is_empty() {
            println!("no events");
        }
        events
    }

    pub fn current_transition(&mut self, global_parameters: &Arc<GlobalParameters>) -> StaticEvent {
        let mut trans = self.root_generator.current_transition();
        for proc in self.processors.iter_mut() {
            proc.process_transition(&mut trans, global_parameters);
        }
        if let Some(tmod) = self.time_mods.pop() {
	    //println!("apply time mod");
            tmod.apply_to(&mut trans);
        }
        trans
    }
}

mod modifier_functions;
pub use modifier_functions::*;

pub mod modifier_functions_raw;
