use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontFace {
    pub family: String,
    pub style: String,
}

impl FontFace {
    pub fn new(family: impl Into<String>, style: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            style: style.into(),
        }
    }
}

impl Default for FontFace {
    fn default() -> Self {
        Self::new(default_family(), "Regular")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    pub normal: FontFace,
    pub bold: FontFace,
    pub italic: FontFace,
    pub bold_italic: FontFace,
    pub size: f32,
    pub builtin_box_drawing: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        let family = default_family();
        Self {
            normal: FontFace::new(family.clone(), "Regular"),
            bold: FontFace::new(family.clone(), "Bold"),
            italic: FontFace::new(family.clone(), "Italic"),
            bold_italic: FontFace::new(family, "Bold Italic"),
            size: 11.25,
            builtin_box_drawing: true,
        }
    }
}

#[cfg(target_os = "macos")]
fn default_family() -> String {
    "Menlo".to_owned()
}

#[cfg(not(target_os = "macos"))]
fn default_family() -> String {
    "monospace".to_owned()
}
