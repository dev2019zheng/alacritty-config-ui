use serde::{Deserialize, Serialize};

use crate::{
    config_graph::{ConfigGraph, ThemeDocumentConfig},
    config_types::EditorConfig,
    theme_catalog::{ThemeEntry, ThemeSource},
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigStateDto {
    pub merged: EditorConfig,
    pub root_path: String,
    pub base_path: Option<String>,
    pub active_theme_path: Option<String>,
    pub dirty: bool,
}

impl ConfigStateDto {
    pub fn from_graph(graph: &ConfigGraph) -> Self {
        Self {
            merged: graph.merged.clone(),
            root_path: graph.root_path.to_string_lossy().into_owned(),
            base_path: graph
                .base_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            active_theme_path: graph
                .active_theme_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            dirty: graph.dirty,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeSourceDto {
    BundledPreset,
    LocalPreset,
    LocalTheme,
    CustomTheme,
}

impl ThemeSourceDto {
    pub fn is_preset(&self) -> bool {
        matches!(self, Self::BundledPreset | Self::LocalPreset)
    }
}

impl From<&ThemeSource> for ThemeSourceDto {
    fn from(value: &ThemeSource) -> Self {
        match value {
            ThemeSource::BundledPreset => Self::BundledPreset,
            ThemeSource::LocalPreset => Self::LocalPreset,
            ThemeSource::LocalTheme => Self::LocalTheme,
            ThemeSource::CustomTheme => Self::CustomTheme,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeEntryDto {
    pub name: String,
    pub path: String,
    pub source: ThemeSourceDto,
    pub fragment: ThemeDocumentConfig,
}

impl From<&ThemeEntry> for ThemeEntryDto {
    fn from(value: &ThemeEntry) -> Self {
        Self {
            name: value.name.clone(),
            path: value.path.to_string_lossy().into_owned(),
            source: ThemeSourceDto::from(&value.source),
            fragment: value.fragment.clone(),
        }
    }
}
