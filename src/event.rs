use ruffbox_synth::building_blocks::{
    EnvelopeSegmentInfo, EnvelopeSegmentType, SynthParameterLabel, SynthParameterValue, ValOp,
};
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
#[derive(Clone, Debug)]
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

    /// collect the envelope information and compile a
    /// single multi-point envelope
    pub fn build_envelope(&mut self) {
        // if this event already has a complete envelope,
        // we don't need to do anything ...
        if self.params.contains_key(&SynthParameterLabel::Envelope) {
            return;
        }

        let mut segments = Vec::new();

        let sustain_level = if let Some(SynthParameterValue::ScalarF32(a)) =
            self.params.remove(&SynthParameterLabel::EnvelopeLevel)
        {
            a
        } else {
            0.7
        };

        let attack_level = if let Some(SynthParameterValue::ScalarF32(a)) =
            self.params.remove(&SynthParameterLabel::AttackPeakLevel)
        {
            a
        } else {
            sustain_level
        };

        // ADSR values are specified as milliseconds,
        // hence some conversion is necessary

        // ATTACK
        if let Some(SynthParameterValue::ScalarF32(a)) =
            self.params.remove(&SynthParameterLabel::Attack)
        {
            segments.push(EnvelopeSegmentInfo {
                from: 0.0,
                to: attack_level,
                time: a * 0.001, // ms to sec
                segment_type: if let Some(SynthParameterValue::EnvelopeSegmentType(e)) =
                    self.params.remove(&SynthParameterLabel::AttackType)
                {
                    e
                } else {
                    EnvelopeSegmentType::Lin
                },
            });
        }

        // DECAY (if applicable)
        if let Some(SynthParameterValue::ScalarF32(d)) =
            self.params.remove(&SynthParameterLabel::Decay)
        {
            segments.push(EnvelopeSegmentInfo {
                from: attack_level,
                to: sustain_level,
                time: d * 0.001, // ms to sec
                segment_type: if let Some(SynthParameterValue::EnvelopeSegmentType(e)) =
                    self.params.remove(&SynthParameterLabel::DecayType)
                {
                    e
                } else {
                    EnvelopeSegmentType::Lin
                },
            });
        }

        // SUSTAIN
        if let Some(SynthParameterValue::ScalarF32(s)) =
            self.params.remove(&SynthParameterLabel::Sustain)
        {
            segments.push(EnvelopeSegmentInfo {
                from: sustain_level,
                to: sustain_level,
                time: s * 0.001, // ms to sec
                segment_type: EnvelopeSegmentType::Constant,
            });
        }

        // RELEASE
        if let Some(SynthParameterValue::ScalarF32(r)) =
            self.params.remove(&SynthParameterLabel::Release)
        {
            segments.push(EnvelopeSegmentInfo {
                from: sustain_level,
                to: 0.0,
                time: r * 0.001, // ms to sec
                segment_type: if let Some(SynthParameterValue::EnvelopeSegmentType(e)) =
                    self.params.remove(&SynthParameterLabel::ReleaseType)
                {
                    e
                } else {
                    EnvelopeSegmentType::Lin
                },
            });
        }

        // only add if there's actual envelope info to be found ...
        if !segments.is_empty() {
            self.params.insert(
                SynthParameterLabel::Envelope,
                SynthParameterValue::MultiPointEnvelope(segments, false, ValOp::Replace),
            );
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
        let mut static_event = StaticEvent {
            name: self.name.clone(),
            params: self.evaluate_parameters(),
            tags: self.tags.clone(),
            op: self.op,
        };
        // before we send the event, make sure we have a self-contained
        // envelope (building and changing the envelope incrementally
        // is a bit annoying later down the line)
        static_event.build_envelope();

        static_event
    }
}
