use crate::{
    builtin_types::VariableStore,
    event::{EventOperation, InterpretableEvent, StaticEvent},
    generator_processor::GeneratorProcessor,
    markov_sequence_generator::MarkovSequenceGenerator,
};
use core::fmt;
use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
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
        if let SynthParameterValue::ScalarF32(old_val) = ev.params[&SynthParameterLabel::Duration] {
            let new_val = match self.op {
                EventOperation::Multiply => old_val * self.val,
                EventOperation::Divide => old_val / self.val,
                EventOperation::Add => old_val + self.val,
                EventOperation::Subtract => old_val - self.val,
                EventOperation::Replace => self.val,
            };
            ev.params.insert(
                SynthParameterLabel::Duration,
                SynthParameterValue::ScalarF32(new_val),
            );
        }
    }
}

#[derive(Clone)]
pub struct Generator {
    pub id_tags: BTreeSet<String>,
    // the root generator is the lowest-level sequence generator
    pub root_generator: MarkovSequenceGenerator,
    // processors modify the root generator or the emitted events ...
    // could use a map but typically this will be 2 or three, rarely more, so linear
    // search is no drawback here.
    pub processors: Vec<(Option<String>, Box<dyn GeneratorProcessor + Send + Sync>)>,
    // time mods manipulate the evaluation timing ...
    pub time_mods: Vec<TimeMod>,
    // the keep_root flag determines whether we replace the root at
    // subsequent evaluations ...
    pub keep_root: bool,
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Generator({:#?})", self.id_tags)
    }
}

impl Generator {
    pub fn transfer_state(&mut self, other: &Generator) {
        self.root_generator.transfer_state(&other.root_generator);
        // this will only work if the generators remain in the same order,
        // but it'll still be helpful I think ..
        for (idx, (id_hint, gp)) in self.processors.iter_mut().enumerate() {
            // some generator processors (such as wrapped generators) have
            // ids, so we can preserve their state. Others will have their
            // state preserved when they are in the same position as before
            if let Some(id) = id_hint {
                if let Some(id_idx) = other
                    .processors
                    .iter()
                    .position(|oh| oh.0 == Some(id.to_string()))
                {
                    gp.set_state(other.processors[id_idx].1.get_state())
                }
            } else if let Some((_, g)) = other.processors.get(idx) {
                gp.set_state(g.get_state());
            }
        }
    }

    pub fn reached_end_state(&self) -> bool {
        self.root_generator.reached_end_state()
    }

    pub fn current_events(&mut self, var_store: &Arc<VariableStore>) -> Vec<InterpretableEvent> {
        let mut events = self.root_generator.current_events(var_store);

        for ev in events.iter_mut() {
            if let InterpretableEvent::Sound(s) = ev {
                s.tags = self.id_tags.union(&s.tags).cloned().collect();
            }
        }

        // temporarily take ownership of processors ...
        // that way we can pass "self" to the "process_generator"
        // function without having to pass the components individually ...
        let mut tmp_procs = Vec::new();
        tmp_procs.append(&mut self.processors);

        for (_, proc) in tmp_procs.iter_mut() {
            proc.process_events(&mut events, var_store);
            proc.process_generator(self, var_store);
        }

        // and back home ...
        self.processors.append(&mut tmp_procs);

        if events.is_empty() {
            println!("no events");
        }

        events
    }

    pub fn current_transition(&mut self, var_store: &Arc<VariableStore>) -> StaticEvent {
        let mut trans = self.root_generator.current_transition(var_store);
        for (_, proc) in self.processors.iter_mut() {
            proc.process_transition(&mut trans, var_store);
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
