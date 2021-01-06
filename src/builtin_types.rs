use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;
use crate::generator_processor::GeneratorProcessor;
use crate::generator::{Generator, GenModFun};
use crate::session::SyncContext;
use std::collections::HashMap;
use dashmap::DashMap;

//pub type SampleSet = HashMap<String, Vec<(HashSet<String>, usize)>>;
pub type PartsStore = HashMap<String, Vec<Generator>>;

// might be unified with event parameters at some point but
// i'm not sure how yet ...
#[derive(Clone)]
pub enum ConfigParameter {
    Numeric(f32),
    Symbolic(String)
}

// only one so far 
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum BuiltinGlobalParameters {
    LifemodelGlobalResources,    
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
    Saw(EventOperation),
    Square(EventOperation),
}

pub enum BuiltInDynamicParameter {
    Bounce,
    Brownian,
    Envelope,
    Fade,
    RandRange
    //Oscil,    
}

pub enum BuiltInGenProc {
    Pear,
    Apple,
    Every,
    Lifemodel
}

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
}

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
}

pub enum BuiltInCommand {
    Clear,
    LoadSample,
    LoadSampleSets,
    LoadSampleSet,
    LoadPart,
}

/// As this doesn't strive to be a turing-complete lisp, we'll start with the basic
/// megra operations, learning and inferring, plus the built-in events
pub enum BuiltIn {
    Constructor(BuiltInConstructor),
    Silence,
    Command(BuiltInCommand),
    SyncContext,
    Parameter(BuiltInDynamicParameter),
    SoundEvent(BuiltInSoundEvent),
    ControlEvent,
    ParameterEvent(BuiltInParameterEvent),
    GenProc(BuiltInGenProc),
    GenModFun(BuiltInGenModFun),
    Multiplexer(BuiltInMultiplexer),
}

pub enum Command {
    Clear,
    LoadSample((String, Vec<String>, String)) ,// set (events), keyword, path
    LoadSampleSet(String), // set path
    LoadSampleSets(String), // top level sets set path
    LoadPart((String, Vec<Generator>)) // set (events), keyword, path
}

pub enum Atom { // atom might not be the right word any longer 
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
    Generator(Generator),
    GeneratorProcessor(Box<dyn GeneratorProcessor + Send>),
    GeneratorProcessorList(Vec<Box<dyn GeneratorProcessor + Send>>),
    GeneratorList(Vec<Generator>),
    Parameter(Parameter),
    GeneratorModifierFunction((GenModFun, Vec<ConfigParameter>, HashMap<String, ConfigParameter>)),
    Nothing
}

pub enum Expr {
    Comment,
    Constant(Atom),
    Custom(String),
    Application(Box<Expr>, Vec<Expr>),    
}
