use std::{path::PathBuf, sync::Mutex};

use crate::config_graph::ConfigGraph;

#[derive(Clone, Debug, Default)]
pub struct RuntimePaths {
    pub resource_dir: Option<PathBuf>,
}

pub struct AppState {
    pub graph: Mutex<Option<ConfigGraph>>,
    pub runtime_paths: RuntimePaths,
}

impl AppState {
    pub fn new(runtime_paths: RuntimePaths) -> Self {
        Self {
            graph: Mutex::default(),
            runtime_paths,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(RuntimePaths::default())
    }
}
