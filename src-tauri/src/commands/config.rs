use std::{fs, path::PathBuf, sync::MutexGuard};

use tauri::State;

use crate::{
    config_graph::{ConfigGraph, ThemeDocumentConfig},
    config_types::EditorConfig,
    dto::{ConfigStateDto, ThemeEntryDto},
    state::AppState,
    theme_catalog::ThemeCatalog,
};

fn graph_guard<'a>(
    state: &'a State<'_, AppState>,
) -> Result<MutexGuard<'a, Option<ConfigGraph>>, String> {
    state
        .graph
        .lock()
        .map_err(|_| "failed to lock application state".to_owned())
}

fn current_graph_mut<'a>(
    state: &'a State<'_, AppState>,
) -> Result<MutexGuard<'a, Option<ConfigGraph>>, String> {
    graph_guard(state)
}

fn require_graph_mut<'a>(
    state: &'a State<'_, AppState>,
) -> Result<MutexGuard<'a, Option<ConfigGraph>>, String> {
    let guard = current_graph_mut(state)?;
    if guard.is_none() {
        return Err("configuration has not been loaded yet".to_owned());
    }
    Ok(guard)
}

#[tauri::command]
pub fn load_default_config(state: State<'_, AppState>) -> Result<ConfigStateDto, String> {
    let graph = ConfigGraph::load_default().map_err(|err| err.to_string())?;
    let dto = ConfigStateDto::from_graph(&graph);
    *graph_guard(&state)? = Some(graph);
    Ok(dto)
}

#[tauri::command]
pub fn load_config(path: String, state: State<'_, AppState>) -> Result<ConfigStateDto, String> {
    let graph = ConfigGraph::load(path).map_err(|err| err.to_string())?;
    let dto = ConfigStateDto::from_graph(&graph);
    *graph_guard(&state)? = Some(graph);
    Ok(dto)
}

#[tauri::command]
pub fn reload_config(state: State<'_, AppState>) -> Result<ConfigStateDto, String> {
    let mut guard = require_graph_mut(&state)?;
    let graph = guard.as_mut().expect("checked by require_graph_mut");
    graph.reload().map_err(|err| err.to_string())?;
    Ok(ConfigStateDto::from_graph(graph))
}

#[tauri::command]
pub fn save_config(state: State<'_, AppState>) -> Result<ConfigStateDto, String> {
    let mut guard = require_graph_mut(&state)?;
    let graph = guard.as_mut().expect("checked by require_graph_mut");
    graph.save().map_err(|err| err.to_string())?;
    Ok(ConfigStateDto::from_graph(graph))
}

#[tauri::command]
pub fn update_config(
    mut merged: EditorConfig,
    state: State<'_, AppState>,
) -> Result<ConfigStateDto, String> {
    let mut guard = require_graph_mut(&state)?;
    let graph = guard.as_mut().expect("checked by require_graph_mut");
    merged.general.import = graph.current_imports();
    graph.merged = merged;
    graph.dirty = true;
    Ok(ConfigStateDto::from_graph(graph))
}

#[tauri::command]
pub fn set_base_import(path: String, state: State<'_, AppState>) -> Result<ConfigStateDto, String> {
    let mut guard = require_graph_mut(&state)?;
    let graph = guard.as_mut().expect("checked by require_graph_mut");
    graph.base_path = Some(PathBuf::from(path));
    graph.merged.general.import = graph.current_imports();
    graph.dirty = true;
    Ok(ConfigStateDto::from_graph(graph))
}

#[tauri::command]
pub fn set_theme_import(
    path: String,
    state: State<'_, AppState>,
) -> Result<ConfigStateDto, String> {
    let raw = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let fragment = toml::from_str::<ThemeDocumentConfig>(&raw).map_err(|err| err.to_string())?;

    let mut guard = require_graph_mut(&state)?;
    let graph = guard.as_mut().expect("checked by require_graph_mut");
    graph.activate_theme_fragment(PathBuf::from(path), fragment);
    Ok(ConfigStateDto::from_graph(graph))
}

#[tauri::command]
pub fn apply_theme_entry(
    entry: ThemeEntryDto,
    state: State<'_, AppState>,
) -> Result<ConfigStateDto, String> {
    let ThemeEntryDto {
        name,
        path,
        source,
        fragment: entry_fragment,
    } = entry;

    let mut guard = require_graph_mut(&state)?;
    let graph = guard.as_mut().expect("checked by require_graph_mut");

    let mut fragment = graph.current_theme_fragment();
    if let Some(window) = entry_fragment.window {
        fragment.window = Some(window);
    }
    if let Some(font) = entry_fragment.font {
        fragment.font = Some(font);
    }
    if let Some(colors) = entry_fragment.colors {
        fragment.colors = Some(colors);
    }

    let target_path = if source.is_preset() {
        let catalog = ThemeCatalog::load(
            &graph.config_dir(),
            state.runtime_paths.resource_dir.as_deref(),
        )
        .map_err(|err| err.to_string())?;
        catalog.suggested_custom_path(&graph.config_dir(), &name)
    } else {
        PathBuf::from(path)
    };

    graph.activate_theme_fragment(target_path, fragment);
    graph.save().map_err(|err| err.to_string())?;
    Ok(ConfigStateDto::from_graph(graph))
}
