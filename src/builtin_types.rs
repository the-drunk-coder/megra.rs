use crate::markov_sequence_generator::{Rule, MarkovSequenceGenerator};
use crate::event::*;
use crate::parameter::*;
use crate::generator_processor::GeneratorProcessor;
use crate::generator::Generator;
use crate::session::SyncContext;
use std::collections::{HashMap, HashSet};

/// maps an event type (like "bd") to a mapping between keywords and buffer number ...
pub type SampleSet = HashMap<String, Vec<(HashSet<String>, usize)>>;

// reflect event hierarchy here, like, Tuned, Param, Sample, Noise ?
pub enum BuiltInEvent {
    Level(EventOperation),
    Reverb(EventOperation),
    Duration(EventOperation),
    //LpQ(EventOperation),
    //LpDist(EventOperation),
    Sine(EventOperation),
    Saw(EventOperation),
    Square(EventOperation),
}

pub enum BuiltInDynamicParameter {
    Bounce,
    //Brownian,
    //Oscil,
    //Env,
    //RandRange
}

pub enum BuiltInGenProc {
    Pear,
    // Apple,    
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
    SyncContext,
    Parameter(BuiltInDynamicParameter),
    SoundEvent(BuiltInEvent),
    ModEvent(BuiltInEvent),
    GenProc(BuiltInGenProc),
}

pub enum Command {
    Clear,
    LoadSample((String, Vec<String>, String)) // set (events), keyword, path
}

pub enum Atom { // atom might not be the right word any longer 
    Float(f32),
    Description(String), // pfa descriptions
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    BuiltIn(BuiltIn),
    MarkovSequenceGenerator(MarkovSequenceGenerator),
    Event(Event),
    Rule(Rule),
    Command(Command),
    SyncContext(SyncContext),
    Generator(Generator),
    GeneratorProcessor(Box<dyn GeneratorProcessor>),
    GeneratorProcessorList(Vec<Box<dyn GeneratorProcessor>>),
    Parameter(Parameter),
    Nothing
}

pub enum Expr {
    Constant(Atom),
    Custom(String),
    Application(Box<Expr>, Vec<Expr>),    
}
