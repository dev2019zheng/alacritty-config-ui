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
        if let Some(bundled_dir) = bundled_theme_dir() {
            collect_theme_entries(&bundled_dir, ThemeSource::BundledPreset, &mut entries)?;
        }
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

fn bundled_theme_dir() -> Option<PathBuf> {
    first_existing_dir(bundled_theme_dir_candidates())
}

fn bundled_theme_dir_candidates() -> Vec<PathBuf> {
    let current_exe = std::env::current_exe().ok();
    let current_dir = std::env::current_dir().ok();
    bundled_theme_dir_candidates_for(current_exe.as_deref(), current_dir.as_deref())
}

fn bundled_theme_dir_candidates_for(
    exe_path: Option<&Path>,
    current_dir: Option<&Path>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(exe_path) = exe_path
        && let Some(exe_dir) = exe_path.parent()
    {
        candidates.push(exe_dir.join("themes"));
        if let Some(contents_dir) = exe_dir.parent() {
            candidates.push(contents_dir.join("Resources/themes"));
        }
    }

    if let Some(current_dir) = current_dir {
        candidates.push(current_dir.join("vendor/alacritty-theme/themes"));
    }

    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/alacritty-theme/themes"));
    candidates
}

fn first_existing_dir(paths: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    let mut seen = Vec::new();

    for path in paths {
        if seen.iter().any(|existing| existing == &path) {
            continue;
        }
        if path.is_dir() {
            return Some(path);
        }
        seen.push(path);
    }

    None
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::{bundled_theme_dir_candidates_for, first_existing_dir};

    #[test]
    fn first_existing_dir_prefers_earliest_existing_candidate() {
        let temp_dir = TempDir::new().unwrap();
        let first = temp_dir.path().join("first");
        let second = temp_dir.path().join("second");
        fs::create_dir_all(&second).unwrap();
        fs::create_dir_all(&first).unwrap();

        let resolved = first_existing_dir(vec![second.clone(), first.clone()]).unwrap();

        assert_eq!(resolved, second);
    }

    #[test]
    fn first_existing_dir_skips_missing_candidates() {
        let temp_dir = TempDir::new().unwrap();
        let missing = temp_dir.path().join("missing");
        let existing = temp_dir.path().join("existing");
        fs::create_dir_all(&existing).unwrap();

        let resolved = first_existing_dir(vec![missing, existing.clone()]).unwrap();

        assert_eq!(resolved, existing);
    }

    #[test]
    fn bundled_theme_dir_candidates_include_app_resources() {
        let exe_path =
            Path::new("/Applications/Alacritty Config UI.app/Contents/MacOS/alacritty-config-ui");
        let current_dir = Path::new("/tmp/alacritty-config-ui");

        let candidates = bundled_theme_dir_candidates_for(Some(exe_path), Some(current_dir));

        assert_eq!(
            candidates[0],
            Path::new("/Applications/Alacritty Config UI.app/Contents/MacOS/themes")
        );
        assert_eq!(
            candidates[1],
            Path::new("/Applications/Alacritty Config UI.app/Contents/Resources/themes")
        );
        assert_eq!(
            candidates[2],
            current_dir.join("vendor/alacritty-theme/themes")
        );
        assert_eq!(
            candidates[3],
            Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/alacritty-theme/themes")
        );
    }
}
