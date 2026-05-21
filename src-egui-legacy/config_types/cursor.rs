use serde::de::Deserializer;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CursorConfig {
    pub style: CursorStyleConfig,
    pub vi_mode_style: ViModeStyle,
    pub unfocused_hollow: bool,
    pub thickness: f32,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            style: CursorStyleConfig::default(),
            vi_mode_style: ViModeStyle::None,
            unfocused_hollow: true,
            thickness: 0.15,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorShape {
    #[default]
    Block,
    Underline,
    Beam,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorBlinking {
    Never,
    #[default]
    Off,
    On,
    Always,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorStyleConfig {
    pub shape: CursorShape,
    pub blinking: CursorBlinking,
}

impl Default for CursorStyleConfig {
    fn default() -> Self {
        Self {
            shape: CursorShape::Block,
            blinking: CursorBlinking::Off,
        }
    }
}

impl Serialize for CursorStyleConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CursorStyleConfig", 2)?;
        state.serialize_field("shape", &self.shape)?;
        state.serialize_field("blinking", &self.blinking)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CursorStyleConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Shape(CursorShape),
            Detailed {
                #[serde(default)]
                shape: CursorShape,
                #[serde(default)]
                blinking: CursorBlinking,
            },
        }

        match Repr::deserialize(deserializer)? {
            Repr::Shape(shape) => Ok(Self {
                shape,
                blinking: CursorBlinking::Off,
            }),
            Repr::Detailed { shape, blinking } => Ok(Self { shape, blinking }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViModeStyle {
    None,
    Style(CursorStyleConfig),
}

impl Serialize for ViModeStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::None => serializer.serialize_str("None"),
            Self::Style(style) => style.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ViModeStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum Disabled {
            None,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Disabled(Disabled),
            Style(CursorStyleConfig),
        }

        match Repr::deserialize(deserializer)? {
            Repr::Disabled(Disabled::None) => Ok(Self::None),
            Repr::Style(style) => Ok(Self::Style(style)),
        }
    }
}
