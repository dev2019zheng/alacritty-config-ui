use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config_graph::ThemeDocumentConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeSource {
    BundledPreset,
    LocalPreset,
    LocalTheme,
    CustomTheme,
}

impl ThemeSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BundledPreset => "Bundled preset",
            Self::LocalPreset => "Local preset repo",
            Self::LocalTheme => "Local theme",
            Self::CustomTheme => "Custom theme",
        }
    }

    pub fn is_preset(&self) -> bool {
        matches!(self, Self::BundledPreset | Self::LocalPreset)
    }

    fn rank(&self) -> u8 {
        match self {
            Self::BundledPreset => 0,
            Self::LocalPreset => 1,
            Self::LocalTheme => 2,
            Self::CustomTheme => 3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThemeEntry {
    pub name: String,
    pub path: PathBuf,
    pub source: ThemeSource,
    pub fragment: ThemeDocumentConfig,
}

#[derive(Clone, Debug, Default)]
pub struct ThemeCatalog {
    pub entries: Vec<ThemeEntry>,
}

impl ThemeCatalog {
    pub fn load(config_root: &Path) -> Result<Self> {
        let mut entries = Vec::new();
        let bundled_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/alacritty-theme/themes");
        collect_theme_entries(&bundled_dir, ThemeSource::BundledPreset, &mut entries)?;
        collect_theme_entries(
            &config_root.join("themes/themes"),
            ThemeSource::LocalPreset,
            &mut entries,
        )?;
        collect_theme_entries(
            &config_root.join("themes/custom"),
            ThemeSource::CustomTheme,
            &mut entries,
        )?;
        collect_theme_entries(
            &config_root.join("themes"),
            ThemeSource::LocalTheme,
            &mut entries,
        )?;

        entries.sort_by(|left, right| {
            left.source
                .rank()
                .cmp(&right.source.rank())
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });

        Ok(Self { entries })
    }

    pub fn suggested_custom_path(&self, config_root: &Path, theme_name: &str) -> PathBuf {
        let custom_dir = config_root.join("themes/custom");
        let slug = slugify(theme_name);
        let mut candidate = custom_dir.join(format!("{slug}.toml"));
        let mut index = 2;
        while candidate.exists() || self.entries.iter().any(|entry| entry.path == candidate) {
            candidate = custom_dir.join(format!("{slug}-{index}.toml"));
            index += 1;
        }
        candidate
    }
}

fn collect_theme_entries(
    dir: &Path,
    source: ThemeSource,
    entries: &mut Vec<ThemeEntry>,
) -> Result<()> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        let fragment = match toml::from_str::<ThemeDocumentConfig>(&raw) {
            Ok(fragment) => fragment,
            Err(_) => continue,
        };
        entries.push(ThemeEntry {
            name: path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("theme")
                .replace('-', " "),
            path,
            source: source.clone(),
            fragment,
        });
    }

    Ok(())
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.trim_matches('-').to_owned()
}
