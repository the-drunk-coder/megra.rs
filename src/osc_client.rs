use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync;

use crate::{osc_sender::OscSender, visualizer_client::VisualizerClient};

#[derive(Clone)]
pub struct OscClient {
    pub vis: sync::Arc<RwLock<Option<VisualizerClient>>>,
    pub custom: sync::Arc<DashMap<String, OscSender>>,
}
impl OscClient {
    pub fn new() -> Self {
        OscClient {
            vis: sync::Arc::new(RwLock::new(None)),
            custom: sync::Arc::new(DashMap::new()),
        }
    }
}
