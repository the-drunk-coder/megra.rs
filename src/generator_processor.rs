use std::collections::{BTreeSet, HashMap};
use std::sync::*;

use crate::eval::FunctionMap;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::OutputMode;
use crate::visualizer_client::VisualizerClient;

use crate::{
    builtin_types::{ConfigParameter, GlobalVariables},
    event::{Event, InterpretableEvent, StaticEvent},
    generator::GenModFun,
    generator::Generator,
    parameter::DynVal,
};

pub enum GeneratorProcessorState {
    Count(usize),
    WrappedGenerator(Generator),
}

/// the generator processor only needs to implement
/// a subset of the methods available ...
pub trait GeneratorProcessor: GeneratorProcessorClone {
    /// some generator processors have a state flag,
    /// others are stateless ...
    /// the ones that have a state need an ID
    fn get_id(&self) -> Option<String> {
        None
    }

    fn collect_id_set(&self, _supplemental: &mut BTreeSet<BTreeSet<String>>) {}

    /// some processors need some internal IDs to be updated
    /// (mostly for visualization)
    /// (so far only the GeneratorWrapperProcessor)
    fn inherit_id(&mut self, _: &BTreeSet<String>) {}

    /// implement this if you want to modify the previous
    /// processor's event stream
    fn process_events(
        &mut self,
        _events: &mut Vec<InterpretableEvent>,
        _globals: &Arc<GlobalVariables>,
        _functions: &Arc<FunctionMap>,
        _sample_set: SampleAndWavematrixSet,
        _out_mode: OutputMode,
    ) {
        /* pass by default */
    }
    /// implement this if you need to modify the previous
    /// processor's structure
    fn process_generator(&mut self, _generator: &mut Generator, _globals: &Arc<GlobalVariables>) {
        /* pass by default */
    }
    /// implement this if you need to modify the transitions
    /// between events ...
    fn process_transition(
        &mut self,
        _transition: &mut StaticEvent,
        _globals: &Arc<GlobalVariables>,
        _functions: &Arc<FunctionMap>,
        _sample_set: SampleAndWavematrixSet,
        _out_mode: OutputMode,
    ) {
        /* pass by default */
    }

    /// implement this if the processor has a state, such as a step
    /// counter
    fn set_state(&mut self, _: GeneratorProcessorState) {
        /* processors are stateless by defalt */
    }

    /// implement this if the processor has a state, such as a step
    /// counter
    fn get_state(&self) -> Option<GeneratorProcessorState> {
        // processors are stateless by default
        None
    }

    /// if the processor holds something that can be visualized
    /// such as a markov chain ...
    fn visualize_if_possible(&mut self, _vis_client: &VisualizerClient) {
        /* most won't need this */
    }

    /// only if visualization is possible ...
    fn clear_visualization(&self, _vis_client: &VisualizerClient) {
        /* most won't need this */
    }
}

pub trait GeneratorProcessorClone {
    fn clone_box(&self) -> Box<dyn GeneratorProcessor + Sync + Send>;
}

impl<T> GeneratorProcessorClone for T
where
    T: 'static + GeneratorProcessor + Clone + Sync + Send,
{
    fn clone_box(&self) -> Box<dyn GeneratorProcessor + Sync + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn GeneratorProcessor + Sync + Send> {
    fn clone(&self) -> Box<dyn GeneratorProcessor + Sync + Send> {
        self.clone_box()
    }
}

type StaticEventsAndFilters = HashMap<Vec<String>, Vec<(StaticEvent, bool)>>;
type EventsAndFilters = HashMap<Vec<String>, (bool, Vec<Event>)>;
type GenModFunsAndArgs = Vec<(
    GenModFun,
    Vec<ConfigParameter>,
    HashMap<String, ConfigParameter>,
)>;

mod pear_processor;
pub use pear_processor::*;

mod apple_processor;
pub use apple_processor::*;

mod every_processor;
pub use every_processor::*;

mod lifemodel_processor;
pub use lifemodel_processor::*;

mod generator_wrapper_processor;
pub use generator_wrapper_processor::*;

mod mapper;
pub use mapper::*;
