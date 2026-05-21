use font_kit::source::SystemSource;

#[tauri::command]
pub fn list_system_fonts() -> Vec<String> {
    let mut families = SystemSource::new().all_families().unwrap_or_default();
    families.sort();
    families.dedup();
    families
}
