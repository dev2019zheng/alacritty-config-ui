use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value, value};

use crate::config_types::{FontFace, Padding};

use super::ConfigGraph;

pub fn save(graph: &mut ConfigGraph) -> Result<()> {
    let imports = graph.current_imports();
    graph.merged.general.import = imports.clone();

    let live_reload = graph.merged.general.live_config_reload;
    let scrolling_history = graph.merged.scrolling.history;
    let scrolling_multiplier = graph.merged.scrolling.multiplier;
    let cursor = graph.merged.cursor.clone();
    let selection = graph.merged.selection.clone();
    let builtin_box_drawing = graph.merged.font.builtin_box_drawing;
    let merged = graph.merged.clone();
    let root_path = graph.root_path.clone();
    let base_path = graph.base_path.clone();
    let active_theme_path = graph.active_theme_path.clone();

    patch_root_document(graph.root_doc_mut(), &imports, live_reload);
    patch_base_document(
        graph.base_doc_mut(),
        scrolling_history,
        scrolling_multiplier,
        cursor,
        selection,
        builtin_box_drawing,
    );
    patch_theme_document(graph.active_theme_doc_mut(), &merged);

    let root_doc = graph.root_doc_mut().to_string();
    write_if_changed(&root_path, &root_doc)?;
    if let Some(base_path) = base_path {
        let base_doc = graph.base_doc_mut().to_string();
        write_if_changed(&base_path, &base_doc)?;
    }
    if let Some(theme_path) = active_theme_path {
        let theme_doc = graph.active_theme_doc_mut().to_string();
        write_if_changed(&theme_path, &theme_doc)?;
    }

    graph.dirty = false;
    Ok(())
}

fn patch_root_document(doc: &mut DocumentMut, imports: &[String], live_reload: bool) {
    let general = ensure_table(doc, "general");

    let mut array = Array::default();
    for import in imports {
        array.push(import.as_str());
    }
    general["import"] = Item::Value(Value::Array(array));
    general["live_config_reload"] = value(live_reload);
}

fn patch_base_document(
    doc: &mut DocumentMut,
    history: u32,
    multiplier: u8,
    cursor: crate::config_types::CursorConfig,
    selection: crate::config_types::SelectionConfig,
    builtin_box_drawing: bool,
) {
    let scrolling = ensure_table(doc, "scrolling");
    scrolling["history"] = value(i64::from(history));
    scrolling["multiplier"] = value(i64::from(multiplier));

    let font = ensure_table(doc, "font");
    font["builtin_box_drawing"] = value(builtin_box_drawing);

    let cursor_table = ensure_table(doc, "cursor");
    cursor_table["style"] = cursor_style_item(&cursor.style);
    cursor_table["vi_mode_style"] = vi_mode_style_item(&cursor.vi_mode_style);
    cursor_table["unfocused_hollow"] = value(cursor.unfocused_hollow);
    cursor_table["thickness"] = value(f64::from(cursor.thickness));

    let selection_table = ensure_table(doc, "selection");
    selection_table["semantic_escape_chars"] = value(selection.semantic_escape_chars);
    selection_table["save_to_clipboard"] = value(selection.save_to_clipboard);
}

