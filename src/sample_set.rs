use crate::parameter::DynVal;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

/// the search request for a sample
#[derive(Clone, Debug)]
pub enum SampleLookup {
    Key(String, HashSet<String>),    // lookup by key
    N(String, usize),                // lookup by position
    Random(String),                  // final random (different sample every time)
    FixedRandom(String, SampleInfo), // parse-time random (random sample will be chosen at parsing time)
}

/// the resolved sample info
#[derive(Debug, Clone)]
pub struct SampleInfo {
    pub key: HashSet<String>, // the key this was stored with
    pub bufnum: usize,
    pub duration: usize, // duration in ms ..
}

impl SampleInfo {
    /// superset match, not absolute match
    pub fn matches(&self, key: &HashSet<String>) -> bool {
        self.key.is_superset(key)
    }
}

/// maps an event type (like "bd") to a mapping between keywords and buffer number ...
/// also contains wavematrices (for now) ... it's a bit inconsistent given that
/// wavematrices are handled on the "megra-size", while buffers are stored at the
/// "ruffbox-size", but that gives me more possibilities to play with the wavematrices
/// on this size, so I suppose that's ok for the moment ...
pub struct SampleAndWavematrixSet {
    subsets: HashMap<String, Vec<SampleInfo>>,
    wavematrices: HashMap<String, Vec<Vec<DynVal>>>,
}

impl Default for SampleAndWavematrixSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleAndWavematrixSet {
    pub fn new() -> Self {
        SampleAndWavematrixSet {
            subsets: HashMap::new(),
            wavematrices: HashMap::new(),
        }
    }

    pub fn insert_wavematrix(&mut self, key: String, table: Vec<Vec<DynVal>>) {
        self.wavematrices.insert(key, table);
    }

    pub fn get_wavematrix(&self, key: &String) -> Option<&Vec<Vec<DynVal>>> {
        self.wavematrices.get(key)
    }

    pub fn insert(&mut self, set: String, keyword_set: HashSet<String>, bufnum: usize, dur: usize) {
        self.subsets
            .entry(set)
            .or_insert_with(Vec::new)
            .push(SampleInfo {
                key: keyword_set,
                bufnum,
                duration: dur,
            });
    }

    pub fn exists_not_empty(&self, set: &str) -> bool {
        self.subsets.contains_key(set) && !self.subsets.get(set).unwrap().is_empty()
    }

    /// get a sample by a set of keys ...
    pub fn keys(&self, set: &str, keywords: &HashSet<String>) -> Option<&SampleInfo> {
        if let Some(subset) = self.subsets.get(set) {
            let choice: Vec<&SampleInfo> = subset.iter().filter(|i| i.matches(keywords)).collect();
            if !choice.is_empty() {
                Some(choice.choose(&mut rand::thread_rng()).unwrap())
            } else {
                subset.get(0)
            }
        } else {
            None
        }
    }

    /// get a sample bufnum by
    pub fn pos(&self, set: &str, pos: usize) -> Option<&SampleInfo> {
        if let Some(subset) = self.subsets.get(set) {
            if let Some(sample_info) = subset.get(pos % subset.len()) {
                Some(sample_info)
            } else {
                subset.get(0)
            }
        } else {
            None
        }
    }

    pub fn random(&self, set: &str) -> Option<&SampleInfo> {
        self.subsets
            .get(set)
            .map(|subset| subset.choose(&mut rand::thread_rng()).unwrap())
    }

    // needs lifetimes for the temp return of the info ...
    pub fn resolve_lookup<'a>(&'a self, lookup: &'a SampleLookup) -> Option<&SampleInfo> {
        match lookup {
            SampleLookup::Key(fname, keywords) => self.keys(fname, keywords),
            SampleLookup::N(fname, pos) => self.pos(fname, *pos),
            SampleLookup::Random(fname) => self.random(fname),
            SampleLookup::FixedRandom(_, info) => Some(info),
        }
    }
}
