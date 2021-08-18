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

        // try to get a transition if there wasn't one
        // that'd mean it's probably the initial one, or there's something wrong ...
        if self.last_transition.is_none() {
            self.last_transition = self.generator.next_transition();
            //println!("no last_trans, fetch");
        }
        //else if let Some(trans) = self.last_transition.clone() {
        //println!("trans present, cur sym: {}", &trans.last_symbol);
        //if !self.event_mapping.contains_key(&trans.last_symbol) {
        //println!("cur sym invalid, fetch next: {}", &trans.last_symbol);
        // if transition was invalidated by shrink, for example ...
        //    self.last_transition = self.generator.next_transition();
        //}
        //}

        if let Some(trans) = &self.last_transition {
            //println!("cur sym EFFECTIVE: {}", &trans.last_symbol);
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
            }
            //else {
            //	println!("no events for sym {}", trans.last_symbol);
            //}
        }

        interpretable_events
    }

    pub fn current_transition(&mut self) -> StaticEvent {
        let mut transition = None;

        if let Some(trans) = &self.last_transition {
            if let Some(dur) = self
                .duration_mapping
                .get_mut(&(trans.last_symbol, trans.next_symbol))
            {
                transition = Some(dur.get_static());
            } else {
                //println!("no dur");

                let mut t = Event::with_name("transition".to_string()).get_static();
                t.params
                    .insert(SynthParameter::Duration, self.default_duration as f32);
                transition = Some(t);
            }
        }
        //else {
        //  println!("no old trans");
        //}

        // advance pfa ...
        self.last_transition = self.generator.next_transition();

        if let Some(t) = transition {
            t
        } else {
            //println!("no new trans");
            let mut t = Event::with_name("transition".to_string()).get_static();
            t.params
                .insert(SynthParameter::Duration, self.default_duration as f32);
            t
        }
    }
}
