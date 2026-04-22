use std::fs;

use super::paths::{
    live_waybar_config_path, live_waybar_dir, omarchist_config_dir, waybar_current_profile_path,
    waybar_profiles_dir,
};
use crate::assets::extract_default_dir;

pub const CUSTOM_WAYBAR_PROFILE: &str = "Custom Waybar";
pub const OMARCHY_DEFAULT_PROFILE: &str = "omarchy-default";
pub const UNKNOWN_MANAGED_PROFILE: &str = "Unknown Applied Profile";
const MANAGED_COMMENT: &str = "// Managed by Omarchist";

pub fn list_waybar_profiles() -> Vec<String> {
    if let Err(e) = ensure_default_waybar_profile() {
        eprintln!("Failed to ensure default Waybar profile: {e}");
    }

    let Some(profiles_dir) = waybar_profiles_dir() else {
        return vec![];
    };

    fs::read_dir(&profiles_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

pub fn create_waybar_profile(profile_name: &str) -> Result<String, String> {
    ensure_default_waybar_profile()?;

    let name = profile_name.trim();
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let dest = waybar_profiles_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(name);

    if dest.exists() {
        return Err(format!("A profile named \"{}\" already exists", name));
    }

    extract_default_dir("omarchist/waybar/profiles/omarchy-default", &dest)?;

    Ok(name.to_string())
}

pub fn unique_waybar_profile_name(base_name: &str) -> String {
    let base = base_name.trim();
    let fallback = if base.is_empty() {
        "waybar-profile"
    } else {
        base
    };

    let existing = list_waybar_profiles();
    if !existing.iter().any(|name| name == fallback) {
        return fallback.to_string();
    }

    for i in 2.. {
        let candidate = format!("{}-{}", fallback, i);
        if !existing.iter().any(|name| name == &candidate) {
            return candidate;
        }
    }

    fallback.to_string()
}

pub fn rename_waybar_profile(old_name: &str, new_name: &str) -> Result<String, String> {
    if old_name == CUSTOM_WAYBAR_PROFILE {
        return Err("Custom Waybar cannot be renamed".to_string());
    }

    let new = new_name.trim();
    if new.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let profiles_dir =
        waybar_profiles_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let src = profiles_dir.join(old_name);
    let dst = profiles_dir.join(new);

    if !src.exists() {
        return Err(format!("Profile \"{}\" not found", old_name));
    }
    if dst.exists() {
        return Err(format!("A profile named \"{}\" already exists", new));
    }

    fs::rename(&src, &dst).map_err(|e| format!("Failed to rename profile: {}", e))?;

    if read_current_profile().as_deref() == Some(old_name) {
        write_current_profile(new)?;
    }

    Ok(new.to_string())
}

pub fn duplicate_waybar_profile(source_name: &str, new_name: &str) -> Result<String, String> {
    if source_name == CUSTOM_WAYBAR_PROFILE {
        return Err("Custom Waybar cannot be duplicated".to_string());
    }

    let new = new_name.trim();
    if new.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let profiles_dir =
        waybar_profiles_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let src = profiles_dir.join(source_name);
    let dst = profiles_dir.join(new);

    if !src.exists() {
        return Err(format!("Profile \"{}\" not found", source_name));
    }
    if dst.exists() {
        return Err(format!("A profile named \"{}\" already exists", new));
    }

    copy_dir_recursive(&src, &dst)?;

    Ok(new.to_string())
}

pub fn delete_waybar_profile(profile_name: &str) -> Result<Option<String>, String> {
    if profile_name == CUSTOM_WAYBAR_PROFILE {
        return Err("Custom Waybar cannot be deleted".to_string());
    }

    let profiles_dir =
        waybar_profiles_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let target = profiles_dir.join(profile_name);
    if !target.exists() {
        return Err(format!("Profile \"{}\" not found", profile_name));
    }

    let remaining: Vec<String> = list_waybar_profiles()
        .into_iter()
        .filter(|n| n != profile_name)
        .collect();

    if remaining.is_empty() {
        return Err("Cannot delete the last profile".to_string());
    }

    fs::remove_dir_all(&target).map_err(|e| format!("Failed to delete profile: {}", e))?;

    let mut sorted = remaining;
    sorted.sort();
    let next = sorted.into_iter().next();

    if read_current_profile().as_deref() == Some(profile_name)
        && let Some(ref switch_to) = next
    {
        write_current_profile(switch_to)?;
    }

    Ok(next)
}

pub fn has_live_waybar_config() -> bool {
    live_waybar_config_path().is_some()
}

pub fn is_live_waybar_managed() -> bool {
    let Some(config_path) = live_waybar_config_path() else {
        return false;
    };

    fs::read_to_string(config_path)
        .map(|raw| raw.starts_with(MANAGED_COMMENT))
        .unwrap_or(false)
}

pub fn ensure_custom_waybar_profile() -> Result<Option<String>, String> {
    ensure_default_waybar_profile()?;
    migrate_backup_original_to_custom_profile()?;

    if !has_live_waybar_config() || is_live_waybar_managed() {
        return Ok(None);
    }

    if has_custom_waybar_profile() {
        return Ok(Some(CUSTOM_WAYBAR_PROFILE.to_string()));
    }

    import_live_waybar_as_profile(CUSTOM_WAYBAR_PROFILE)?;
    Ok(Some(CUSTOM_WAYBAR_PROFILE.to_string()))
}

pub fn current_live_waybar_profile() -> Option<String> {
    if let Some(saved_profile) = read_current_profile() {
        let profile_dir = waybar_profiles_dir()?.join(&saved_profile);
        let live_dir = live_waybar_dir()?;
        if live_dir.exists() && profile_dir.exists() {
            let live_is_managed = is_live_waybar_managed();
            let saved_is_read_only = is_read_only_waybar_profile(&saved_profile);
            let expected_read_only = !live_is_managed;
            if saved_is_read_only == expected_read_only
                && directories_match(&live_dir, &profile_dir).unwrap_or(false)
            {
                return Some(saved_profile);
            }
        }
    }

    let live_dir = live_waybar_dir()?;
    if !live_dir.exists() {
        return None;
    }

    let profiles_dir = waybar_profiles_dir()?;
    let mut profile_names = list_waybar_profiles();
    profile_names.sort();
    let live_is_managed = is_live_waybar_managed();

    if live_is_managed {
        if let Some(profile_name) = profile_names
            .iter()
            .filter(|name| !is_read_only_waybar_profile(name))
            .find(|profile_name| {
                let profile_dir = profiles_dir.join(profile_name);
                directories_match(&live_dir, &profile_dir).unwrap_or(false)
            })
        {
            return Some(profile_name.clone());
        }
    } else if let Some(profile_name) = profile_names
        .iter()
        .filter(|name| is_read_only_waybar_profile(name))
        .find(|profile_name| {
            let profile_dir = profiles_dir.join(profile_name);
            directories_match(&live_dir, &profile_dir).unwrap_or(false)
        })
    {
        return Some(profile_name.clone());
    }

    profile_names.into_iter().find(|profile_name| {
        let profile_dir = profiles_dir.join(profile_name);
        directories_match(&live_dir, &profile_dir).unwrap_or(false)
    })
}

pub fn has_unknown_managed_live_waybar() -> bool {
    has_live_waybar_config() && is_live_waybar_managed() && current_live_waybar_profile().is_none()
}

pub fn has_custom_waybar_profile() -> bool {
    waybar_profiles_dir()
        .map(|dir| dir.join(CUSTOM_WAYBAR_PROFILE).exists())
        .unwrap_or(false)
}

pub fn ensure_default_waybar_profile() -> Result<(), String> {
    let profiles_dir =
        waybar_profiles_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    let default_dir = profiles_dir.join(OMARCHY_DEFAULT_PROFILE);

    if default_dir.exists() {
        return Ok(());
    }

    extract_default_dir("omarchist/waybar/profiles/omarchy-default", &default_dir)
}

pub fn is_read_only_waybar_profile(profile_name: &str) -> bool {
    profile_name == CUSTOM_WAYBAR_PROFILE
}

pub fn apply_waybar_profile(profile_name: &str) -> Result<(), String> {
    ensure_default_waybar_profile()?;

    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let profile_dir = omarchist_config_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join("waybar")
        .join("profiles")
        .join(profile_name);

    if !profile_dir.exists() {
        return Err(format!("Profile \"{}\" not found", profile_name));
    }

    let live_waybar = home.join(".config").join("waybar");

    if live_waybar.exists() {
        fs::remove_dir_all(&live_waybar)
            .map_err(|e| format!("Failed to clear ~/.config/waybar: {}", e))?;
    }

    copy_dir_recursive(&profile_dir, &live_waybar)
        .map_err(|e| format!("Failed to apply profile \"{}\": {}", profile_name, e))?;

    if profile_name == CUSTOM_WAYBAR_PROFILE {
        remove_managed_comment_from_live_config()?;
    } else {
        mark_live_waybar_managed()?;
    }

    write_current_profile(profile_name)?;

    Ok(())
}

pub fn import_live_waybar_as_profile(profile_name: &str) -> Result<String, String> {
    let name = profile_name.trim();
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let live_dir =
        live_waybar_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    if !live_dir.exists() {
        return Err("No live Waybar config found".to_string());
    }

    let profiles_dir =
        waybar_profiles_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    let dst = profiles_dir.join(name);
    if dst.exists() {
        return Err(format!("A profile named \"{}\" already exists", name));
    }

    copy_dir_recursive(&live_dir, &dst)?;
    remove_managed_comment_from_profile(name)?;

    Ok(name.to_string())
}

pub fn start_with_omarchy_default_profile(profile_name: &str) -> Result<String, String> {
    create_waybar_profile(profile_name)
}

pub fn adopt_live_waybar(profile_name: &str) -> Result<String, String> {
    let imported = import_live_waybar_as_profile(profile_name)?;
    apply_waybar_profile(&imported)?;
    Ok(imported)
}

fn original_waybar_backup_dir() -> Option<std::path::PathBuf> {
    omarchist_config_dir().map(|d| d.join("waybar").join("backup-original"))
}

fn migrate_backup_original_to_custom_profile() -> Result<(), String> {
    if has_custom_waybar_profile() {
        return Ok(());
    }

    let Some(backup_dir) = original_waybar_backup_dir() else {
        return Ok(());
    };

    if !backup_dir.exists() {
        return Ok(());
    }

    let profiles_dir =
        waybar_profiles_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    let custom_dir = profiles_dir.join(CUSTOM_WAYBAR_PROFILE);

    copy_dir_recursive(&backup_dir, &custom_dir)?;
    remove_managed_comment_from_profile(CUSTOM_WAYBAR_PROFILE)?;

    fs::remove_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to remove migrated backup-original: {}", e))?;

    Ok(())
}

fn mark_live_waybar_managed() -> Result<(), String> {
    let config_path =
        live_waybar_config_path().ok_or_else(|| "No live Waybar config found".to_string())?;
    prepend_managed_comment(&config_path)
}

fn remove_managed_comment_from_live_config() -> Result<(), String> {
    let Some(config_path) = live_waybar_config_path() else {
        return Ok(());
    };
    remove_managed_comment(&config_path)
}

fn remove_managed_comment_from_profile(profile_name: &str) -> Result<(), String> {
    let profile_dir = waybar_profiles_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(profile_name);

    let jsonc = profile_dir.join("config.jsonc");
    if jsonc.exists() {
        return remove_managed_comment(&jsonc);
    }

    let plain = profile_dir.join("config");
    if plain.exists() {
        return remove_managed_comment(&plain);
    }

    Ok(())
}

fn prepend_managed_comment(config_path: &std::path::Path) -> Result<(), String> {
    let raw = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config {:?}: {}", config_path, e))?;

    if raw.starts_with(MANAGED_COMMENT) {
        return Ok(());
    }

    let mut new_raw = String::with_capacity(raw.len() + MANAGED_COMMENT.len() + 1);
    new_raw.push_str(MANAGED_COMMENT);
    new_raw.push('\n');
    new_raw.push_str(&raw);
    fs::write(config_path, new_raw)
        .map_err(|e| format!("Failed to write config {:?}: {}", config_path, e))
}

fn remove_managed_comment(config_path: &std::path::Path) -> Result<(), String> {
    let raw = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config {:?}: {}", config_path, e))?;

    let new_raw = raw
        .strip_prefix(MANAGED_COMMENT)
        .map(|rest| rest.strip_prefix('\n').unwrap_or(rest).to_string())
        .unwrap_or(raw);

    fs::write(config_path, new_raw)
        .map_err(|e| format!("Failed to write config {:?}: {}", config_path, e))
}

fn read_current_profile() -> Option<String> {
    let path = waybar_current_profile_path()?;
    let value = fs::read_to_string(path).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn write_current_profile(profile_name: &str) -> Result<(), String> {
    let path = waybar_current_profile_path()
        .ok_or_else(|| "Could not determine current profile path".to_string())?;
    let parent = path
        .parent()
        .ok_or_else(|| "Could not determine current profile directory".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
    fs::write(&path, format!("{}\n", profile_name))
        .map_err(|e| format!("Failed to write current profile {:?}: {}", path, e))
}

fn directories_match(left: &std::path::Path, right: &std::path::Path) -> Result<bool, String> {
    if !left.exists() || !right.exists() {
        return Ok(false);
    }

    let mut left_entries: Vec<_> = fs::read_dir(left)
        .map_err(|e| format!("Failed to read directory {:?}: {}", left, e))?
        .filter_map(|e| e.ok())
        .collect();
    let mut right_entries: Vec<_> = fs::read_dir(right)
        .map_err(|e| format!("Failed to read directory {:?}: {}", right, e))?
        .filter_map(|e| e.ok())
        .collect();

    left_entries.sort_by_key(|entry| entry.file_name());
    right_entries.sort_by_key(|entry| entry.file_name());

    if left_entries.len() != right_entries.len() {
        return Ok(false);
    }

    for (left_entry, right_entry) in left_entries.iter().zip(right_entries.iter()) {
        if left_entry.file_name() != right_entry.file_name() {
            return Ok(false);
        }

        let left_path = left_entry.path();
        let right_path = right_entry.path();
        let left_is_dir = left_path.is_dir();
        let right_is_dir = right_path.is_dir();

        if left_is_dir != right_is_dir {
            return Ok(false);
        }

        if left_is_dir {
            if !directories_match(&left_path, &right_path)? {
                return Ok(false);
            }
        } else if !files_match(&left_path, &right_path)? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn files_match(left: &std::path::Path, right: &std::path::Path) -> Result<bool, String> {
    let left_name = left
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let right_name = right
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    if left_name != right_name {
        return Ok(false);
    }

    if matches!(left_name, "config" | "config.jsonc") {
        let left_raw = fs::read_to_string(left)
            .map_err(|e| format!("Failed to read config {:?}: {}", left, e))?;
        let right_raw = fs::read_to_string(right)
            .map_err(|e| format!("Failed to read config {:?}: {}", right, e))?;
        return Ok(strip_managed_comment(&left_raw) == strip_managed_comment(&right_raw));
    }

    let left_bytes =
        fs::read(left).map_err(|e| format!("Failed to read file {:?}: {}", left, e))?;
    let right_bytes =
        fs::read(right).map_err(|e| format!("Failed to read file {:?}: {}", right, e))?;
    Ok(left_bytes == right_bytes)
}

fn strip_managed_comment(raw: &str) -> &str {
    raw.strip_prefix(MANAGED_COMMENT)
        .map(|rest| rest.strip_prefix('\n').unwrap_or(rest))
        .unwrap_or(raw)
}

pub fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory {:?}: {}", dst, e))?;

    for entry in
        fs::read_dir(src).map_err(|e| format!("Failed to read directory {:?}: {}", src, e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {:?} to {:?}: {}", src_path, dst_path, e))?;
        }
    }

    Ok(())
}
