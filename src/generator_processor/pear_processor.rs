use rand::*;
use std::collections::HashMap;
use std::sync::*;

use crate::{
    builtin_types::GlobalVariables,
    event::{InterpretableEvent, StaticEvent},
    generator_processor::*,
    parameter::DynVal,
};

/// Apple-ys events to the throughcoming ones
#[derive(Clone)]
pub struct PearProcessor {
    pub events_to_be_applied: Vec<(DynVal, EventsAndFilters)>,
    pub last_static: Vec<(usize, StaticEventsAndFilters)>,
}

impl PearProcessor {
    pub fn new() -> Self {
        PearProcessor {
            events_to_be_applied: Vec::new(),
            last_static: Vec::new(),
        }
    }
}

// zip mode etc seem to be outdated ... going for any mode for now
impl GeneratorProcessor for PearProcessor {
    // this one only processes the event stream ...
    fn process_events(
        &mut self,
        events: &mut Vec<InterpretableEvent>,
        _: &Arc<GlobalVariables>,
        _functions: &Arc<FunctionMap>,
        _sample_set: SampleAndWavematrixSet,
        _out_mode: OutputMode,
    ) {
        let mut rng = rand::thread_rng();

        for (cur_prob, static_events) in self.last_static.iter() {
            for (filter, inner_static) in static_events.iter() {
                for (static_event, mode) in inner_static.iter() {
                    for in_ev in events.iter_mut() {
                        match in_ev {
                            InterpretableEvent::Sound(s) => {
                                if rng.gen_range(0..100) < *cur_prob {
                                    s.apply(static_event, filter, *mode);
                                }
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
    // .. including transition events
    fn process_transition(
        &mut self,
        trans: &mut StaticEvent,
        globals: &Arc<GlobalVariables>,
        _functions: &Arc<FunctionMap>,
        _sample_set: SampleAndWavematrixSet,
        _out_mode: OutputMode,
    ) {
        // generate static events, as this is always called first ...
        self.last_static.clear();

        for (prob, filtered_events) in self.events_to_be_applied.iter_mut() {
            let mut stat_evs = HashMap::new();
            let cur_prob: usize = (prob.evaluate_numerical() as usize) % 101; // make sure prob is always between 0 and 100
                                                                              //println!("cur p {}", cur_prob);
            for (filter, (mode, evs)) in filtered_events.iter_mut() {
                let mut evs_static = Vec::new();
                for ev in evs.iter_mut() {
                    let ev_static = ev.get_static(globals);

                    evs_static.push((ev_static, *mode));
                }
                stat_evs.insert(filter.to_vec(), evs_static);
            }
            self.last_static.push((cur_prob, stat_evs));
        }

        let mut rng = rand::thread_rng();
        for (prob, filtered_events) in self.last_static.iter_mut() {
            for (filter, evs) in filtered_events.iter_mut() {
                for (ev, mode) in evs.iter() {
                    if (rng.gen_range(0..100) as usize) < *prob {
                        trans.apply(ev, filter, *mode); // not sure
                    }
                }
            }
        }
    }
}
