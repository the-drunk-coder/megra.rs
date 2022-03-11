use std::sync::*;

use crate::{
    builtin_types::GlobalParameters,
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
    fn process_generator(&mut self, _: &mut Generator, _: &Arc<GlobalParameters>) {
        /* pass */
    }

    fn process_events(
        &mut self,
        events: &mut Vec<InterpretableEvent>,
        glob: &Arc<GlobalParameters>,
    ) {
        self.current_events = self.wrapped_generator.current_events(glob);

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

    fn process_transition(&mut self, trans: &mut StaticEvent, glob: &Arc<GlobalParameters>) {
        for ev in self.current_events.iter_mut() {
            if let InterpretableEvent::Sound(sev) = ev {
                trans.apply(sev, &self.filter, true);
            }
        }
        self.wrapped_generator.current_transition(glob);
    }

    fn visualize_if_possible(&mut self, vis_client: &sync::Arc<VisualizerClient>) {
        if self.wrapped_generator.root_generator.is_modified() {
            vis_client.create_or_update(&self.wrapped_generator);
            self.wrapped_generator.root_generator.clear_modified();
        }
        vis_client.update_active_node(&self.wrapped_generator);
        for proc in self.wrapped_generator.processors.iter_mut() {
            proc.visualize_if_possible(vis_client);
        }
    }

    fn clear_visualization(&self, vc: &sync::Arc<VisualizerClient>) {
        vc.clear(&self.wrapped_generator.id_tags);
        for proc in self.wrapped_generator.processors.iter() {
            proc.clear_visualization(vc);
        }
    }
}
