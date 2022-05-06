use crate::parameter::Parameter;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

pub struct SampleInfo {
    pub key: HashSet<String>,
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
    wavematrices: HashMap<String, Vec<Vec<Parameter>>>,
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

    pub fn insert_wavematrix(&mut self, key: String, table: Vec<Vec<Parameter>>) {
        self.wavematrices.insert(key, table);
    }

    pub fn get_wavematrix(&self, key: &String) -> Option<&Vec<Vec<Parameter>>> {
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
            if let Some(sample_info) = subset.get(pos) {
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
}
