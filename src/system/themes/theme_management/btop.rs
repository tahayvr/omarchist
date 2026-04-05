use std::fs;

use crate::types::themes::BtopConfig;

use super::paths::get_custom_themes_dir;

pub(super) fn parse_btop_theme(theme_content: &str) -> Option<BtopConfig> {
    let mut main_bg = None;
    let mut main_fg = None;
    let mut title = None;
    let mut hi_fg = None;
    let mut selected_bg = None;
    let mut selected_fg = None;
    let mut inactive_fg = None;
    let mut proc_misc = None;
    let mut cpu_box = None;
    let mut mem_box = None;
    let mut net_box = None;
    let mut proc_box = None;
    let mut div_line = None;
    let mut temp_start = None;
    let mut temp_mid = None;
    let mut temp_end = None;
    let mut cpu_start = None;
    let mut cpu_mid = None;
    let mut cpu_end = None;
    let mut free_start = None;
    let mut free_mid = None;
    let mut free_end = None;
    let mut cached_start = None;
    let mut cached_mid = None;
    let mut cached_end = None;
    let mut available_start = None;
    let mut available_mid = None;
    let mut available_end = None;
    let mut used_start = None;
    let mut used_mid = None;
    let mut used_end = None;
    let mut download_start = None;
    let mut download_mid = None;
    let mut download_end = None;
    let mut upload_start = None;
    let mut upload_mid = None;
    let mut upload_end = None;

    for line in theme_content.lines() {
        let trimmed = line.trim();

        if let Some(key_value) = trimmed.strip_prefix("theme[")
            && let Some(end_idx) = key_value.find("]=\"")
        {
            let key = &key_value[..end_idx];
            let value_part = &key_value[end_idx + 3..];
            if let Some(end_quote) = value_part.find('"') {
                let value = value_part[..end_quote].to_string();

                match key {
                    "main_bg" => main_bg = Some(value),
                    "main_fg" => main_fg = Some(value),
                    "title" => title = Some(value),
                    "hi_fg" => hi_fg = Some(value),
                    "selected_bg" => selected_bg = Some(value),
                    "selected_fg" => selected_fg = Some(value),
                    "inactive_fg" => inactive_fg = Some(value),
                    "proc_misc" => proc_misc = Some(value),
                    "cpu_box" => cpu_box = Some(value),
                    "mem_box" => mem_box = Some(value),
                    "net_box" => net_box = Some(value),
                    "proc_box" => proc_box = Some(value),
                    "div_line" => div_line = Some(value),
                    "temp_start" => temp_start = Some(value),
                    "temp_mid" => temp_mid = Some(value),
                    "temp_end" => temp_end = Some(value),
                    "cpu_start" => cpu_start = Some(value),
                    "cpu_mid" => cpu_mid = Some(value),
                    "cpu_end" => cpu_end = Some(value),
                    "free_start" => free_start = Some(value),
                    "free_mid" => free_mid = Some(value),
                    "free_end" => free_end = Some(value),
                    "cached_start" => cached_start = Some(value),
                    "cached_mid" => cached_mid = Some(value),
                    "cached_end" => cached_end = Some(value),
                    "available_start" => available_start = Some(value),
                    "available_mid" => available_mid = Some(value),
                    "available_end" => available_end = Some(value),
                    "used_start" => used_start = Some(value),
                    "used_mid" => used_mid = Some(value),
                    "used_end" => used_end = Some(value),
                    "download_start" => download_start = Some(value),
                    "download_mid" => download_mid = Some(value),
                    "download_end" => download_end = Some(value),
                    "upload_start" => upload_start = Some(value),
                    "upload_mid" => upload_mid = Some(value),
                    "upload_end" => upload_end = Some(value),
                    _ => {}
                }
            }
        }
    }

    if main_bg.is_some()
        || main_fg.is_some()
        || title.is_some()
        || hi_fg.is_some()
        || selected_bg.is_some()
    {
        Some(BtopConfig {
            main_bg: main_bg.unwrap_or_else(|| "#0F0F19".to_string()),
            main_fg: main_fg.unwrap_or_else(|| "#EDEDFE".to_string()),
            title: title.unwrap_or_else(|| "#6e6e92".to_string()),
            hi_fg: hi_fg.unwrap_or_else(|| "#33A1FF".to_string()),
            selected_bg: selected_bg.unwrap_or_else(|| "#f59e0b".to_string()),
            selected_fg: selected_fg.unwrap_or_else(|| "#EDEDFE".to_string()),
            inactive_fg: inactive_fg.unwrap_or_else(|| "#333333".to_string()),
            proc_misc: proc_misc.unwrap_or_else(|| "#8a8a8d".to_string()),
            cpu_box: cpu_box.unwrap_or_else(|| "#6e6e92".to_string()),
            mem_box: mem_box.unwrap_or_else(|| "#6e6e92".to_string()),
            net_box: net_box.unwrap_or_else(|| "#6e6e92".to_string()),
            proc_box: proc_box.unwrap_or_else(|| "#6e6e92".to_string()),
            div_line: div_line.unwrap_or_else(|| "#6e6e92".to_string()),
            temp_start: temp_start.unwrap_or_else(|| "#00F59B".to_string()),
            temp_mid: temp_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            temp_end: temp_end.unwrap_or_else(|| "#FF3366".to_string()),
            cpu_start: cpu_start.unwrap_or_else(|| "#00F59B".to_string()),
            cpu_mid: cpu_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            cpu_end: cpu_end.unwrap_or_else(|| "#FF3366".to_string()),
            free_start: free_start.unwrap_or_else(|| "#00F59B".to_string()),
            free_mid: free_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            free_end: free_end.unwrap_or_else(|| "#FF3366".to_string()),
            cached_start: cached_start.unwrap_or_else(|| "#00F59B".to_string()),
            cached_mid: cached_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            cached_end: cached_end.unwrap_or_else(|| "#FF3366".to_string()),
            available_start: available_start.unwrap_or_else(|| "#00F59B".to_string()),
            available_mid: available_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            available_end: available_end.unwrap_or_else(|| "#FF3366".to_string()),
            used_start: used_start.unwrap_or_else(|| "#00F59B".to_string()),
            used_mid: used_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            used_end: used_end.unwrap_or_else(|| "#FF3366".to_string()),
            download_start: download_start.unwrap_or_else(|| "#00F59B".to_string()),
            download_mid: download_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            download_end: download_end.unwrap_or_else(|| "#FF3366".to_string()),
            upload_start: upload_start.unwrap_or_else(|| "#00F59B".to_string()),
            upload_mid: upload_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            upload_end: upload_end.unwrap_or_else(|| "#FF3366".to_string()),
        })
    } else {
        None
    }
}

