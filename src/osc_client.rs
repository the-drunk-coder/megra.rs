use std::collections::HashMap;
use std::sync;

use crate::{osc_sender::OscSender, visualizer_client::VisualizerClient};

pub struct OscClient {
    pub vis: Option<sync::Arc<VisualizerClient>>,
    pub custom: HashMap<String, sync::Arc<OscSender>>,
}

impl OscClient {
    pub fn new() -> Self {
        OscClient {
            vis: None,
            custom: HashMap::new(),
        }
    }
}

impl Clone for OscClient {
    fn clone(&self) -> OscClient {
        let mut cm = HashMap::new();
        for (k, v) in self.custom.iter() {
            cm.insert(k.to_string(), sync::Arc::clone(v));
        }
        OscClient {
            vis: if let Some(v) = self.vis.as_ref() {
                Some(sync::Arc::clone(v))
            } else {
                None
            },
            custom: cm,
        }
    }
}
