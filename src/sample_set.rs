use crate::parameter::DynVal;
use dashmap::DashMap;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::sync::Arc;

/// the search request for a sample
#[derive(Clone, Debug)]
pub enum SampleLookup {
    Key(String, HashSet<String>),        // lookup by key
    N(String, usize),                    // lookup by position
    Random(String),                      // final random (different sample every time)
    FixedRandom(String, (usize, usize)), // parse-time random (random sample will be chosen at parsing time)
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
/// wavematrices are handled on the "megra-side", while buffers are stored at the
/// "ruffbox-side", but that gives me more possibilities to play with the wavematrices
/// on this size, so I suppose that's ok for the moment ...
/// also, you'll see many many clones, but keep in mind it's basically only the arcs that's
/// cloned
#[derive(Clone)]
pub struct SampleAndWavematrixSet {
    subsets: Arc<DashMap<String, Vec<SampleInfo>>>,
    wavematrices: Arc<DashMap<String, Vec<Vec<DynVal>>>>,
}

impl Default for SampleAndWavematrixSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleAndWavematrixSet {
    pub fn new() -> Self {
        SampleAndWavematrixSet {
            subsets: Arc::new(DashMap::new()),
            wavematrices: Arc::new(DashMap::new()),
        }
    }

    pub fn insert_wavematrix(&mut self, key: String, table: Vec<Vec<DynVal>>) {
        self.wavematrices.insert(key, table);
    }

    pub fn get_wavematrix(&self, key: &String) -> Option<Vec<Vec<DynVal>>> {
        self.wavematrices.get(key).map(|wm| wm.clone())
    }

    pub fn insert(&mut self, set: String, keyword_set: HashSet<String>, bufnum: usize, dur: usize) {
        self.subsets.entry(set).or_default().push(SampleInfo {
            key: keyword_set,
            bufnum,
            duration: dur,
        });
    }

    pub fn exists_not_empty(&self, set: &str) -> bool {
        self.subsets.contains_key(set) && !self.subsets.get(set).unwrap().is_empty()
    }

    /// get a sample by a set of keys ...
    pub fn keys(&self, set: &str, keywords: &HashSet<String>) -> Option<(usize, usize)> {
        if let Some(subset) = self.subsets.get(set) {
            let choice: Vec<&SampleInfo> = subset.iter().filter(|i| i.matches(keywords)).collect();
            if !choice.is_empty() {
                let res = choice.choose(&mut rand::thread_rng()).unwrap();
                Some((res.bufnum, res.duration))
            } else {
                // there's always one ...
                let res = subset.get(0).unwrap();
                Some((res.bufnum, res.duration))
            }
        } else {
            None
        }
    }

    /// get a sample bufnum by
    pub fn pos(&self, set: &str, pos: usize) -> Option<(usize, usize)> {
        if let Some(subset) = self.subsets.get(set) {
            if let Some(sample_info) = subset.get(pos % subset.len()) {
                Some((sample_info.bufnum, sample_info.duration))
            } else {
                let res = subset.get(0).unwrap();
                Some((res.bufnum, res.duration))
            }
        } else {
            None
        }
    }

    pub fn random(&self, set: &str) -> Option<(usize, usize)> {
        self.subsets.get(set).map(|subset| {
            let res = subset.choose(&mut rand::thread_rng()).unwrap();
            (res.bufnum, res.duration)
        })
    }

    // needs lifetimes for the temp return of the info ...
    pub fn resolve_lookup<'a>(&'a self, lookup: &'a SampleLookup) -> Option<(usize, usize)> {
        match lookup {
            SampleLookup::Key(fname, keywords) => self.keys(fname, keywords),
            SampleLookup::N(fname, pos) => self.pos(fname, *pos),
            SampleLookup::Random(fname) => self.random(fname),
            SampleLookup::FixedRandom(_, info) => Some(*info),
        }
    }
}
