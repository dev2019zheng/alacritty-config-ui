use tauri::State;

use crate::{dto::ThemeEntryDto, state::AppState, theme_catalog::ThemeCatalog};

#[tauri::command]
pub fn list_themes(state: State<'_, AppState>) -> Result<Vec<ThemeEntryDto>, String> {
    let guard = state
        .graph
        .lock()
        .map_err(|_| "failed to lock application state".to_owned())?;
    let graph = guard
        .as_ref()
        .ok_or_else(|| "configuration has not been loaded yet".to_owned())?;
    let catalog = ThemeCatalog::load(&graph.config_dir()).map_err(|err| err.to_string())?;
    Ok(catalog.entries.iter().map(ThemeEntryDto::from).collect())
}
