use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::boxed::Box;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::builtin_types::Command;
use crate::parameter::Parameter;
use crate::session::SyncContext;

/// Events can represent arithmetic operations.
#[derive(Clone, Copy)]
pub enum EventOperation {
    Replace,
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Parser result.
#[derive(Clone)]
pub struct Event {
    pub name: String,
    pub params: HashMap<SynthParameter, Box<Parameter>>,
    pub tags: BTreeSet<String>,
    pub op: EventOperation,
}

/// This is the final sound or operation event. An event can also represent an operation,
/// such as an addition or multiplication.
#[derive(Clone)]
pub struct StaticEvent {
    pub name: String,
    pub params: HashMap<SynthParameter, f32>,
    pub tags: BTreeSet<String>,
    pub op: EventOperation,
}

/// A ControlEvent can call any function when interpreted.
#[derive(Clone)]
pub struct ControlEvent {
    pub tags: BTreeSet<String>,
    pub ctx: Option<Vec<SyncContext>>,
    pub cmd: Option<Vec<Command>>,
}

/// This is the "latent" event, where the parameters haven't been evaluated yet.
#[derive(Clone)]
pub enum SourceEvent {
    Sound(Event),
    Control(ControlEvent),
}

/// This is the "final" event after all the parameters have been evaluated,
/// so that it can be interpreted to either actual sound or control.
#[derive(Clone)]
pub enum InterpretableEvent {
    Sound(StaticEvent),
    Control(ControlEvent),
}

impl StaticEvent {
    pub fn apply(&mut self, other: &StaticEvent, filters: &[String], positive_mode: bool) {
        let mut apply = false;

        // check if tags contain one of the filters (filters are always or-matched)
        for f in filters.iter() {
            if f.is_empty()
                || (positive_mode && self.tags.contains(f))
                || (!positive_mode && !self.tags.contains(f))
            {
                apply = true;
            }
        }

        if !apply {
            return;
        }

        for (k, v) in other.params.iter() {
            if self.params.contains_key(k) {
                match other.op {
                    EventOperation::Replace => {
                        self.params.insert(*k, *v);
                    }
                    EventOperation::Add => {
                        let new_val = self.params[k] + *v;
                        self.params.insert(*k, new_val);
                    }
                    EventOperation::Subtract => {
                        let new_val = self.params[k] - *v;
                        self.params.insert(*k, new_val);
                    }
                    EventOperation::Multiply => {
                        let new_val = self.params[k] * *v;
                        self.params.insert(*k, new_val);
                    }
                    EventOperation::Divide => {
                        let new_val = self.params[k] / *v;
                        self.params.insert(*k, new_val);
                    }
                }
            } else {
                self.params.insert(*k, *v);
            }
        }
    }
}

impl Event {
    pub fn with_name_and_operation(name: String, op: EventOperation) -> Self {
        let mut tags = BTreeSet::new();
        tags.insert(name.clone()); // add to tags, for subsequent filters ...
        Event {
            name,
            params: HashMap::new(),
            tags,
            op,
        }
    }

    pub fn with_name(name: String) -> Self {
        let mut tags = BTreeSet::new();
        tags.insert(name.clone()); // add to tags, for subsequent filters ...
        Event {
            name,
            params: HashMap::new(),
            tags,
            op: EventOperation::Replace,
        }
    }

    pub fn evaluate_parameters(&mut self) -> HashMap<SynthParameter, f32> {
        let mut map = HashMap::new();

        for (k, v) in self.params.iter_mut() {
            map.insert(*k, v.evaluate());
        }

        map
    }

    pub fn shake(&mut self, factor: f32, keep: &HashSet<SynthParameter>) {
        for (k, v) in self.params.iter_mut() {
            if !keep.contains(k) && *k != SynthParameter::SampleBufferNumber {
                v.shake(factor);
            }
        }
    }

    pub fn get_static(&mut self) -> StaticEvent {
        StaticEvent {
            name: self.name.clone(),
            params: self.evaluate_parameters(),
            tags: self.tags.clone(),
            op: self.op,
        }
    }
}
