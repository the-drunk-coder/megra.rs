use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::*;

use crate::builtin_types::Command;
use crate::parameter::ParameterValue;
use crate::session::SyncContext;

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
                match other.op {
                    EventOperation::Replace => {
                        self.params.insert(*k, v.clone());
                    }
                    EventOperation::Add => {
                        if let SynthParameterValue::ScalarF32(new_val) = self.params[k] {
                            if let SynthParameterValue::ScalarF32(old_val) = v {
                                self.params
                                    .insert(*k, SynthParameterValue::ScalarF32(old_val + new_val));
                            };
                        }
                    }
                    EventOperation::Subtract => {
                        if let SynthParameterValue::ScalarF32(new_val) = self.params[k] {
                            if let SynthParameterValue::ScalarF32(old_val) = v {
                                self.params
                                    .insert(*k, SynthParameterValue::ScalarF32(old_val - new_val));
                            };
                        }
                    }
                    EventOperation::Multiply => {
                        if let SynthParameterValue::ScalarF32(new_val) = self.params[k] {
                            if let SynthParameterValue::ScalarF32(old_val) = v {
                                self.params
                                    .insert(*k, SynthParameterValue::ScalarF32(old_val * new_val));
                            };
                        }
                    }
                    EventOperation::Divide => {
                        if let SynthParameterValue::ScalarF32(new_val) = self.params[k] {
                            if let SynthParameterValue::ScalarF32(old_val) = v {
                                self.params
                                    .insert(*k, SynthParameterValue::ScalarF32(old_val / new_val));
                            };
                        }
                    }
                }
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
            match v {
                ParameterValue::Scalar(val) => {
                    if *k == SynthParameterLabel::SampleBufferNumber {
                        map.insert(*k, val.evaluate_val_usize());
                    } else {
                        map.insert(*k, val.evaluate_val_f32());
                    }
                }
                ParameterValue::Vector(vals) => {
                    let mut static_vals: Vec<f32> = Vec::new();
                    for val in vals.iter_mut() {
                        static_vals.push(val.evaluate_numerical());
                    }
                    map.insert(*k, SynthParameterValue::VecF32(static_vals));
                }
                ParameterValue::Matrix(mat) => {
                    let mut static_vals: Vec<Vec<f32>> = Vec::new();
                    let mut rows = 0;
                    let mut cols = 0;
                    for (r, row) in mat.iter_mut().enumerate() {
                        static_vals.push(Vec::new());
                        rows += 1;
                        if row.len() > cols {
                            cols = row.len();
                        }
                        for col in row.iter_mut() {
                            static_vals[r].push(col.evaluate_numerical());
                        }
                    }

                    // make sure all rows have the same lenght
                    for row in static_vals.iter_mut() {
                        if row.len() < cols {
                            row.append(&mut vec![0.0; cols - row.len()])
                        }
                    }

                    map.insert(
                        *k,
                        SynthParameterValue::MatrixF32((rows, cols), static_vals),
                    );
                }
                ParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => {
                    map.insert(
                        *k,
                        SynthParameterValue::Lfo(
                            init.evaluate_numerical(),
                            freq.evaluate_numerical(),
                            eff_phase.evaluate_numerical(),
                            amp.evaluate_numerical(),
                            add.evaluate_numerical(),
                            *op,
                        ),
                    );
                }
                ParameterValue::LFSaw(init, freq, eff_phase, amp, add, op) => {
                    map.insert(
                        *k,
                        SynthParameterValue::LFSaw(
                            init.evaluate_numerical(),
                            freq.evaluate_numerical(),
                            eff_phase.evaluate_numerical(),
                            amp.evaluate_numerical(),
                            add.evaluate_numerical(),
                            *op,
                        ),
                    );
                }
                ParameterValue::LFTri(init, freq, eff_phase, amp, add, op) => {
                    map.insert(
                        *k,
                        SynthParameterValue::LFTri(
                            init.evaluate_numerical(),
                            freq.evaluate_numerical(),
                            eff_phase.evaluate_numerical(),
                            amp.evaluate_numerical(),
                            add.evaluate_numerical(),
                            *op,
                        ),
                    );
                }
                ParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                    map.insert(
                        *k,
                        SynthParameterValue::LFSquare(
                            init.evaluate_numerical(),
                            freq.evaluate_numerical(),
                            pw.evaluate_numerical(),
                            amp.evaluate_numerical(),
                            add.evaluate_numerical(),
                            *op,
                        ),
                    );
                }
            }
        }

        map
    }

    pub fn shake(&mut self, factor: f32, keep: &HashSet<SynthParameterLabel>) {
        for (k, v) in self.params.iter_mut() {
            if !keep.contains(k) && *k != SynthParameterLabel::SampleBufferNumber {
                match v {
                    ParameterValue::Scalar(val) => {
                        val.shake(factor);
                    }
                    ParameterValue::Vector(vals) => {
                        for val in vals.iter_mut() {
                            val.shake(factor);
                        }
                    }
                    ParameterValue::Matrix(mat) => {
                        for row in mat.iter_mut() {
                            for col in row.iter_mut() {
                                col.shake(factor);
                            }
                        }
                    }
                    ParameterValue::Lfo(init, freq, eff_phase, amp, add, _) => {
                        init.shake(factor);
                        freq.shake(factor);
                        eff_phase.shake(factor);
                        amp.shake(factor);
                        add.shake(factor);
                    }
                    ParameterValue::LFSaw(init, freq, eff_phase, amp, add, _) => {
                        init.shake(factor);
                        freq.shake(factor);
                        eff_phase.shake(factor);
                        amp.shake(factor);
                        add.shake(factor);
                    }
                    ParameterValue::LFTri(init, freq, eff_phase, amp, add, _) => {
                        init.shake(factor);
                        freq.shake(factor);
                        eff_phase.shake(factor);
                        amp.shake(factor);
                        add.shake(factor);
                    }
                    ParameterValue::LFSquare(init, freq, pw, amp, add, _) => {
                        init.shake(factor);
                        freq.shake(factor);
                        pw.shake(factor);
                        amp.shake(factor);
                        add.shake(factor);
                    }
                }
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
