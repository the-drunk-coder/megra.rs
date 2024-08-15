use core::fmt;
use ruffbox_synth::building_blocks::{
    EnvelopeSegmentInfo, EnvelopeSegmentType, SynthParameterAddress, SynthParameterLabel,
    SynthParameterValue, ValOp,
};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::*;

use crate::builtin_types::Command;
use crate::parameter::{resolve_parameter, shake_parameter, ParameterValue};
use crate::sample_set::SampleLookup;
use crate::session::SyncContext;
use crate::synth_parameter_value_arithmetic::*;
use crate::GlobalVariables;

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
    pub params: HashMap<SynthParameterAddress, ParameterValue>,
    pub tags: BTreeSet<String>,
    pub op: EventOperation,
    // sample lookup is handled apart from the
    // parameters, as this makes things much easier ...
    pub sample_lookup: Option<SampleLookup>,
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
    pub params: HashMap<SynthParameterAddress, SynthParameterValue>,
    pub tags: BTreeSet<String>,
    pub op: EventOperation,
    pub sample_lookup: Option<SampleLookup>,
}

/// A ControlEvent can call any function when interpreted.
#[derive(Clone)]
pub struct ControlEvent {
    pub tags: BTreeSet<String>,
    pub ctx: Option<Vec<SyncContext>>,
    pub cmd: Option<Vec<Command>>,
}

impl fmt::Debug for ControlEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ControlEvent({:#?})", self.tags)
    }
}

/// This is the "latent" event, where the parameters haven't been evaluated yet.
#[derive(Clone)]
pub enum SourceEvent {
    Sound(Event),
    Control(ControlEvent),
}

