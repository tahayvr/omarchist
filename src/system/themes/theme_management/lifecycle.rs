use crate::error::{Error, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use chrono::Utc;

use crate::types::themes::EditingTheme;

use super::colors::{read_colors_toml, update_colors_toml};
use super::paths::get_custom_themes_dir;
use crate::assets::extract_default_dir;
use crate::system::fs::write_atomic;
use crate::system::themes::theme_file_ops::{is_system_theme, omarchist_theme_dir};

pub fn generate_unique_theme_name() -> String {
    let themes_dir = match get_custom_themes_dir() {
        Some(dir) => dir,
        None => return format!("custom-theme-{}", Utc::now().timestamp()),
    };

    let base_name = "custom-theme";
    let mut counter = 1;

    loop {
        let name = format!("{}-{}", base_name, counter);
        let theme_path = themes_dir.join(&name);

        if !theme_path.exists() {
            return name;
        }

        counter += 1;

        // Safety check to prevent infinite loops
        if counter > 1000 {
            return format!("custom-theme-{}", Utc::now().timestamp());
        }
    }
}

// Turns arbitrary text (typically an image file stem) into a theme folder
// name Omarchy accepts: lowercase ASCII letters, digits and single dashes.
// `omarchy-theme-set` itself only lowercases and swaps spaces, and rejects
// names starting with a dot or containing a slash.
pub fn slugify_theme_name(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut pending_dash = false;
    for ch in input.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    if slug.is_empty() {
        "custom-theme".to_string()
    } else {
        slug
    }
}

// `base`, or `base-2`, `base-3`, ... — the first that isn't already a theme.
pub fn unique_theme_name(base: &str) -> String {
    let Some(themes_dir) = get_custom_themes_dir() else {
        return base.to_string();
    };
    if !name_is_taken(&themes_dir, base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|name| !name_is_taken(&themes_dir, name))
        .unwrap_or_else(|| format!("{base}-{}", Utc::now().timestamp()))
}

/// A name is taken by a folder in the user's themes dir or by one of
/// Omarchy's own themes: `omarchy-theme-set` copies the official theme first
/// and overlays the user's folder of the same name, so the two would merge.
fn name_is_taken(themes_dir: &Path, name: &str) -> bool {
    themes_dir.join(name).exists() || is_system_theme(name)
}

pub fn create_theme_from_defaults(theme_name: &str) -> Result<String> {
    let themes_dir = get_custom_themes_dir().ok_or(Error::UnknownDirectory("custom themes"))?;

    let new_theme_dir = themes_dir.join(theme_name);

    if name_is_taken(&themes_dir, theme_name) {
        return Err(Error::ThemeExists(theme_name.to_string()));
    }

    extract_default_dir("theme", &new_theme_dir)?;
    update_theme_metadata(&new_theme_dir, theme_name)?;

    Ok(theme_name.to_string())
}

fn update_theme_metadata(theme_dir: &Path, theme_name: &str) -> Result<()> {
    let json_path = theme_dir.join("omarchist.json");

    if !json_path.exists() {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();

    let content = fs::read_to_string(&json_path)
        .map_err(|e| Error::io("Failed to read omarchist.json", e))?;

    let updated_content = content
        .replace("{{THEME_NAME}}", theme_name)
        .replace("{{CREATED_AT}}", &now)
        .replace("{{MODIFIED_AT}}", &now)
        .replace("{{AUTHOR}}", "");

    write_atomic(&json_path, updated_content, "omarchist.json")?;

    Ok(())
}

pub fn load_theme_for_editing(theme_name: &str) -> Result<EditingTheme> {
    let themes_dir = get_custom_themes_dir().ok_or(Error::UnknownDirectory("custom themes"))?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(Error::ThemeNotFound(theme_name.to_string()));
    }

    let json_path = theme_dir.join("omarchist.json");
    if !json_path.is_file() {
        return Err(Error::NotOmarchistTheme(theme_name.to_string()));
    }
    let content = fs::read_to_string(&json_path)
        .map_err(|e| Error::io("Failed to read omarchist.json", e))?;
    let mut editing_theme: EditingTheme = serde_json::from_str(&content)
        .map_err(|e| Error::json("Failed to parse omarchist.json", e))?;

    // colors.toml is what Omarchy reads and what users may hand-edit, so it
    // is the palette's source of truth; the manifest's copy only fills gaps.
    if let Ok(toml) = fs::read_to_string(theme_dir.join("colors.toml")) {
        editing_theme.colors = read_colors_toml(&toml, &editing_theme.colors);
    }

    // `mode` in colors.toml is authoritative; the `light.mode` marker file is
    // only honored for themes written by pre-Quattro versions of Omarchist.
    editing_theme.is_light_theme =
        editing_theme.colors.mode == "light" || theme_dir.join("light.mode").exists();

    Ok(editing_theme)
}

pub fn save_theme_data(theme_name: &str, theme_data: &EditingTheme) -> Result<()> {
    let theme_dir = omarchist_theme_dir(theme_name)?;

    let mut updated_theme = theme_data.clone();
    updated_theme.modified_at = Utc::now().to_rfc3339();
    // `is_light_theme` is runtime-only; `colors.mode` is what persists (in
    // both the manifest and colors.toml) and what `load_theme_for_editing`
    // reads back, so keep them in sync here.
    updated_theme.colors.mode = if theme_data.is_light_theme {
        "light".to_string()
    } else {
        "dark".to_string()
    };

    let json_path = theme_dir.join("omarchist.json");
    let json_content = serde_json::to_string_pretty(&updated_theme)
        .map_err(|e| Error::json("Failed to serialize theme data", e))?;
    write_atomic(&json_path, json_content, "omarchist.json")?;

    remove_legacy_light_mode_file(&theme_dir)?;

    // Omarchy generates every app's file from colors.toml.
    update_colors_toml(theme_name, &updated_theme.colors)?;

    Ok(())
}

// Loads the theme fresh from disk, applies `edit`, and saves. Every tab of
// the Theme Designer holds its own snapshot of the theme, so tabs must go
// through this to change only the fields they own rather than saving a whole
// stale snapshot over another tab's work.
pub fn update_theme<F>(theme_name: &str, edit: F) -> Result<()>
where
    F: FnOnce(&mut EditingTheme),
{
    // Tabs save from the UI thread and from background tasks; the lock
    // makes each load-edit-save one step so no tab can overwrite another's
    // fields with a stale snapshot.
    let lock = theme_lock(theme_name);
    let _guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut theme = load_theme_for_editing(theme_name)?;
    edit(&mut theme);
    save_theme_data(theme_name, &theme)
}

/// One mutex per theme name, shared by every thread that saves it.
fn theme_lock(theme_name: &str) -> Arc<Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = OnceLock::new();
    let locks = LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut locks = locks
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    locks
        .entry(theme_name.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

/// Renames an Omarchist theme to the folder name `new_name` slugs to (see
/// [`slugify_theme_name`]), so the result is always one folder directly
/// under the themes directory that `omarchy-theme-set` can apply. Returns
/// that folder name.
pub fn rename_theme(old_name: &str, new_name: &str) -> Result<String> {
    if !new_name.chars().any(|c| c.is_ascii_alphanumeric()) {
        return Err(Error::Invalid(
            "A theme name needs at least one letter or digit".into(),
        ));
    }
    let new_name = slugify_theme_name(new_name);
    let new_name = new_name.as_str();
    let themes_dir = get_custom_themes_dir().ok_or(Error::UnknownDirectory("custom themes"))?;

    let old_path = omarchist_theme_dir(old_name)?;
    if new_name == old_name {
        return Ok(new_name.to_string());
    }
    let new_path = themes_dir.join(new_name);

    if name_is_taken(&themes_dir, new_name) {
        return Err(Error::ThemeExists(new_name.to_string()));
    }

    fs::rename(&old_path, &new_path).map_err(|e| Error::io("Failed to rename theme", e))?;

    let json_path = new_path.join("omarchist.json");
    if json_path.exists() {
        let content = fs::read_to_string(&json_path)
            .map_err(|e| Error::io("Failed to read omarchist.json", e))?;
        let mut theme: EditingTheme = serde_json::from_str(&content)
            .map_err(|e| Error::json("Failed to parse omarchist.json", e))?;
        theme.name = new_name.to_string();
        theme.modified_at = Utc::now().to_rfc3339();

        let updated_content = serde_json::to_string_pretty(&theme)
            .map_err(|e| Error::json("Failed to serialize theme data", e))?;
        write_atomic(&json_path, updated_content, "omarchist.json")?;
    }

    Ok(new_name.to_string())
}

// Pre-Quattro Omarchist marked light themes with an empty `light.mode` file.
// Omarchy's resolver (`omarchy-theme-color`) now treats that file as a legacy
// fallback behind the `mode` key in colors.toml, which Omarchist always writes,
// so the marker is removed on save rather than kept in sync.
fn remove_legacy_light_mode_file(theme_dir: &Path) -> Result<()> {
    let light_mode_path = theme_dir.join("light.mode");

    if light_mode_path.exists() {
        fs::remove_file(&light_mode_path)
            .map_err(|e| Error::io("Failed to remove legacy light.mode file", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::slugify_theme_name;

    #[test]
    fn slugify_lowercases_and_collapses_separators() {
        assert_eq!(
            slugify_theme_name("My Wallpaper (1).png"),
            "my-wallpaper-1-png"
        );
        assert_eq!(slugify_theme_name("IMG_2024  final"), "img-2024-final");
        assert_eq!(slugify_theme_name("--Tokyo Night--"), "tokyo-night");
        assert_eq!(slugify_theme_name("café ☕"), "caf");
    }

    #[test]
    fn rename_refuses_a_name_with_nothing_to_keep() {
        assert!(super::rename_theme("any", "../..").is_err());
        assert!(super::rename_theme("any", "   ").is_err());
    }

    #[test]
    fn slugify_falls_back_when_nothing_survives() {
        assert_eq!(slugify_theme_name("☕☕"), "custom-theme");
        assert_eq!(slugify_theme_name(""), "custom-theme");
    }
}
