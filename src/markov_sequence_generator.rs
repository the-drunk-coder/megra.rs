use crate::event::{Event, InterpretableEvent, SourceEvent, StaticEvent};
use ruffbox_synth::ruffbox::synth::SynthParameter;
use std::collections::HashMap;
use vom_rs::pfa;

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
    pub event_mapping: HashMap<char, Vec<SourceEvent>>,
    pub duration_mapping: HashMap<(char, char), Event>,
    pub modified: bool,
    pub symbol_ages: HashMap<char, u64>,
    pub default_duration: u64,
    pub last_transition: Option<pfa::PfaQueryResult<char>>,
}

impl MarkovSequenceGenerator {
    pub fn transfer_state(&mut self, other: &MarkovSequenceGenerator) {
        if let Some(t) = &other.last_transition {
            self.last_transition = Some(t.clone());
            self.generator.transfer_state(&other.generator);
        }
    }

    pub fn current_events(&mut self) -> Vec<InterpretableEvent> {
        let mut interpretable_events = Vec::new();

        if let Some(trans) = &self.last_transition {
            // println!("cur sym EFFECTIVE: {}", &trans.last_symbol);
            // increment symbol age ...
            *self.symbol_ages.entry(trans.last_symbol).or_insert(0) += 1;
            // get static events ...
            if let Some(events) = self.event_mapping.get_mut(&trans.last_symbol) {
                for e in events.iter_mut() {
                    interpretable_events.push(match e {
                        SourceEvent::Sound(e) => InterpretableEvent::Sound(e.get_static()),
                        // this is quite an effort to copy the whole sync ctx all the time.
                        // i hope i can find a mor efficient method later ...
                        SourceEvent::Control(e) => InterpretableEvent::Control(e.clone()),
                    });
                }
            } else {
                println!("no events for sym {}", trans.last_symbol);
            }
        }

        interpretable_events
    }

    pub fn current_transition(&mut self) -> StaticEvent {
        // advance pfa ...
        self.last_transition = self.generator.next_transition();
        //println!("cur trans");
        if let Some(trans) = &self.last_transition {
            //println!("last sym: {}", trans.last_symbol);
            if let Some(dur) = self
                .duration_mapping
                .get_mut(&(trans.last_symbol, trans.next_symbol))
            {
                dur.get_static()
            } else {
                let mut t = Event::with_name("transition".to_string()).get_static();
                t.params
                    .insert(SynthParameter::Duration, self.default_duration as f32);
                t
            }
        } else {
            // these double else blocks doing the same thing sometimes make rust ugly
            let mut t = Event::with_name("transition".to_string()).get_static();
            t.params
                .insert(SynthParameter::Duration, self.default_duration as f32);
            t
        }
    }
}
