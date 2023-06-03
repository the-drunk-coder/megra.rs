use std::sync::*;

use crate::{
    builtin_types::VariableStore,
    event::{InterpretableEvent, StaticEvent},
    generator::Generator,
    generator_processor::*,
};

/// Apple-ys events to the throughcoming ones
#[derive(Clone)]
pub struct GeneratorWrapperProcessor {
    wrapped_generator: Generator,
    current_events: Vec<InterpretableEvent>,
    filter: Vec<String>,
}

impl GeneratorWrapperProcessor {
    pub fn with_generator(gen: Generator) -> Self {
        GeneratorWrapperProcessor {
            wrapped_generator: gen,
            current_events: Vec::new(),
            filter: vec!["".to_string()],
        }
    }
}

// zip mode etc seem to be outdated ... going for any mode for now
impl GeneratorProcessor for GeneratorWrapperProcessor {
    /// id helps us to preserve state ...
    fn get_id(&self) -> Option<String> {
        let mut id = "".to_string();
        for tag in self.wrapped_generator.id_tags.iter() {
            id.push_str(tag);
        }
        Some(id)
    }

    fn set_state(&mut self, other: GeneratorProcessorState) {
        if let GeneratorProcessorState::WrappedGenerator(g) = other {
            //println!("transfer state");
            self.wrapped_generator.transfer_state(&g);
        }
    }

    fn get_state(&self) -> GeneratorProcessorState {
        GeneratorProcessorState::WrappedGenerator(self.wrapped_generator.clone())
    }

    // another pure event-stream processor
    fn process_events(&mut self, events: &mut Vec<InterpretableEvent>, _glob: &Arc<VariableStore>) {
        for ev in self.current_events.iter_mut() {
            if let InterpretableEvent::Sound(sev) = ev {
                for in_ev in events.iter_mut() {
                    match in_ev {
                        InterpretableEvent::Sound(s) => {
                            s.apply(sev, &self.filter, true);
                            s.tags = sev.tags.union(&s.tags).cloned().collect();
                        }
                        InterpretableEvent::Control(_) => {
                            // ??
                        }
                    }
                }
            }
        }
    }

    fn process_transition(&mut self, trans: &mut StaticEvent, glob: &Arc<VariableStore>) {
        self.wrapped_generator.current_transition(glob);

        // already get current events here so we have the same execution
        // order and still can properly process the first transition
        self.current_events = self.wrapped_generator.current_events(glob);

        for ev in self.current_events.iter_mut() {
            if let InterpretableEvent::Sound(sev) = ev {
                trans.apply(sev, &self.filter, true);
            }
        }
    }

    fn visualize_if_possible(&mut self, vis_client: &sync::Arc<VisualizerClient>) {
        if self.wrapped_generator.root_generator.is_modified() {
            vis_client.create_or_update(&self.wrapped_generator);
            self.wrapped_generator.root_generator.clear_modified();
        }
        vis_client.update_active_node(&self.wrapped_generator);
        for (_, proc) in self.wrapped_generator.processors.iter_mut() {
            proc.visualize_if_possible(vis_client);
        }
    }

    fn clear_visualization(&self, vc: &sync::Arc<VisualizerClient>) {
        vc.clear(&self.wrapped_generator.id_tags);
        for (_, proc) in self.wrapped_generator.processors.iter() {
            proc.clear_visualization(vc);
        }
    }
}
