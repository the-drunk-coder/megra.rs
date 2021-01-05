use std::collections::{HashMap, HashSet};

pub struct SampleInfo {
    pub key: HashSet<String>,
    pub bufnum: usize,
}

impl SampleInfo {
    pub fn is_superset(&self, key: &HashSet<String>) -> bool {
	self.key.is_superset(key)
    }
}

/// maps an event type (like "bd") to a mapping between keywords and buffer number ...
pub struct SampleSet {
    subsets: HashMap<String, Vec<SampleInfo>>
}

impl SampleSet {
    pub fn new() -> Self {
	SampleSet {
	    subsets: HashMap::new()
	}
    }

    pub fn insert(&mut self, set: String, keyword_set: HashSet<String>, bufnum: usize) {
	self.subsets.entry(set).or_insert(Vec::new()).push(SampleInfo{key: keyword_set, bufnum: bufnum});    
    }
    
    /// get a sample bufnum by 
    pub fn pos(&self, set: &String, pos: usize) -> Option<&SampleInfo> {
	if let Some(sample_subset) = self.subsets.get(set) {
	    if let Some(sample_info) = sample_subset.get(pos) {
		Some(sample_info)
	    } else {
		None
	    }
	} else {
	    None
	}
    }
}