/// This is the "final" event after all the parameters have been evaluated,
/// so that it can be interpreted to either actual sound or control.
#[derive(Clone, Debug)]
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

        for (ko, vo) in other.params.iter() {
            let mut label_found = false;

            for (ks, vs) in self.params.iter_mut() {
                // if the labels don't match, don't do anything
                if ko.label == ks.label {
                    // found label of incoming in current param map
                    //
                    // if the incoming event has an index specified,
                    // apply if the index matches
                    if let Some(idxo) = ko.idx {
                        if let Some(idxs) = ks.idx {
                            if idxs == idxo {
                                *vs = calc_spv(vs, vo, other.op);
                            }
                        }
                    } else {
                        // if the incoming has no index specified,
                        // apply to all self params that have the
                        // same label
                        *vs = calc_spv(vs, vo, other.op);
                    }
                    label_found = true;
                }
            }

            // if the label hasn't been found at all,
            // add the parameter
            if !label_found {
                self.params.insert(*ko, vo.clone());
            }
        }

        // handle sample lookup "arithmetic"
        match (self.sample_lookup.as_mut(), other.sample_lookup.as_ref()) {
            (Some(my_lookup), Some(SampleLookup::Key(_, other_keys))) => {
                match my_lookup {
                    // key "arithmetic"
                    SampleLookup::Key(_, my_keys) => {
                        match other.op {
                            EventOperation::Add => {
                                // add all keys from the other set
                                for key in other_keys.iter() {
                                    my_keys.insert(key.clone());
                                }
                            }
                            EventOperation::Subtract => {
                                for key in other_keys.iter() {
                                    my_keys.remove(key);
                                }
                            }
                            _ => {
                                my_keys.clear();
                                for key in other_keys.iter() {
                                    my_keys.insert(key.clone());
                                }
                            }
                        }
                    }
                    // replace fixed random by keys
                    SampleLookup::FixedRandom(fname, _) => {
                        *my_lookup = SampleLookup::Key(fname.to_string(), other_keys.clone());
                    }
                    SampleLookup::Random(fname) => {
                        *my_lookup = SampleLookup::Key(fname.to_string(), other_keys.clone());
                    }
                    SampleLookup::N(fname, _) => {
                        *my_lookup = SampleLookup::Key(fname.to_string(), other_keys.clone());
                    }
                }
            }
            (Some(my_lookup), Some(SampleLookup::Random(_)))
                if matches!(other.op, EventOperation::Replace) =>
            {
                match my_lookup {
                    // replace by randomness
                    SampleLookup::Key(fname, _) => {
                        *my_lookup = SampleLookup::Random(fname.to_string());
                    }
                    SampleLookup::FixedRandom(fname, _) => {
                        *my_lookup = SampleLookup::Random(fname.to_string());
                    }
                    SampleLookup::Random(fname) => {
                        *my_lookup = SampleLookup::Random(fname.to_string());
                    }
                    SampleLookup::N(fname, _) => {
                        *my_lookup = SampleLookup::Random(fname.to_string());
                    }
                }
            }
            (Some(my_lookup), Some(SampleLookup::N(_, n))) => {
                match my_lookup {
                    // replace by randomness
                    SampleLookup::Key(fname, _) => {
                        *my_lookup = SampleLookup::N(fname.to_string(), *n);
                    }

                    SampleLookup::FixedRandom(fname, _) => {
                        *my_lookup = SampleLookup::N(fname.to_string(), *n);
                    }
                    SampleLookup::Random(fname) => {
                        *my_lookup = SampleLookup::N(fname.to_string(), *n);
                    }
                    SampleLookup::N(_, original_n) => match other.op {
                        EventOperation::Add => {
                            *original_n += *n;
                        }
                        EventOperation::Subtract => {
                            if *original_n >= *n {
                                *original_n -= *n;
                            } else {
                                *original_n = 0;
                            }
                        }
                        EventOperation::Multiply => {
                            *original_n *= *n;
                        }
                        EventOperation::Divide => {
                            *original_n /= *n;
                        }
                        EventOperation::Replace => {
                            *original_n = *n;
                        }
                    },
                }
            }
            _ => {}
        }
    }

    /// collect the envelope information and compile a
    /// single multi-point envelope
    pub fn build_envelope(&mut self) {
        // if this event already has a complete envelope,
        // we don't need to do anything ...
        if self
            .params
            .contains_key(&SynthParameterLabel::Envelope.into())
        {
            return;
        }

        let mut segments = Vec::new();

        let sustain_level = if let Some(SynthParameterValue::ScalarF32(a)) = self
            .params
            .remove(&SynthParameterLabel::EnvelopeLevel.into())
        {
            a
        } else {
            0.7
        };

        let attack_level = if let Some(SynthParameterValue::ScalarF32(a)) = self
            .params
            .remove(&SynthParameterLabel::AttackPeakLevel.into())
        {
            a
        } else {
            sustain_level
        };

        // ADSR values are specified as milliseconds,
        // hence some conversion is necessary

        // ATTACK
        if let Some(SynthParameterValue::ScalarF32(a)) =
            self.params.remove(&SynthParameterLabel::Attack.into())
        {
            segments.push(EnvelopeSegmentInfo {
                from: 0.0,
                to: attack_level,
                time: a * 0.001, // ms to sec
                segment_type: if let Some(SynthParameterValue::EnvelopeSegmentType(e)) =
                    self.params.remove(&SynthParameterLabel::AttackType.into())
                {
                    e
                } else {
                    EnvelopeSegmentType::Lin
                },
            });
        }

        // DECAY (if applicable)
        if let Some(SynthParameterValue::ScalarF32(d)) =
            self.params.remove(&SynthParameterLabel::Decay.into())
        {
            segments.push(EnvelopeSegmentInfo {
                from: attack_level,
                to: sustain_level,
                time: d * 0.001, // ms to sec
                segment_type: if let Some(SynthParameterValue::EnvelopeSegmentType(e)) =
                    self.params.remove(&SynthParameterLabel::DecayType.into())
                {
                    e
                } else {
                    EnvelopeSegmentType::Lin
                },
            });
        }

        // SUSTAIN
        if let Some(SynthParameterValue::ScalarF32(s)) =
            self.params.remove(&SynthParameterLabel::Sustain.into())
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
            self.params.remove(&SynthParameterLabel::Release.into())
        {
            segments.push(EnvelopeSegmentInfo {
                from: sustain_level,
                to: 0.0,
                time: r * 0.001, // ms to sec
                segment_type: if let Some(SynthParameterValue::EnvelopeSegmentType(e)) =
                    self.params.remove(&SynthParameterLabel::ReleaseType.into())
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
                SynthParameterLabel::Envelope.into(),
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
            sample_lookup: None,
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
            sample_lookup: None,
        }
    }

    pub fn evaluate_parameters(
        &mut self,
        globals: &std::sync::Arc<GlobalVariables>,
    ) -> HashMap<SynthParameterAddress, SynthParameterValue> {
        let mut map = HashMap::new();

        for (k, v) in self.params.iter_mut() {
            map.insert(*k, resolve_parameter(k.label, v, globals));
        }

        map
    }

    pub fn shake(&mut self, factor: f32, keep: &HashSet<SynthParameterLabel>) {
        for (k, v) in self.params.iter_mut() {
            if !keep.contains(&k.label) && k.label != SynthParameterLabel::SampleBufferNumber {
                shake_parameter(v, factor);
            }
        }
    }

    pub fn get_static(&mut self, globals: &std::sync::Arc<GlobalVariables>) -> StaticEvent {
        StaticEvent {
            name: self.name.clone(),
            params: self.evaluate_parameters(globals),
            tags: self.tags.clone(),
            op: self.op,
            sample_lookup: self.sample_lookup.clone(),
        }
    }
}
