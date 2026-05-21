use serde::de::Deserializer;
use serde::{Deserialize, Serialize, Serializer};

use super::Rgb;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Colors {
    pub draw_bold_text_with_bright_colors: bool,
    pub primary: PrimaryColors,
    pub cursor: CursorColors,
    pub selection: SelectionColors,
    pub normal: NamedColors,
    pub bright: NamedColors,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            draw_bold_text_with_bright_colors: true,
            primary: PrimaryColors::default(),
            cursor: CursorColors::default(),
            selection: SelectionColors::default(),
            normal: NamedColors::default_normal(),
            bright: NamedColors::default_bright(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PrimaryColors {
    pub background: Rgb,
    pub foreground: Rgb,
    pub dim_foreground: Option<Rgb>,
    pub bright_foreground: Option<Rgb>,
}

impl Default for PrimaryColors {
    fn default() -> Self {
        Self {
            background: Rgb::new(0x0F, 0x11, 0x1A),
            foreground: Rgb::new(0xC5, 0xD1, 0xEB),
            dim_foreground: Some(Rgb::new(0x8A, 0x93, 0xA8)),
            bright_foreground: Some(Rgb::new(0xE6, 0xED, 0xF7)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CursorColors {
    pub text: CellColor,
    pub cursor: CellColor,
}

impl Default for CursorColors {
    fn default() -> Self {
        Self {
            text: CellColor::CellBackground,
            cursor: CellColor::Rgb(Rgb::new(0x89, 0xDD, 0xFF)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SelectionColors {
    pub text: CellColor,
    pub background: CellColor,
}

impl Default for SelectionColors {
    fn default() -> Self {
        Self {
            text: CellColor::CellForeground,
            background: CellColor::Rgb(Rgb::new(0x27, 0x32, 0x44)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NamedColors {
    pub black: Rgb,
    pub red: Rgb,
    pub green: Rgb,
    pub yellow: Rgb,
    pub blue: Rgb,
    pub magenta: Rgb,
    pub cyan: Rgb,
    pub white: Rgb,
}

impl NamedColors {
    pub fn default_normal() -> Self {
        Self {
            black: Rgb::new(0x1B, 0x1F, 0x2A),
            red: Rgb::new(0xF0, 0x71, 0x78),
            green: Rgb::new(0xA6, 0xDA, 0x95),
            yellow: Rgb::new(0xEB, 0xCB, 0x8B),
            blue: Rgb::new(0x7A, 0xA2, 0xF7),
            magenta: Rgb::new(0xC0, 0x99, 0xFF),
            cyan: Rgb::new(0x7F, 0xDB, 0xCA),
            white: Rgb::new(0xC5, 0xD1, 0xEB),
        }
    }

    pub fn default_bright() -> Self {
        Self {
            black: Rgb::new(0x3A, 0x41, 0x54),
            red: Rgb::new(0xFF, 0x8F, 0x97),
            green: Rgb::new(0xB8, 0xF0, 0xB0),
            yellow: Rgb::new(0xF5, 0xD7, 0xA1),
            blue: Rgb::new(0x8A, 0xB4, 0xFF),
            magenta: Rgb::new(0xD2, 0xA6, 0xFF),
            cyan: Rgb::new(0x94, 0xF0, 0xE0),
            white: Rgb::new(0xE6, 0xED, 0xF7),
        }
    }
}

impl Default for NamedColors {
    fn default() -> Self {
        Self::default_normal()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellColor {
    CellBackground,
    #[default]
    CellForeground,
    Rgb(Rgb),
}

impl CellColor {
    pub fn as_display_string(self) -> String {
        match self {
            Self::CellBackground => "CellBackground".to_owned(),
            Self::CellForeground => "CellForeground".to_owned(),
            Self::Rgb(rgb) => rgb.to_hex(),
        }
    }
}

impl Serialize for CellColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_display_string())
    }
}

impl<'de> Deserialize<'de> for CellColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "CellBackground" => Ok(Self::CellBackground),
            "CellForeground" => Ok(Self::CellForeground),
            _ => Ok(Self::Rgb(value.parse().map_err(serde::de::Error::custom)?)),
        }
    }
}
