mod load;
mod save;

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml_edit::DocumentMut;

use crate::config_types::{
    Colors, CursorConfig, EditorConfig, FontConfig, FontFace, GeneralConfig, ScrollingConfig,
    SelectionConfig, WindowConfig,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RootDocumentConfig {
    pub general: GeneralConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BaseDocumentConfig {
    pub scrolling: Option<ScrollingConfig>,
    pub cursor: Option<CursorConfig>,
    pub selection: Option<SelectionConfig>,
    pub font: Option<BaseFontConfig>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BaseFontConfig {
    pub builtin_box_drawing: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeDocumentConfig {
    pub window: Option<WindowConfig>,
    pub font: Option<ThemeFontConfig>,
    pub colors: Option<Colors>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeFontConfig {
    pub normal: FontFace,
    pub bold: FontFace,
    pub italic: FontFace,
    pub bold_italic: FontFace,
    pub size: f32,
}

impl Default for ThemeFontConfig {
    fn default() -> Self {
        let font = FontConfig::default();
        Self::from_font(&font)
    }
}

impl ThemeFontConfig {
    pub fn from_font(font: &FontConfig) -> Self {
        Self {
            normal: font.normal.clone(),
            bold: font.bold.clone(),
            italic: font.italic.clone(),
            bold_italic: font.bold_italic.clone(),
            size: font.size,
        }
    }

    pub fn apply_to_font(&self, font: &mut FontConfig) {
        font.normal = self.normal.clone();
        font.bold = self.bold.clone();
        font.italic = self.italic.clone();
        font.bold_italic = self.bold_italic.clone();
        font.size = self.size;
    }
}

pub struct ConfigGraph {
    pub root_path: PathBuf,
    pub base_path: Option<PathBuf>,
    pub active_theme_path: Option<PathBuf>,
    pub merged: EditorConfig,
    pub dirty: bool,
    root_doc: DocumentMut,
    base_doc: Option<DocumentMut>,
    active_theme_doc: Option<DocumentMut>,
}

impl ConfigGraph {
    pub fn load_default() -> Result<Self> {
        load::load_default()
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        load::load(path.as_ref())
    }

    pub fn reload(&mut self) -> Result<()> {
        let reloaded = Self::load(&self.root_path)?;
        *self = reloaded;
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        save::save(self)
    }

    pub fn root_dir(&self) -> &Path {
        self.root_path.parent().unwrap_or_else(|| Path::new("."))
    }

    pub fn config_dir(&self) -> PathBuf {
        self.root_dir().to_path_buf()
    }

    pub fn current_theme_fragment(&self) -> ThemeDocumentConfig {
        ThemeDocumentConfig {
            window: Some(self.merged.window.clone()),
            font: Some(ThemeFontConfig::from_font(&self.merged.font)),
            colors: Some(self.merged.colors.clone()),
        }
    }

    pub fn activate_theme_fragment(&mut self, path: PathBuf, fragment: ThemeDocumentConfig) {
        self.active_theme_path = Some(path);
        self.active_theme_doc = Some(DocumentMut::new());

        if let Some(window) = fragment.window {
            self.merged.window = window;
        }
        if let Some(font) = fragment.font {
            font.apply_to_font(&mut self.merged.font);
        }
        if let Some(colors) = fragment.colors {
            self.merged.colors = colors;
        }

        self.merged.general.import = self.current_imports();
        self.dirty = true;
    }

    pub fn current_imports(&self) -> Vec<String> {
        let mut imports = Vec::new();
        if let Some(base_path) = &self.base_path {
            imports.push(load::format_import_path(self.root_dir(), base_path));
        }
        if let Some(theme_path) = &self.active_theme_path {
            imports.push(load::format_import_path(self.root_dir(), theme_path));
        }
        imports
    }

    pub(crate) fn root_doc_mut(&mut self) -> &mut DocumentMut {
        &mut self.root_doc
    }

    pub(crate) fn base_doc_mut(&mut self) -> &mut DocumentMut {
        self.base_doc.get_or_insert_with(DocumentMut::new)
    }

    pub(crate) fn active_theme_doc_mut(&mut self) -> &mut DocumentMut {
        self.active_theme_doc.get_or_insert_with(DocumentMut::new)
    }
}

pub(crate) fn merge_documents(
    root: &RootDocumentConfig,
    base: &BaseDocumentConfig,
    theme: &ThemeDocumentConfig,
) -> EditorConfig {
    let mut merged = EditorConfig::default();
    merged.general.import = root.general.import.clone();
    merged.general.live_config_reload = root.general.live_config_reload;

    if let Some(scrolling) = &base.scrolling {
        merged.scrolling = scrolling.clone();
    }
    if let Some(cursor) = &base.cursor {
        merged.cursor = cursor.clone();
    }
    if let Some(selection) = &base.selection {
        merged.selection = selection.clone();
    }
    if let Some(font) = &base.font {
        merged.font.builtin_box_drawing = font.builtin_box_drawing;
    }

    if let Some(window) = &theme.window {
        merged.window = window.clone();
    }
    if let Some(font) = &theme.font {
        font.apply_to_font(&mut merged.font);
    }
    if let Some(colors) = &theme.colors {
        merged.colors = colors.clone();
    }

    merged
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::ConfigGraph;

    fn write_fixture(temp_dir: &TempDir) -> std::path::PathBuf {
        let root = temp_dir.path().join("alacritty.toml");
        let base = temp_dir.path().join("base.toml");
        let themes_dir = temp_dir.path().join("themes");
        let theme = themes_dir.join("night-calm.toml");

        fs::create_dir_all(&themes_dir).unwrap();
        fs::write(
            &root,
            r##"[general]
import = ["base.toml", "themes/night-calm.toml"]
live_config_reload = true
"##,
        )
        .unwrap();
        fs::write(
            &base,
            r##"[scrolling]
history = 20000
multiplier = 3

[font]
builtin_box_drawing = true

[cursor]
style = { shape = "Beam", blinking = "Off" }
vi_mode_style = "None"
unfocused_hollow = true
thickness = 0.16

[selection]
semantic_escape_chars = ",│`|:\"' ()[]{}<>\t"
save_to_clipboard = false
"##,
        )
        .unwrap();
        fs::write(
            &theme,
            r##"[window]
padding = { x = 12, y = 10 }
dynamic_padding = true
decorations = "Buttonless"
opacity = 0.92
blur = true
option_as_alt = "OnlyLeft"
dynamic_title = true

[font]
normal = { family = "JetBrainsMono Nerd Font Mono", style = "Regular" }
bold = { family = "JetBrainsMono Nerd Font Mono", style = "Bold" }
italic = { family = "JetBrainsMono Nerd Font Mono", style = "Italic" }
bold_italic = { family = "JetBrainsMono Nerd Font Mono", style = "Bold Italic" }
size = 13.5

[colors]
draw_bold_text_with_bright_colors = true

[colors.primary]
background = "#0F111A"
foreground = "#C5D1EB"
dim_foreground = "#8A93A8"
bright_foreground = "#E6EDF7"

[colors.cursor]
text = "CellBackground"
cursor = "#89DDFF"

[colors.selection]
text = "CellForeground"
background = "#273244"

[colors.normal]
black = "#1B1F2A"
red = "#F07178"
green = "#A6DA95"
yellow = "#EBCB8B"
blue = "#7AA2F7"
magenta = "#C099FF"
cyan = "#7FDBCA"
white = "#C5D1EB"

[colors.bright]
black = "#3A4154"
red = "#FF8F97"
green = "#B8F0B0"
yellow = "#F5D7A1"
blue = "#8AB4FF"
magenta = "#D2A6FF"
cyan = "#94F0E0"
white = "#E6EDF7"
"##,
        )
        .unwrap();

        root
    }

    #[test]
    fn load_merges_root_base_and_theme() {
        let temp_dir = TempDir::new().unwrap();
        let root = write_fixture(&temp_dir);
        let graph = ConfigGraph::load(&root).unwrap();

        assert_eq!(graph.merged.scrolling.history, 20_000);
        assert_eq!(graph.merged.cursor.thickness, 0.16);
        assert_eq!(graph.merged.window.opacity, 0.92);
        assert_eq!(graph.merged.font.size, 13.5);
        assert_eq!(graph.merged.general.import.len(), 2);
    }

    #[test]
    fn load_detects_theme_by_contents_even_outside_themes_dir() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("alacritty.toml");
        let base = temp_dir.path().join("base.toml");
        let visuals = temp_dir.path().join("visuals.toml");

        fs::write(
            &root,
            r##"[general]
import = ["base.toml", "visuals.toml"]
live_config_reload = true
"##,
        )
        .unwrap();
        fs::write(
            &base,
            r##"[scrolling]
history = 12345
multiplier = 3
"##,
        )
        .unwrap();
        fs::write(
            &visuals,
            r##"[window]
opacity = 0.77

[colors.primary]
background = "#101010"
foreground = "#F0F0F0"
"##,
        )
        .unwrap();

        let graph = ConfigGraph::load(&root).unwrap();

        assert_eq!(graph.base_path.as_deref(), Some(base.as_path()));
        assert_eq!(graph.active_theme_path.as_deref(), Some(visuals.as_path()));
        assert_eq!(graph.merged.scrolling.history, 12_345);
        assert_eq!(graph.merged.window.opacity, 0.77);
        assert_eq!(graph.merged.colors.primary.background.to_hex(), "#101010");
    }

    #[test]
    fn save_updates_only_owning_documents() {
        let temp_dir = TempDir::new().unwrap();
        let root = write_fixture(&temp_dir);
        let base_path = temp_dir.path().join("base.toml");
        let theme_path = temp_dir.path().join("themes/night-calm.toml");

        let original_root = fs::read_to_string(&root).unwrap();
        let mut graph = ConfigGraph::load(&root).unwrap();
        graph.merged.scrolling.history = 42_000;
        graph.merged.window.opacity = 0.85;
        graph.save().unwrap();

        let saved_root = fs::read_to_string(&root).unwrap();
        let saved_base = fs::read_to_string(&base_path).unwrap();
        let saved_theme = fs::read_to_string(&theme_path).unwrap();

        assert_eq!(saved_root, original_root);
        assert!(saved_base.contains("history = 42000"));
        assert!(saved_theme.contains("opacity = 0.85"));
    }

    #[test]
    fn activating_new_theme_updates_imports() {
        let temp_dir = TempDir::new().unwrap();
        let root = write_fixture(&temp_dir);
        let mut graph = ConfigGraph::load(&root).unwrap();
        let custom_path = temp_dir.path().join("themes/custom/test-theme.toml");

        let mut fragment = graph.current_theme_fragment();
        fragment.colors.as_mut().unwrap().primary.background = "#101010".parse().unwrap();
        graph.activate_theme_fragment(custom_path.clone(), fragment);
        graph.save().unwrap();

        let saved_root = fs::read_to_string(&root).unwrap();
        let saved_theme = fs::read_to_string(&custom_path).unwrap();
        assert!(saved_root.contains("themes/custom/test-theme.toml"));
        assert!(saved_theme.contains("background = \"#101010\""));
    }
}
