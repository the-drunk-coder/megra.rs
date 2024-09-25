use crate::{
    builtin_types::GlobalVariables,
    event::{EventOperation, InterpretableEvent, StaticEvent},
    generator_processor::GeneratorProcessor,
    markov_sequence_generator::MarkovSequenceGenerator,
    parser::FunctionMap,
    sample_set::SampleAndWavematrixSet,
    session::OutputMode,
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
        if let SynthParameterValue::ScalarF32(old_val) =
            ev.params[&SynthParameterLabel::Duration.into()]
        {
            let new_val = match self.op {
                EventOperation::Multiply => old_val * self.val,
                EventOperation::Divide => old_val / self.val,
                EventOperation::Add => old_val + self.val,
                EventOperation::Subtract => old_val - self.val,
                EventOperation::Replace => self.val,
            };
            ev.params.insert(
                SynthParameterLabel::Duration.into(),
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
    pub processors: Vec<Box<dyn GeneratorProcessor + Send + Sync>>,
    // time mods manipulate the evaluation timing ...
    pub time_mods: Vec<TimeMod>,

    pub time_shift: i32,

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
    pub fn update_internal_ids(&mut self) {
        for p in self.processors.iter_mut() {
            p.inherit_id(&self.id_tags);
        }
    }

    // get processor ids
    pub fn collect_supplemental_ids(&self, supplemental: &mut BTreeSet<BTreeSet<String>>) {
        for p in self.processors.iter() {
            p.collect_id_set(supplemental);
        }
    }

    pub fn transfer_state(&mut self, other: &Generator) {
        self.root_generator.transfer_state(&other.root_generator);
        // this will only work if the generators remain in the same order,
        // but it'll still be helpful I think ..
        for (idx, gp) in self.processors.iter_mut().enumerate() {
            // some generator processors (such as wrapped generators) have
            // ids, so we can preserve their state. Others will have their
            // state preserved when they are in the same position as before
            if let Some(id) = gp.get_id() {
                if let Some(id_idx) = other
                    .processors
                    .iter()
                    .position(|oh| oh.get_id() == Some(id.to_string()))
                {
                    if let Some(state) = other.processors[id_idx].get_state() {
                        gp.set_state(state)
                    }
                }
            } else if let Some(g) = other.processors.get(idx) {
                if let Some(state) = g.get_state() {
                    gp.set_state(state);
                }
            }
        }
    }

    pub fn reached_end_state(&self) -> bool {
        self.root_generator.reached_end_state()
    }

    pub fn current_events(
        &mut self,
        globals: &Arc<GlobalVariables>,
        functions: &Arc<FunctionMap>,
        sample_set: SampleAndWavematrixSet,
        out_mode: OutputMode,
    ) -> Vec<InterpretableEvent> {
        let mut events = self.root_generator.current_events(globals);

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

        for proc in tmp_procs.iter_mut() {
            proc.process_events(
                &mut events,
                globals,
                functions,
                sample_set.clone(),
                out_mode,
            );
            proc.process_generator(self, globals);
        }

        // and back home ...
        self.processors.append(&mut tmp_procs);

        if events.is_empty() {
            println!("no events");
        }

        events
    }

    pub fn current_transition(
        &mut self,
        globals: &Arc<GlobalVariables>,
        functions: &Arc<FunctionMap>,
        sample_set: SampleAndWavematrixSet,
        out_mode: OutputMode,
    ) -> StaticEvent {
        let mut trans = self.root_generator.current_transition(globals);
        for proc in self.processors.iter_mut() {
            proc.process_transition(&mut trans, globals, functions, sample_set.clone(), out_mode);
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
