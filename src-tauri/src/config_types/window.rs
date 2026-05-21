use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Padding {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub padding: Padding,
    pub dynamic_padding: bool,
    pub decorations: Decorations,
    pub opacity: f32,
    pub blur: bool,
    pub option_as_alt: OptionAsAlt,
    pub dynamic_title: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            padding: Padding::default(),
            dynamic_padding: false,
            decorations: Decorations::default(),
            opacity: 1.0,
            blur: false,
            option_as_alt: OptionAsAlt::default(),
            dynamic_title: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decorations {
    #[default]
    Full,
    Transparent,
    Buttonless,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionAsAlt {
    OnlyLeft,
    OnlyRight,
    Both,
    #[default]
    None,
}