pub fn update_btop_theme(theme_name: &str, config: &BtopConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let theme_content = format!(
        r#"# Main background, empty for terminal default, need to be empty if you want transparent background
theme[main_bg]="{}"

# Main text color
theme[main_fg]="{}"

# Title color for boxes
theme[title]="{}"

# Highlight color for keyboard shortcuts
theme[hi_fg]="{}"

# Background color of selected item in processes box
theme[selected_bg]="{}"

# Foreground color of selected item in processes box
theme[selected_fg]="{}"

# Color of inactive/disabled text
theme[inactive_fg]="{}"

# Misc colors for processes box including mini cpu graphs, details memory graph and details status text
theme[proc_misc]="{}"

# Cpu box outline color
theme[cpu_box]="{}"

# Memory/disks box outline color
theme[mem_box]="{}"

# Net up/down box outline color
theme[net_box]="{}"

# Processes box outline color
theme[proc_box]="{}"

# Box divider line and small boxes line color
theme[div_line]="{}"

# Temperature graph colors
theme[temp_start]="{}"
theme[temp_mid]="{}"
theme[temp_end]="{}"

# CPU graph colors
theme[cpu_start]="{}"
theme[cpu_mid]="{}"
theme[cpu_end]="{}"

# Mem/Disk free meter
theme[free_start]="{}"
theme[free_mid]="{}"
theme[free_end]="{}"

# Mem/Disk cached meter
theme[cached_start]="{}"
theme[cached_mid]="{}"
theme[cached_end]="{}"

# Mem/Disk available meter
theme[available_start]="{}"
theme[available_mid]="{}"
theme[available_end]="{}"

# Mem/Disk used meter
theme[used_start]="{}"
theme[used_mid]="{}"
theme[used_end]="{}"

# Download graph colors
theme[download_start]="{}"
theme[download_mid]="{}"
theme[download_end]="{}"

# Upload graph colors
theme[upload_start]="{}"
theme[upload_mid]="{}"
theme[upload_end]="{}"
"#,
        config.main_bg,
        config.main_fg,
        config.title,
        config.hi_fg,
        config.selected_bg,
        config.selected_fg,
        config.inactive_fg,
        config.proc_misc,
        config.cpu_box,
        config.mem_box,
        config.net_box,
        config.proc_box,
        config.div_line,
        config.temp_start,
        config.temp_mid,
        config.temp_end,
        config.cpu_start,
        config.cpu_mid,
        config.cpu_end,
        config.free_start,
        config.free_mid,
        config.free_end,
        config.cached_start,
        config.cached_mid,
        config.cached_end,
        config.available_start,
        config.available_mid,
        config.available_end,
        config.used_start,
        config.used_mid,
        config.used_end,
        config.download_start,
        config.download_mid,
        config.download_end,
        config.upload_start,
        config.upload_mid,
        config.upload_end,
    );

    let theme_path = theme_dir.join("btop.theme");
    fs::write(&theme_path, theme_content)
        .map_err(|e| format!("Failed to write btop.theme: {}", e))?;

    Ok(())
}
