use dashmap::DashMap;
use std::sync;

use crate::{osc_sender::OscSender, visualizer_client::VisualizerClient};

pub struct OscClient {
    pub vis: Option<sync::Arc<VisualizerClient>>,
    pub custom: sync::Arc<DashMap<String, OscSender>>,
}

impl OscClient {
    pub fn new() -> Self {
        OscClient {
            vis: None,
            custom: sync::Arc::new(DashMap::new()),
        }
    }
}

impl Clone for OscClient {
    fn clone(&self) -> OscClient {
        OscClient {
            vis: if let Some(v) = self.vis.as_ref() {
                Some(sync::Arc::clone(v))
            } else {
                None
            },
            custom: sync::Arc::clone(&self.custom),
        }
    }
}
