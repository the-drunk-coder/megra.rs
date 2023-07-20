use crate::event::*;
use crate::generator::{GenModFun, Generator};
use crate::generator_processor::GeneratorProcessor;
use crate::markov_sequence_generator::Rule;
use crate::parameter::*;

use core::fmt;
use dashmap::DashMap;
use std::collections::{BTreeSet, HashMap};

use ruffbox_synth::building_blocks::SynthParameterLabel;

// might be unified with event parameters at some point but
// i'm not sure how yet ...
#[derive(Clone, Debug)]
pub enum ConfigParameter {
    Numeric(f32),
    Dynamic(DynVal),
    Symbolic(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Comparable {
    Float(f32),
    Double(f64),
    Int32(i32),
    Int64(i64),
    String(String),
    Symbol(String),
    Character(char),
    Boolean(bool),
}

#[derive(Clone, Debug)]
pub enum TypedEntity {
    Comparable(Comparable),
    ConfigParameter(ConfigParameter),
    Generator(Generator),
    GeneratorProcessorOrModifier(GeneratorProcessorOrModifier),
    ControlEvent(ControlEvent),
    SoundEvent(Event),
    Parameter(DynVal),
    ParameterValue(ParameterValue),
    Rule(Rule),
    // compound types, some specialized for convenience
    Vec(Vec<Box<TypedEntity>>),
    Pair(Box<TypedEntity>, Box<TypedEntity>),
    Matrix(Vec<Vec<Box<TypedEntity>>>),
    GeneratorList(Vec<Generator>),
    GeneratorProcessorOrModifierList(Vec<GeneratorProcessorOrModifier>),
    GeneratorModifierList(Vec<GeneratorProcessorOrModifier>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum VariableId {
    LifemodelGlobalResources,
    GlobalTimeModifier, // the global factor applied to all durations, usually 1.0
    GlobalLatency,      // latency between language and dsp
    DefaultDuration,    // default duration for two subsequent events (200ms usuallyd)
    DefaultCycleDuration, // default duration for a cycle (800ms, or four times the default event duration)
    Custom(String),
    Symbol(String),
}

pub type VariableStore = DashMap<VariableId, TypedEntity>;

#[derive(Clone, Debug)]
pub enum SampleResource {
    File(String, Option<String>), // file path, checksum
    Url(String, Option<String>),  // Url path, checksum
}

#[derive(Clone, Debug)]
pub enum Command {
    Clear,                                                               // clear the entire session
    Tmod(DynVal),         // set global time mod parameter
    Latency(DynVal),      // set global latency parameter
    Bpm(f32),             // set default tempo in bpm
    DefaultDuration(f32), // set default duration in milliseconds
    GlobRes(f32),         // global resources for lifemodel algorithm
    GlobalRuffboxParams(HashMap<SynthParameterLabel, ParameterValue>), // global ruffbox params
    LoadSampleAsWavematrix(String, String, String, (usize, usize), f32), // key, path, method, matrix size, start
    ImportSampleSet(SampleResource),
    LoadSample(String, Vec<String>, String, bool), // set (events), keyword, path, downmix_stereo
    LoadSampleSet(String, bool),                   // set path, downmix stereo
    LoadSampleSets(String, bool),                  // top level sets set path
    StepPart(String),                              // step through specified path
    FreezeBuffer(usize, usize),                    // freeze live buffer
    ExportDotStatic(String, Generator),            // filename, generator
    ExportDotRunning((String, BTreeSet<String>)),  // filename, generator id
    Once(Vec<StaticEvent>, Vec<ControlEvent>),     // execute event(s) once
    ConnectVisualizer,                             // connect visualizer
    StartRecording(Option<String>, bool),          // start recording, prefix, input
    StopRecording,                                 // stop recording ...
    OscDefineClient(String, String),
    OscSendMessage(String, String, Vec<TypedEntity>),
    OscStartReceiver(String),
    MidiStartReceiver(usize),
    MidiListPorts,
    Print(TypedEntity),
    Push(VariableId, TypedEntity),
}

#[derive(Clone)]
pub enum PartProxy {
    // part, mods
    Proxy(String, Vec<GeneratorProcessorOrModifier>),
}

#[derive(Clone)]
pub enum GeneratorProcessorOrModifier {
    GeneratorProcessor(Box<dyn GeneratorProcessor + Send + Sync>),
    GeneratorModifierFunction(
        (
            GenModFun,
            Vec<ConfigParameter>,
            HashMap<String, ConfigParameter>,
        ),
    ),
}

impl fmt::Debug for GeneratorProcessorOrModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GenProcOrMod")
    }
}
