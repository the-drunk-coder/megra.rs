use crate::event::{Event, InterpretableEvent, SourceEvent, StaticEvent};
use crate::parameter::DynVal;
use crate::GlobalVariables;
use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
use std::collections::{BTreeMap, HashMap};
use vom_rs::pfa::{self, Label};

#[derive(Clone, Debug)]
pub struct Rule {
    pub source: Vec<char>,
    pub symbol: char,
    pub probability: f32,
    pub duration: u64,
}

impl Rule {
    pub fn to_pfa_rule(&self) -> pfa::Rule<char> {
        pfa::Rule {
            source: self.source.clone(),
            symbol: self.symbol,
            probability: self.probability,
        }
    }
}

/*

about the time model:

An "event" is a sound- or control event AND the duration to the following
event.

then there's "override" durations, i.e. if you want to specify a separate duration
after a certain number of repetitions to the next symbol

i.e. assuming the duration after event 'a is 200 ms, but after a few repetitions
you want it to be 400 ms to state 'b,

then the pseudo-rules would be

a    --- 200ms ---> x
aaaa --- 400ms ---> b

*/

#[derive(Clone)]
pub struct MarkovSequenceGenerator {
    // the name of this generator
    pub name: String,

    // the inner probabilistic finite automaton
    pub generator: pfa::Pfa<char>,

    // an event, in this context, is a sound or control event and the duration
    // to the next event ...
    pub event_mapping: BTreeMap<char, (Vec<SourceEvent>, Event)>,

    // override durations for certain states ...
    // optional because few generators use it ...
    pub override_durations: Option<BTreeMap<(Label<char>, char), Event>>,

    // map the internal chars to more human-readable labels ...
    pub label_mapping: Option<BTreeMap<char, String>>,

    // whether this generator has been modified
    pub modified: bool,

    // number of evaluations for each symbol, mostly important
    // for the lifemodeling algorithm
    pub symbol_ages: HashMap<char, u64>,

    // the duration to be used when no other duration
    // specified
    pub default_duration: u64,

    // the last transition
    pub last_transition: Option<pfa::PfaQueryResult<char>>,

    // the last emitted symbol
    pub last_symbol: Option<char>,
}

impl MarkovSequenceGenerator {
    pub fn transfer_state(&mut self, other: &MarkovSequenceGenerator) {
        if let Some(t) = &other.last_transition {
            self.last_transition = Some(t.clone());
            self.generator.transfer_state(&other.generator);
        }
    }

    pub fn current_events(
        &mut self,
        globals: &std::sync::Arc<GlobalVariables>,
    ) -> Vec<InterpretableEvent> {
        let mut interpretable_events = Vec::new();

        if let Some(last_symbol) = &self.last_symbol {
            // increment symbol age ...
            *self.symbol_ages.entry(*last_symbol).or_insert(0) += 1;

            // get static events ...
            if let Some((events, _)) = self.event_mapping.get_mut(last_symbol) {
                for e in events.iter_mut() {
                    interpretable_events.push(match e {
                        SourceEvent::Sound(e) => InterpretableEvent::Sound(e.get_static(globals)),
                        // this is quite an effort to copy the whole sync ctx all the time.
                        // i hope i can find a mor efficient method later ...
                        SourceEvent::Control(e) => InterpretableEvent::Control(e.clone()),
                    });
                }
            } else {
                println!("no events for sym {last_symbol}");
            }
        }

        // assume this is an end state ...
        if self.last_symbol.is_some() && self.last_transition.is_none() {
            println!("seems like this generator has reached it's end");
            self.last_symbol = None;
        }

        interpretable_events
    }

    pub fn current_transition(&mut self, globals: &std::sync::Arc<GlobalVariables>) -> StaticEvent {
        // advance pfa ...
        self.last_transition = self.generator.next_transition();

        let mut dur_ev = None;

        if let Some(trans) = &self.last_transition {
            self.last_symbol = Some(trans.last_symbol);
            println!(
                "LAST state {:#?} sym {:#?} CUR STATE {:#?} NEXT {:?}",
                trans.last_state, self.last_symbol, trans.current_state, trans.next_symbol
            );
            // if there is an override duration for a certain state/symbol combo, use that one ...
            if let Some(overrides) = self.override_durations.as_mut() {
                if let Some(ev) = overrides.get_mut(&(trans.last_state.clone(), trans.next_symbol))
                {
                    dur_ev = Some(ev.get_static(globals));
                    println!("FOUND OVERRIDE {dur_ev:#?}");
                }
            } else if let Some((_, dur)) = self.event_mapping.get_mut(&trans.last_symbol) {
                // otherwise, use the duration associated with the event ...
                dur_ev = Some(dur.get_static(globals))
            }
        }

        // if nothing could be found at all, use the default ...
        if let Some(ev) = dur_ev {
            ev
        } else {
            Event::transition(DynVal::with_value(self.default_duration as f32)).get_static(globals)
        }
    }

    /// this generator has reached a state that has no exits
    pub fn reached_end_state(&self) -> bool {
        self.last_symbol.is_none() && self.last_transition.is_none()
    }

    /// use this if the basic structure of the
    /// generator has been persistently modified (i.e. after growing,
    /// shrinking or adding rules)
    pub fn set_modified(&mut self) {
        self.modified = true;
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn clear_modified(&mut self) {
        self.modified = false;
    }
}
