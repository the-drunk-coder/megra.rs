use rand::*;
use std::collections::HashMap;
use std::sync::*;

use crate::{
    builtin_types::GlobalParameters,
    event::{InterpretableEvent, StaticEvent},
    generator::TimeMod,
    generator_processor::*,
    markov_sequence_generator::MarkovSequenceGenerator,
    parameter::Parameter,
};

/// Apple-ys events to the throughcoming ones
#[derive(Clone)]
pub struct PearProcessor {
    pub events_to_be_applied: Vec<(Parameter, EventsAndFilters)>,
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
    fn process_generator(
        &mut self,
        _: &mut MarkovSequenceGenerator,
        _: &Arc<GlobalParameters>,
        _: &mut Vec<TimeMod>,
    ) { /* pass */
    }

    fn process_events(&mut self, events: &mut Vec<InterpretableEvent>, _: &Arc<GlobalParameters>) {
        self.last_static.clear();
        let mut rng = rand::thread_rng();
        // the four nested loops are intimidating but keep in mind that the
        // event count is usually very small ...
        for (prob, filtered_events) in self.events_to_be_applied.iter_mut() {
            let mut stat_evs = HashMap::new();
            let cur_prob: usize = (prob.evaluate() as usize) % 101; // make sure prob is always between 0 and 100
                                                                    //println!("cur p {}", cur_prob);
            for (filter, evs) in filtered_events.iter_mut() {
                let mut evs_static = Vec::new();
                for ev in evs.iter_mut() {
                    let ev_static = ev.get_static();
                    for in_ev in events.iter_mut() {
                        match in_ev {
                            InterpretableEvent::Sound(s) => {
                                if rng.gen_range(0, 100) < cur_prob {
                                    s.apply(&ev_static, filter, true);
                                }
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
            self.last_static.push((cur_prob, stat_evs));
        }
    }

    fn process_transition(&mut self, trans: &mut StaticEvent, _: &Arc<GlobalParameters>) {
        let mut rng = rand::thread_rng();
        for (prob, filtered_events) in self.last_static.iter_mut() {
            for (filter, evs) in filtered_events.iter_mut() {
                for ev in evs.iter() {
                    if (rng.gen_range(0, 100) as usize) < *prob {
                        trans.apply(&ev, filter, true); // not sure
                    }
                }
            }
        }
    }
}
