pub mod commands;
pub mod config_graph;
pub mod config_types;
pub mod dto;
pub mod state;
pub mod theme_catalog;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let resource_dir = app.path().resource_dir().ok();
            let _ = app.manage(state::AppState::new(state::RuntimePaths { resource_dir }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::catalog::list_themes,
            commands::config::apply_theme_entry,
            commands::config::load_config,
            commands::config::load_default_config,
            commands::config::reload_config,
            commands::config::save_config,
            commands::config::set_base_import,
            commands::config::set_theme_import,
            commands::config::update_config,
            commands::fonts::list_system_fonts,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}
