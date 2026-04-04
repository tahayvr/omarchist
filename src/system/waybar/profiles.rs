use std::fs;
use std::path::PathBuf;

pub fn list_waybar_profiles() -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return vec![];
    };
    let profiles_dir = home
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles");

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
    let name = profile_name.trim();
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let dest = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles")
        .join(name);

    if dest.exists() {
        return Err(format!("A profile named \"{}\" already exists", name));
    }

    let src = PathBuf::from("defaults/omarchist/waybar/profiles/omarchy-default");
    if !src.exists() {
        return Err(format!("Default profile source not found at {:?}", src));
    }

    copy_dir_recursive(&src, &dest)?;

    Ok(name.to_string())
}

pub fn rename_waybar_profile(old_name: &str, new_name: &str) -> Result<String, String> {
    let new = new_name.trim();
    if new.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let profiles_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles");

    let src = profiles_dir.join(old_name);
    let dst = profiles_dir.join(new);

    if !src.exists() {
        return Err(format!("Profile \"{}\" not found", old_name));
    }
    if dst.exists() {
        return Err(format!("A profile named \"{}\" already exists", new));
    }

    fs::rename(&src, &dst).map_err(|e| format!("Failed to rename profile: {}", e))?;

    Ok(new.to_string())
}

pub fn duplicate_waybar_profile(source_name: &str, new_name: &str) -> Result<String, String> {
    let new = new_name.trim();
    if new.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let profiles_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles");

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
    let profiles_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles");

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
    Ok(sorted.into_iter().next())
}

pub fn has_original_waybar_backup() -> bool {
    original_waybar_backup_dir()
        .map(|p| p.exists())
        .unwrap_or(false)
}

pub fn apply_waybar_profile(profile_name: &str) -> Result<(), String> {
    backup_original_waybar_config()?;

    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let profile_dir = home
        .join(".config")
        .join("omarchist")
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
        .map_err(|e| format!("Failed to apply profile \"{}\": {}", profile_name, e))
}

pub fn restore_original_waybar_config() -> Result<(), String> {
    let backup_dir = original_waybar_backup_dir()
        .ok_or_else(|| "Could not determine backup directory".to_string())?;

    if !backup_dir.exists() {
        return Err("No original waybar backup found".to_string());
    }

    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let live_waybar = home.join(".config").join("waybar");

    if live_waybar.exists() {
        fs::remove_dir_all(&live_waybar)
            .map_err(|e| format!("Failed to clear ~/.config/waybar: {}", e))?;
    }

    copy_dir_recursive(&backup_dir, &live_waybar)
        .map_err(|e| format!("Failed to restore original waybar config: {}", e))
}

fn original_waybar_backup_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".config")
            .join("omarchist")
            .join("waybar")
            .join("backup-original")
    })
}

fn backup_original_waybar_config() -> Result<(), String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let live_waybar = home.join(".config").join("waybar");
    if !live_waybar.exists() {
        return Ok(());
    }

    let backup_dir = original_waybar_backup_dir()
        .ok_or_else(|| "Could not determine backup directory".to_string())?;

    if backup_dir.exists() {
        // Backup already taken — never overwrite it.
        return Ok(());
    }

    copy_dir_recursive(&live_waybar, &backup_dir)
        .map_err(|e| format!("Failed to back up original waybar config: {}", e))
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
