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
pub struct SampleSet {
    subsets: HashMap<String, Vec<SampleInfo>>,
}

impl SampleSet {
    pub fn new() -> Self {
        SampleSet {
            subsets: HashMap::new(),
        }
    }

    pub fn insert(&mut self, set: String, keyword_set: HashSet<String>, bufnum: usize, dur: usize) {
        self.subsets
            .entry(set)
            .or_insert(Vec::new())
            .push(SampleInfo {
                key: keyword_set,
                bufnum: bufnum,
                duration: dur,
            });
    }

    pub fn exists_not_empty(&self, set: &String) -> bool {
        self.subsets.contains_key(set) && !self.subsets.get(set).unwrap().is_empty()
    }

    pub fn keys(&self, set: &String, keywords: &HashSet<String>) -> Option<&SampleInfo> {
        if let Some(subset) = self.subsets.get(set) {
            let choice: Vec<&SampleInfo> = subset.iter().filter(|i| i.matches(keywords)).collect();
            if !choice.is_empty() {
                Some(choice.choose(&mut rand::thread_rng()).unwrap())
            } else if let Some(sample_info) = subset.get(0) {
                // fallback
                Some(sample_info)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// get a sample bufnum by
    pub fn pos(&self, set: &String, pos: usize) -> Option<&SampleInfo> {
        if let Some(subset) = self.subsets.get(set) {
            if let Some(sample_info) = subset.get(pos) {
                Some(sample_info)
            } else if let Some(sample_info) = subset.get(0) {
                // fallback
                Some(sample_info)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn random(&self, set: &String) -> Option<&SampleInfo> {
        if let Some(subset) = self.subsets.get(set) {
            Some(subset.choose(&mut rand::thread_rng()).unwrap())
        } else {
            None
        }
    }
}
