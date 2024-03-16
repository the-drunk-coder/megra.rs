use std::sync::*;

use crate::{
    builtin_types::GlobalVariables,
    event::{InterpretableEvent, StaticEvent},
    generator::Generator,
    generator_processor::*,
};

/// Apple-ys events to the throughcoming ones
#[derive(Clone)]
pub struct EveryProcessor {
    // optional ID in case we want to preserve state ...
    pub id: Option<String>,
    pub step_count: usize,
    pub things_to_be_applied: Vec<(DynVal, EventsAndFilters, GenModFunsAndArgs)>,
    pub last_static: Vec<StaticEventsAndFilters>, // only needed for events, not filters
}

impl EveryProcessor {
    pub fn new() -> Self {
        EveryProcessor {
            id: None,
            step_count: 1,
            things_to_be_applied: Vec::new(),
            last_static: Vec::new(),
        }
    }
}

impl GeneratorProcessor for EveryProcessor {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_state(&mut self, other: GeneratorProcessorState) {
        if let GeneratorProcessorState::Count(c) = other {
            self.step_count = c;
        }
    }

    fn get_state(&self) -> GeneratorProcessorState {
        GeneratorProcessorState::Count(self.step_count)
    }

    // this one
    fn process_events(&mut self, events: &mut Vec<InterpretableEvent>, _: &Arc<GlobalVariables>) {
        // last_static compiled in `process_transitions`, which is always called first ...
        for static_events in self.last_static.iter() {
            for (filter, inner_static) in static_events.iter() {
                for (static_event, mode) in inner_static.iter() {
                    for in_ev in events.iter_mut() {
                        match in_ev {
                            InterpretableEvent::Sound(s) => {
                                s.apply(static_event, filter, *mode);
                            }
                            InterpretableEvent::Control(_) => {
                                // ??
                            }
                        }
                    }
                }
            }
        }
    }

    fn process_generator(&mut self, gen: &mut Generator, globals: &Arc<GlobalVariables>) {
        for (step, _, gen_mods) in self.things_to_be_applied.iter_mut() {
            // genmodfuns not needed here ...
            let cur_step: usize = step.static_val as usize;
            if self.step_count % cur_step == 0 {
                for (gen_mod_fun, pos_args, named_args) in gen_mods.iter() {
                    gen_mod_fun(gen, pos_args, named_args, globals)
                }
            }
        }
        // finally increment step count, as this is the last one to be handled
        self.step_count += 1;
    }

    fn process_transition(&mut self, trans: &mut StaticEvent, globals: &Arc<GlobalVariables>) {
        // process_transition is always compiled first, so here we compile the static events ...
        self.last_static.clear();

        for (step, filtered_events, _) in self.things_to_be_applied.iter_mut() {
            // genmodfuns not needed here ...
            let cur_step: usize = step.evaluate_numerical() as usize;
            if self.step_count % cur_step == 0 {
                let mut stat_evs = HashMap::new();
                for (filter, (mode, evs)) in filtered_events.iter_mut() {
                    let mut evs_static = Vec::new();
                    for ev in evs.iter_mut() {
                        let ev_static = ev.get_static(globals);

                        evs_static.push((ev_static, *mode));
                    }
                    stat_evs.insert(filter.to_vec(), evs_static);
                }
                self.last_static.push(stat_evs);
            }
        }

        for static_events in self.last_static.iter() {
            for (filter, inner_static) in static_events.iter() {
                for (static_event, mode) in inner_static.iter() {
                    trans.apply(static_event, filter, *mode);
                }
            }
        }
    }
}
