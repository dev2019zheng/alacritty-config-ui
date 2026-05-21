use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use toml_edit::DocumentMut;

use super::{
    BaseDocumentConfig, ConfigGraph, RootDocumentConfig, ThemeDocumentConfig, merge_documents,
};

pub fn load_default() -> Result<ConfigGraph> {
    let root = home_dir()?.join(".config/alacritty/alacritty.toml");
    load(&root)
}

pub fn load(root_path: &Path) -> Result<ConfigGraph> {
    let root_raw = fs::read_to_string(root_path)
        .with_context(|| format!("failed to read root config {}", root_path.display()))?;
    let root_doc = parse_document(&root_raw)?;
    let root_config: RootDocumentConfig = toml::from_str(&root_raw)
        .with_context(|| format!("failed to parse root config {}", root_path.display()))?;

    let imports = root_config.general.import.clone();
    let base_path = detect_base_path(root_path, &imports)?;
    let active_theme_path = detect_active_theme_path(root_path, &imports)?;

    let (base_doc, base_config) = if let Some(path) = &base_path {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read base config {}", path.display()))?;
        let doc = parse_document(&raw)?;
        let config = toml::from_str(&raw)
            .with_context(|| format!("failed to parse base config {}", path.display()))?;
        (Some(doc), config)
    } else {
        (None, BaseDocumentConfig::default())
    };

    let (active_theme_doc, active_theme_config) = if let Some(path) = &active_theme_path {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read theme config {}", path.display()))?;
        let doc = parse_document(&raw)?;
        let config = toml::from_str(&raw)
            .with_context(|| format!("failed to parse theme config {}", path.display()))?;
        (Some(doc), config)
    } else {
        (None, ThemeDocumentConfig::default())
    };

    let merged = merge_documents(&root_config, &base_config, &active_theme_config);

    Ok(ConfigGraph {
        root_path: root_path.to_path_buf(),
        base_path,
        active_theme_path,
        merged,
        dirty: false,
        root_doc,
        base_doc,
        active_theme_doc,
    })
}

pub fn home_dir() -> Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set; cannot resolve configuration directory")
}

pub fn resolve_import_path(root_path: &Path, import: &str) -> Result<PathBuf> {
    if let Some(stripped) = import.strip_prefix("~/") {
        return Ok(home_dir()?.join(stripped));
    }

    let path = PathBuf::from(import);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(root_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path))
    }
}

pub fn format_import_path(root_dir: &Path, path: &Path) -> String {
    if let Ok(relative) = path.strip_prefix(root_dir) {
        return relative.to_string_lossy().replace('\\', "/");
    }

    if let Ok(home) = home_dir()
        && let Ok(relative) = path.strip_prefix(&home)
    {
        return format!("~/{}", relative.to_string_lossy().replace('\\', "/"));
    }

    path.to_string_lossy().into_owned()
}

fn parse_document(raw: &str) -> Result<DocumentMut> {
    if raw.trim().is_empty() {
        Ok(DocumentMut::new())
    } else {
        raw.parse::<DocumentMut>()
            .context("failed to parse TOML document for editing")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportKind {
    Base,
    Theme,
    Unknown,
}

fn detect_base_path(root_path: &Path, imports: &[String]) -> Result<Option<PathBuf>> {
    for import in imports {
        let resolved = resolve_import_path(root_path, import)?;
        if matches!(classify_import_kind(&resolved), ImportKind::Base) {
            return Ok(Some(resolved));
        }
    }

    for import in imports {
        let resolved = resolve_import_path(root_path, import)?;
        if !matches!(fallback_import_kind_from_path(&resolved), ImportKind::Theme) {
            return Ok(Some(resolved));
        }
    }

    Ok(None)
}

fn detect_active_theme_path(root_path: &Path, imports: &[String]) -> Result<Option<PathBuf>> {
    let mut theme_candidate = None;
    for import in imports {
        let resolved = resolve_import_path(root_path, import)?;
        if matches!(classify_import_kind(&resolved), ImportKind::Theme) {
            theme_candidate = Some(resolved);
        }
    }

    if theme_candidate.is_some() {
        return Ok(theme_candidate);
    }

    for import in imports {
        let resolved = resolve_import_path(root_path, import)?;
        if matches!(fallback_import_kind_from_path(&resolved), ImportKind::Theme) {
            theme_candidate = Some(resolved);
        }
    }

    if theme_candidate.is_some() {
        return Ok(theme_candidate);
    }

    let fallback = imports
        .last()
        .map(|import| resolve_import_path(root_path, import))
        .transpose()?;
    Ok(fallback)
}

fn classify_import_kind(path: &Path) -> ImportKind {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return fallback_import_kind_from_path(path),
    };

    let base_config = toml::from_str::<BaseDocumentConfig>(&raw).unwrap_or_default();
    let theme_config = toml::from_str::<ThemeDocumentConfig>(&raw).unwrap_or_default();
    let has_base = base_config != BaseDocumentConfig::default();
    let has_theme = theme_config != ThemeDocumentConfig::default();

    match (has_base, has_theme) {
        (true, false) => ImportKind::Base,
        (false, true) => ImportKind::Theme,
        _ => fallback_import_kind_from_path(path),
    }
}

fn fallback_import_kind_from_path(path: &Path) -> ImportKind {
    if path.components().any(|component| {
        let part = component.as_os_str();
        part == "themes" || part == "custom"
    }) {
        ImportKind::Theme
    } else {
        ImportKind::Unknown
    }
}
