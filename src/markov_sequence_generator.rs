use crate::event;
use vom_rs::pfa;
use std::collections::HashMap;

pub struct MarkovSequenceGenerator {
    name: String,
    pub generator: pfa::Pfa<char>,
    event_mapping: HashMap<char, event::Event>,
    duration_mapping: HashMap<(char, char), u64>,
    modified: bool,    
    symbol_ages: HashMap<char, u64>,
    default_duration: u64,
    last_transition: pfa::PfaQueryResult<char>,    
}
