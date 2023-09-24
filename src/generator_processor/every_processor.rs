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
    pub last_static: Vec<(usize, StaticEventsAndFilters)>, // only needed for events, not filters
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
    fn process_events(
        &mut self,
        events: &mut Vec<InterpretableEvent>,
        globals: &Arc<GlobalVariables>,
    ) {
        self.last_static.clear();
        for (step, filtered_events, _) in self.things_to_be_applied.iter_mut() {
            // genmodfuns not needed here ...
            let cur_step: usize = (step.evaluate_numerical() as usize) % 101; // make sure prob is always between 0 and 100
            if self.step_count % cur_step == 0 {
                let mut stat_evs = HashMap::new();
                for (filter, (mode, evs)) in filtered_events.iter_mut() {
                    let mut evs_static = Vec::new();
                    for ev in evs.iter_mut() {
                        let ev_static = ev.get_static(globals);
                        for in_ev in events.iter_mut() {
                            match in_ev {
                                InterpretableEvent::Sound(s) => {
                                    s.apply(&ev_static, filter, *mode);
                                }
                                InterpretableEvent::Control(_) => {
                                    // ??
                                }
                            }
                        }
                        evs_static.push(ev_static);
                    }
                    stat_evs.insert(filter.to_vec(), evs_static);
                }
                self.last_static.push((cur_step, stat_evs));
            }
        }
    }

    fn process_generator(&mut self, gen: &mut Generator, globals: &Arc<GlobalVariables>) {
        for (step, _, gen_mods) in self.things_to_be_applied.iter_mut() {
            // genmodfuns not needed here ...
            let cur_step: usize = (step.static_val as usize) % 101;
            if self.step_count % cur_step == 0 {
                for (gen_mod_fun, pos_args, named_args) in gen_mods.iter() {
                    gen_mod_fun(gen, pos_args, named_args, globals)
                }
            }
        }
        // finally increment step count, as this is the last one to be handled
        self.step_count += 1;
    }

    fn process_transition(&mut self, trans: &mut StaticEvent, _: &Arc<GlobalVariables>) {
        for (cur_step, filtered_events) in self.last_static.iter() {
            if self.step_count % cur_step == 0 {
                for (filter, evs) in filtered_events.iter() {
                    for ev in evs.iter() {
                        trans.apply(ev, filter, true); // not sure
                    }
                }
            }
        }
    }
}