fn patch_theme_document(doc: &mut DocumentMut, merged: &crate::config_types::EditorConfig) {
    let window = ensure_table(doc, "window");
    window["padding"] = padding_item(merged.window.padding);
    window["dynamic_padding"] = value(merged.window.dynamic_padding);
    window["decorations"] = value(format!("{:?}", merged.window.decorations));
    window["opacity"] = value(f64::from(merged.window.opacity));
    window["blur"] = value(merged.window.blur);
    window["option_as_alt"] = value(format!("{:?}", merged.window.option_as_alt));
    window["dynamic_title"] = value(merged.window.dynamic_title);

    let font = ensure_table(doc, "font");
    font["normal"] = font_face_item(&merged.font.normal);
    font["bold"] = font_face_item(&merged.font.bold);
    font["italic"] = font_face_item(&merged.font.italic);
    font["bold_italic"] = font_face_item(&merged.font.bold_italic);
    font["size"] = value(f64::from(merged.font.size));

    let colors = ensure_table(doc, "colors");
    colors["draw_bold_text_with_bright_colors"] =
        value(merged.colors.draw_bold_text_with_bright_colors);

    let primary = ensure_subtable(colors, "primary");
    primary["background"] = value(merged.colors.primary.background.to_hex());
    primary["foreground"] = value(merged.colors.primary.foreground.to_hex());
    if let Some(dim) = merged.colors.primary.dim_foreground {
        primary["dim_foreground"] = value(dim.to_hex());
    }
    if let Some(bright) = merged.colors.primary.bright_foreground {
        primary["bright_foreground"] = value(bright.to_hex());
    }

    let cursor = ensure_subtable(colors, "cursor");
    cursor["text"] = value(merged.colors.cursor.text.as_display_string());
    cursor["cursor"] = value(merged.colors.cursor.cursor.as_display_string());

    let selection = ensure_subtable(colors, "selection");
    selection["text"] = value(merged.colors.selection.text.as_display_string());
    selection["background"] = value(merged.colors.selection.background.as_display_string());

    patch_named_colors(ensure_subtable(colors, "normal"), &merged.colors.normal);
    patch_named_colors(ensure_subtable(colors, "bright"), &merged.colors.bright);
}

fn patch_named_colors(table: &mut Table, colors: &crate::config_types::NamedColors) {
    table["black"] = value(colors.black.to_hex());
    table["red"] = value(colors.red.to_hex());
    table["green"] = value(colors.green.to_hex());
    table["yellow"] = value(colors.yellow.to_hex());
    table["blue"] = value(colors.blue.to_hex());
    table["magenta"] = value(colors.magenta.to_hex());
    table["cyan"] = value(colors.cyan.to_hex());
    table["white"] = value(colors.white.to_hex());
}

fn ensure_table<'a>(doc: &'a mut DocumentMut, key: &str) -> &'a mut Table {
    let root = doc.as_table_mut();
    if !root.contains_key(key) || !root[key].is_table() {
        root[key] = Item::Table(Table::new());
    }
    root[key].as_table_mut().expect("table created above")
}

fn ensure_subtable<'a>(table: &'a mut Table, key: &str) -> &'a mut Table {
    if !table.contains_key(key) || !table[key].is_table() {
        table[key] = Item::Table(Table::new());
    }
    table[key].as_table_mut().expect("table created above")
}

fn padding_item(padding: Padding) -> Item {
    let mut table = InlineTable::new();
    table.insert("x", Value::from(i64::from(padding.x)));
    table.insert("y", Value::from(i64::from(padding.y)));
    Item::Value(Value::InlineTable(table))
}

fn font_face_item(face: &FontFace) -> Item {
    let mut table = InlineTable::new();
    table.insert("family", Value::from(face.family.clone()));
    table.insert("style", Value::from(face.style.clone()));
    Item::Value(Value::InlineTable(table))
}

fn cursor_style_item(style: &crate::config_types::CursorStyleConfig) -> Item {
    let mut table = InlineTable::new();
    table.insert("shape", Value::from(format!("{:?}", style.shape)));
    table.insert("blinking", Value::from(format!("{:?}", style.blinking)));
    Item::Value(Value::InlineTable(table))
}

fn vi_mode_style_item(style: &crate::config_types::ViModeStyle) -> Item {
    match style {
        crate::config_types::ViModeStyle::None => value("None"),
        crate::config_types::ViModeStyle::Style(style) => cursor_style_item(style),
    }
}

fn write_if_changed(path: &Path, contents: &str) -> Result<()> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }

    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory {}", parent.display()))?;

    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config.toml".to_owned());
    let temp_path = unique_temp_path(parent, &file_name);
    fs::write(&temp_path, contents)
        .with_context(|| format!("failed to write temp file {}", temp_path.display()))?;
    fs::rename(&temp_path, path)
        .with_context(|| format!("failed to replace {}", path.display()))?;
    Ok(())
}

fn unique_temp_path(parent: &Path, file_name: &str) -> PathBuf {
    for attempt in 0..1000 {
        let candidate = parent.join(format!(".{file_name}.tmp.{attempt}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!(".{file_name}.tmp.fallback"))
}
