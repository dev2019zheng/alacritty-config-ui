mod color;
mod cursor;
mod font;
mod general;
mod rgb;
mod scrolling;
mod selection;
mod window;

pub use color::{CellColor, Colors, NamedColors};
pub use cursor::{CursorBlinking, CursorConfig, CursorShape, CursorStyleConfig, ViModeStyle};
pub use font::{FontConfig, FontFace};
pub use general::GeneralConfig;
pub use rgb::Rgb;
pub use scrolling::ScrollingConfig;
pub use selection::SelectionConfig;
pub use window::{Decorations, OptionAsAlt, Padding, WindowConfig};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub general: GeneralConfig,
    pub scrolling: ScrollingConfig,
    pub cursor: CursorConfig,
    pub selection: SelectionConfig,
    pub font: FontConfig,
    pub window: WindowConfig,
    pub colors: Colors,
}
