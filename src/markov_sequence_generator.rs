use crate::event::{Event, InterpretableEvent, SourceEvent, StaticEvent};
use crate::GlobalVariables;
use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
use std::collections::{BTreeMap, HashMap};
use vom_rs::pfa;

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

#[derive(Clone)]
pub struct MarkovSequenceGenerator {
    pub name: String,
    pub generator: pfa::Pfa<char>,
    pub event_mapping: BTreeMap<char, Vec<SourceEvent>>,
    // map the internal chars to more human-readable labels ...
    pub label_mapping: Option<BTreeMap<char, String>>,
    pub duration_mapping: HashMap<(char, char), Event>,
    pub modified: bool,
    pub symbol_ages: HashMap<char, u64>,
    pub default_duration: u64,
    pub last_transition: Option<pfa::PfaQueryResult<char>>,
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
            //println!("cur sym EFFECTIVE: {}", last_symbol);
            // increment symbol age ...
            *self.symbol_ages.entry(*last_symbol).or_insert(0) += 1;
            // get static events ...
            if let Some(events) = self.event_mapping.get_mut(last_symbol) {
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
        // keep in case there's no next transition because
        // the generator has reached it's end ...
        let tmp_next = if self.last_transition.is_some() {
            Some(self.last_transition.as_ref().unwrap().next_symbol)
        } else {
            None
        };
        // advance pfa ...
        self.last_transition = self.generator.next_transition();
        //println!("cur trans");
        if let Some(trans) = &self.last_transition {
            self.last_symbol = Some(trans.last_symbol);
            //println!("last sym: {}", trans.last_symbol);
            if let Some(dur) = self
                .duration_mapping
                .get_mut(&(trans.last_symbol, trans.next_symbol))
            {
                dur.get_static(globals)
            } else {
                let mut t = Event::with_name("transition".to_string()).get_static(globals);
                t.params.insert(
                    SynthParameterLabel::Duration.into(),
                    SynthParameterValue::ScalarF32(self.default_duration as f32),
                );
                t
            }
        } else {
            self.last_symbol = tmp_next;
            // these double else blocks doing the same thing sometimes make rust ugly
            let mut t = Event::with_name("transition".to_string()).get_static(globals);
            t.params.insert(
                SynthParameterLabel::Duration.into(),
                SynthParameterValue::ScalarF32(self.default_duration as f32),
            );
            t
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
