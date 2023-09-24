use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::{self, atomic::AtomicBool};

use crate::{osc_sender::OscSender, visualizer_client::VisualizerClient};

#[derive(Clone)]
pub struct OscClient {
    // probably this extra flag isn't all that necessary, but maybe
    // it saves a bit of time if we just check the flag instead of
    // trying to acquire the read lock on the client all the time?
    // Or at least that's what I'm thinking ...
    pub vis_connected: sync::Arc<AtomicBool>,
    pub vis: sync::Arc<RwLock<Option<VisualizerClient>>>,
    pub custom: sync::Arc<DashMap<String, OscSender>>,
}
impl OscClient {
    pub fn new() -> Self {
        OscClient {
            vis_connected: sync::Arc::new(AtomicBool::new(false)),
            vis: sync::Arc::new(RwLock::new(None)),
            custom: sync::Arc::new(DashMap::new()),
        }
    }
}
