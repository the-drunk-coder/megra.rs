use crate::event::*;
use crate::generator::{GenModFun, Generator};
use crate::generator_processor::GeneratorProcessor;
use crate::parameter::*;
use dashmap::DashMap;
use std::collections::{BTreeSet, HashMap};

use ruffbox_synth::building_blocks::SynthParameterLabel;

#[derive(Clone)]
pub enum Part {
    Combined(Vec<Generator>, Vec<PartProxy>),
}

// might be unified with event parameters at some point but
// i'm not sure how yet ...
#[derive(Clone)]
pub enum ConfigParameter {
    Numeric(f32),
    Dynamic(DynVal),
    Symbolic(String),
}

#[derive(Clone)]
pub enum TypedVariable {
    Part(Part),
    ConfigParameter(ConfigParameter),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum VariableId {
    LifemodelGlobalResources,
    GlobalTimeModifier, // the global factor applied to all durations, usually 1.0
    GlobalLatency,      // latency between language and dsp
    DefaultDuration,    // default duration for two subsequent events (200ms usually)
    DefaultCycleDuration, // defualt duration for a cycle (800ms, or four times the default event duration)
    Custom(String),
}

pub type VariableStore = DashMap<VariableId, TypedVariable>;

#[derive(Clone)]
pub enum SampleResource {
    File(String, Option<String>), // file path, checksum
    Url(String, Option<String>),  // Url path, checksum
}

#[derive(Clone)]
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
    LoadPart((String, Part)),                      // set (events), keyword, path
    StepPart(String),                              // step through specified path
    FreezeBuffer(usize, usize),                    // freeze live buffer
    ExportDotStatic((String, Generator)),          // filename, generator
    ExportDotRunning((String, BTreeSet<String>)),  // filename, generator id
    ExportDotPart((String, String)),               // filename, part name
    Once((Vec<StaticEvent>, Vec<ControlEvent>)),   // execute event(s) once
    ConnectVisualizer,                             // connect visualizer
    StartRecording(Option<String>, bool),          // start recording, prefix, input
    StopRecording,                                 // stop recording ...
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
