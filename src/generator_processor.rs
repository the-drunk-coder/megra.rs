use std::collections::HashMap;
use std::sync::*;

use crate::visualizer_client::VisualizerClient;
use std::sync;

use crate::{
    builtin_types::{ConfigParameter, GlobalParameters},
    event::{Event, InterpretableEvent, StaticEvent},
    generator::GenModFun,
    generator::Generator,
    parameter::DynVal,
};

pub enum GeneratorProcessorState {
    Count(usize),
    WrappedGenerator(Generator),
    None,
}

pub trait GeneratorProcessor: GeneratorProcessorClone {
    fn process_events(
        &mut self,
        _events: &mut Vec<InterpretableEvent>,
        _global_parameters: &Arc<GlobalParameters>,
    ) { /* pass by default */
    }
    fn process_generator(
        &mut self,
        _generator: &mut Generator,
        _global_parameters: &Arc<GlobalParameters>,
    ) { /* pass by default */
    }
    fn process_transition(
        &mut self,
        _transition: &mut StaticEvent,
        _global_parameters: &Arc<GlobalParameters>,
    ) { /* pass by default */
    }

    fn set_state(&mut self, _: GeneratorProcessorState) { /* processors are stateless by defalt */
    }

    fn get_state(&self) -> GeneratorProcessorState {
        // processors are stateless by default
        GeneratorProcessorState::None
    }

    fn visualize_if_possible(&mut self, _vis_client: &sync::Arc<VisualizerClient>) {
        /* most won't need this */
    }
    fn clear_visualization(&self, _vis_client: &sync::Arc<VisualizerClient>) { /* most won't need this */
    }
}

pub trait GeneratorProcessorClone {
    fn clone_box(&self) -> Box<dyn GeneratorProcessor + Send>;
}

impl<T> GeneratorProcessorClone for T
where
    T: 'static + GeneratorProcessor + Clone + Send,
{
    fn clone_box(&self) -> Box<dyn GeneratorProcessor + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn GeneratorProcessor + Send> {
    fn clone(&self) -> Box<dyn GeneratorProcessor + Send> {
        self.clone_box()
    }
}

type StaticEventsAndFilters = HashMap<Vec<String>, Vec<StaticEvent>>;
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
