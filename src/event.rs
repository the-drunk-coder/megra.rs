use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::*;

use crate::builtin_types::Command;
use crate::parameter::{resolve_parameter, shake_parameter, ParameterValue};
use crate::session::SyncContext;
use crate::synth_parameter_value_arithmetic::*;

/// Events can represent arithmetic operations.
#[derive(Clone, Copy, Debug)]
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
    pub params: HashMap<SynthParameterLabel, ParameterValue>,
    pub tags: BTreeSet<String>,
    pub op: EventOperation,
}

impl Debug for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Event")
            .field("name", &self.name)
            .field("op", &self.op)
            .finish()
    }
}

/// This is the final sound or operation event. An event can also represent an operation,
/// such as an addition or multiplication.
#[derive(Clone)]
pub struct StaticEvent {
    pub name: String,
    pub params: HashMap<SynthParameterLabel, SynthParameterValue>,
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

        // TODO: handle vector values here ...
        for (k, v) in other.params.iter() {
            if self.params.contains_key(k) {
                self.params
                    .insert(*k, calc_spv(&self.params[k], v, other.op));
            } else {
                self.params.insert(*k, v.clone());
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

    pub fn evaluate_parameters(&mut self) -> HashMap<SynthParameterLabel, SynthParameterValue> {
        let mut map = HashMap::new();

        for (k, v) in self.params.iter_mut() {
            map.insert(*k, resolve_parameter(*k, v));
        }

        map
    }

    pub fn shake(&mut self, factor: f32, keep: &HashSet<SynthParameterLabel>) {
        for (k, v) in self.params.iter_mut() {
            if !keep.contains(k) && *k != SynthParameterLabel::SampleBufferNumber {
                shake_parameter(v, factor);
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
