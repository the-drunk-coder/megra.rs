use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;
use crate::generator_processor::GeneratorProcessor;
use crate::generator::{Generator, GenModFun};
use crate::session::SyncContext;
use std::collections::{HashMap, HashSet};

/// maps an event type (like "bd") to a mapping between keywords and buffer number ...
pub type SampleSet = HashMap<String, Vec<(HashSet<String>, usize)>>;
pub type PartsStore = HashMap<String, Vec<Generator>>;

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
    //Oscil,
    //Env,
    //RandRange
}

pub enum BuiltInGenProc {
    Pear,
    Apple,
    Every
}

pub enum BuiltInGenModFun {
    Haste,
    Relax,
    Grow,
    //Rew,
    //Skip,
    
}

pub enum BuiltInMultiplexer {
    XDup,
    XSpread,
    //XBounce,
    //XRot
}

/// As this doesn't strive to be a turing-complete lisp, we'll start with the basic
/// megra operations, learning and inferring, plus the built-in events
pub enum BuiltIn {
    Learn,
    Infer,    
    Rule,
    Clear,
    Silence,
    LoadSample,
    LoadPart,
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
    GeneratorModifierFunction((GenModFun, Vec<f32>)),
    Nothing
}

pub enum Expr {
    Constant(Atom),
    Custom(String),
    Application(Box<Expr>, Vec<Expr>),    
}
