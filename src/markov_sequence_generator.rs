use crate::event;
use vom_rs::pfa;
use std::collections::HashMap;

pub struct MarkovSequenceGenerator {
    pub name: String,
    pub generator: pfa::Pfa<char>,
    pub event_mapping: HashMap<char, event::Event>,
    pub duration_mapping: HashMap<(char, char), u64>,
    pub modified: bool,    
    pub symbol_ages: HashMap<char, u64>,
    pub default_duration: u64,
    pub last_transition: Option<pfa::PfaQueryResult<char>>,    
}
