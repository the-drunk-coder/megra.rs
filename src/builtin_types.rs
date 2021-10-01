use crate::event::*;
use crate::generator::{GenModFun, Generator};
use crate::generator_processor::GeneratorProcessor;
use crate::markov_sequence_generator::{MarkovSequenceGenerator, Rule};
use crate::parameter::*;
use crate::session::SyncContext;
use dashmap::DashMap;
use std::collections::HashMap;

use ruffbox_synth::ruffbox::synth::SynthParameter;

#[derive(Clone)]
pub enum Part {
    Combined(Vec<Generator>, Vec<PartProxy>),
}

pub type PartsStore = HashMap<String, Part>;

// might be unified with event parameters at some point but
// i'm not sure how yet ...
#[derive(Clone)]
pub enum ConfigParameter {
    Numeric(f32),
    Dynamic(Parameter),
    Symbolic(String),
}

// only one so far
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum BuiltinGlobalParameters {
    LifemodelGlobalResources,
    GlobalTimeModifier,
}

pub type GlobalParameters = DashMap<BuiltinGlobalParameters, ConfigParameter>;

// reflect event hierarchy here, like, Tuned, Param, Sample, Noise ?
pub enum BuiltInParameterEvent {
    PitchFrequency(EventOperation),
    Attack(EventOperation),
    Release(EventOperation),
    Sustain(EventOperation),
    ChannelPosition(EventOperation),
    Level(EventOperation),
    Duration(EventOperation),
    Reverb(EventOperation),
    Delay(EventOperation),
    LpFreq(EventOperation),
    LpQ(EventOperation),
    LpDist(EventOperation),
    PeakFreq(EventOperation),
    PeakQ(EventOperation),
    PeakGain(EventOperation),
    Pulsewidth(EventOperation),
    PlaybackStart(EventOperation),
    PlaybackRate(EventOperation),
}

pub enum BuiltInSoundEvent {
    Sine(EventOperation),
    Tri(EventOperation),
    Cub(EventOperation),
    Saw(EventOperation),
    Square(EventOperation),
    RissetBell(EventOperation),
}

pub enum BuiltInDynamicParameter {
    Bounce,
    Brownian,
    Envelope,
    Fade,
    RandRange, //Oscil,
}

pub enum BuiltInGenProc {
    Inhibit,
    Exhibit,
    InExhibit,
    Pear,
    Apple,
    Every,
    Lifemodel,
}
pub enum BuiltInCompose {}

pub enum BuiltInGenModFun {
    Haste,
    Relax,
    Grow,
    Shrink,
    Blur,
    Sharpen,
    Shake,
    Skip,
    Rewind,
    Rnd,
    Rep,
}

#[derive(Clone, Copy)]
pub enum BuiltInMultiplexer {
    XDup,
    XSpread,
    //XBounce,
    //XRot
}


/// constructor for generators ...
pub enum BuiltInConstructor {
    Learn,
    Infer,
    Rule,
    Nucleus,
    Cycle,
    Fully,
    Flower,
    Friendship,
    Chop,
    Stages,
    // Pseq, ?
}

pub enum BuiltInCommand {
    Clear,
    Tmod,
    GlobRes,
    Delay,
    Reverb,
    ExportDot,
    LoadSample,
    LoadSampleSets,
    LoadSampleSet,
    LoadPart,
    FreezeBuffer,
    Once,
}

/// As this doesn't strive to be a turing-complete lisp, we'll start with the basic
/// megra operations, learning and inferring, plus the built-in events
pub enum BuiltIn {
    Constructor(BuiltInConstructor),
    Silence,
    Command(BuiltInCommand),
    Compose,
    SyncContext,
    Parameter(BuiltInDynamicParameter),
    SoundEvent(BuiltInSoundEvent),
    ControlEvent,
    ParameterEvent(BuiltInParameterEvent),
    GenProc(BuiltInGenProc),
    GenModFun(BuiltInGenModFun),
    Multiplexer(BuiltInMultiplexer),
    GeneratorList
}

#[derive(Clone)]
pub enum Command {
    Clear,                                             // clear the entire session
    Tmod(Parameter),                                   // set global time mod parameter
    GlobRes(f32),                                      // global resources for lifemodel algorithm
    GlobalRuffboxParams(HashMap<SynthParameter, f32>), // global ruffbox params
    LoadSample((String, Vec<String>, String)),         // set (events), keyword, path
    LoadSampleSet(String),                             // set path
    LoadSampleSets(String),                            // top level sets set path
    LoadPart((String, Part)),                          // set (events), keyword, path
    FreezeBuffer(usize),                               // freeze live buffer
    ExportDot((String, Generator)),                    // filename, generator
    Once((Vec<Event>, Vec<ControlEvent>)),
}

#[derive(Clone)]
pub enum PartProxy {
    // part, mods
    Proxy(String, Vec<GeneratorProcessorOrModifier>),
}

#[derive(Clone)]
pub enum GeneratorProcessorOrModifier {
    GeneratorProcessor(Box<dyn GeneratorProcessor + Send>),
    GeneratorModifierFunction(
        (
            GenModFun,
            Vec<ConfigParameter>,
            HashMap<String, ConfigParameter>,
        ),
    ),
}

pub enum Atom {
    // atom might not be the right word any longer
    Float(f32),
    Description(String), // pfa descriptions
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    BuiltIn(BuiltIn),
    MarkovSequenceGenerator(MarkovSequenceGenerator),
    SoundEvent(Event),
    ControlEvent(ControlEvent),
    Rule(Rule),
    Command(Command),
    SyncContext(SyncContext),
    PartProxy(PartProxy),
    ProxyList(Vec<PartProxy>),
    Generator(Generator),
    GeneratorList(Vec<Generator>),
    GeneratorProcessorOrModifier(GeneratorProcessorOrModifier),
    GeneratorProcessorOrModifierList(Vec<GeneratorProcessorOrModifier>),
    Parameter(Parameter),
    Nothing,
}

pub enum Expr {
    Comment,
    Constant(Atom),
    Custom(String),
    Application(Box<Expr>, Vec<Expr>),
}
