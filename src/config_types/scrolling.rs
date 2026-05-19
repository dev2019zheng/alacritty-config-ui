use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScrollingConfig {
    pub history: u32,
    pub multiplier: u8,
}

impl Default for ScrollingConfig {
    fn default() -> Self {
        Self {
            history: 10_000,
            multiplier: 3,
        }
    }
}
