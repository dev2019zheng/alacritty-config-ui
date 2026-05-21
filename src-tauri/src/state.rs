use std::sync::Mutex;

use crate::config_graph::ConfigGraph;

#[derive(Default)]
pub struct AppState {
    pub graph: Mutex<Option<ConfigGraph>>,
}
