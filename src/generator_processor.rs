use std::collections::HashMap;
use std::sync::*;

use crate::visualizer_client::VisualizerClient;
use std::sync;

use crate::{
    builtin_types::{ConfigParameter, GlobalParameters},
    event::{Event, InterpretableEvent, StaticEvent},
    generator::GenModFun,
    generator::Generator,
    parameter::Parameter,
};

pub trait GeneratorProcessor: GeneratorProcessorClone {
    fn process_events(
        &mut self,
        events: &mut Vec<InterpretableEvent>,
        global_parameters: &Arc<GlobalParameters>,
    );
    fn process_generator(
        &mut self,
        generator: &mut Generator,
        global_parameters: &Arc<GlobalParameters>,
    );
    fn process_transition(
        &mut self,
        transition: &mut StaticEvent,
        global_parameters: &Arc<GlobalParameters>,
    );
    fn visualize_if_possible(&mut self, vis_client: &sync::Arc<VisualizerClient>);
    fn clear_visualization(&self, vis_client: &sync::Arc<VisualizerClient>);
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
