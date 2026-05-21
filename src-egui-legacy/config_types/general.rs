use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub import: Vec<String>,
    pub live_config_reload: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            import: Vec::new(),
            live_config_reload: true,
        }
    }
}
