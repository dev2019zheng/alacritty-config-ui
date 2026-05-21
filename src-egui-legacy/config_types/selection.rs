use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SelectionConfig {
    pub semantic_escape_chars: String,
    pub save_to_clipboard: bool,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            semantic_escape_chars: ",│`|:\"' ()[]{}<>\t".to_owned(),
            save_to_clipboard: false,
        }
    }
}
