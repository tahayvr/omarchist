use super::ConfigGenerator;
use serde_json::{json, Value};

#[allow(unused_macros)]
macro_rules! big_json {
    ($($json:tt)+) => {
        json!($($json)+)
    };
}

pub struct BtopGenerator;

unsafe impl Send for BtopGenerator {}
unsafe impl Sync for BtopGenerator {}

impl ConfigGenerator for BtopGenerator {
    fn get_app_name(&self) -> &'static str {
        "btop"
    }

    fn get_file_name(&self) -> &'static str {
        "btop.theme"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let btop = theme_data.get("btop").unwrap_or(&empty_obj);
        let colors = btop.get("colors").unwrap_or(&empty_obj);

        // gets nested color values from sections
        let get_color = |section: &str, field: &str, default: &str| -> String {
            colors.get(section)
                .and_then(|s| s.get(field))
                .and_then(|v| v.as_str())
                .unwrap_or(default)
                .to_string()
        };

        // Extract colors with defaults from template
        let main_bg = get_color("basic", "main_bg", "");
        let main_fg = get_color("basic", "main_fg", "#EAEAEA");
        let title = get_color("basic", "title", "#8a8a8d");
        let hi_fg = get_color("basic", "hi_fg", "#f59e0b");
        let selected_bg = get_color("basic", "selected_bg", "#f59e0b");
        let selected_fg = get_color("basic", "selected_fg", "#EAEAEA");
        let inactive_fg = get_color("basic", "inactive_fg", "#333333");
        let proc_misc = get_color("basic", "proc_misc", "#8a8a8d");
        let cpu_box = get_color("boxes", "cpu_box", "#8a8a8d");
        let mem_box = get_color("boxes", "mem_box", "#8a8a8d");
        let net_box = get_color("boxes", "net_box", "#8a8a8d");
        let proc_box = get_color("boxes", "proc_box", "#8a8a8d");
        let div_line = get_color("boxes", "div_line", "#8a8a8d");
        let temp_start = get_color("temperature", "temp_start", "#8a8a8d");
        let temp_mid = get_color("temperature", "temp_mid", "#f59e0b");
        let temp_end = get_color("temperature", "temp_end", "#b91c1c");
        let cpu_start = get_color("cpu", "cpu_start", "#8a8a8d");
        let cpu_mid = get_color("cpu", "cpu_mid", "#f59e0b");
        let cpu_end = get_color("cpu", "cpu_end", "#b91c1c");
        let free_start = get_color("memory", "free_start", "#8a8a8d");
        let free_mid = get_color("memory", "free_mid", "#f59e0b");
        let free_end = get_color("memory", "free_end", "#b91c1c");
        let cached_start = get_color("memory", "cached_start", "#8a8a8d");
        let cached_mid = get_color("memory", "cached_mid", "#f59e0b");
        let cached_end = get_color("memory", "cached_end", "#b91c1c");
        let available_start = get_color("memory", "available_start", "#8a8a8d");
        let available_mid = get_color("memory", "available_mid", "#f59e0b");
        let available_end = get_color("memory", "available_end", "#b91c1c");
        let used_start = get_color("memory", "used_start", "#8a8a8d");
        let used_mid = get_color("memory", "used_mid", "#f59e0b");
        let used_end = get_color("memory", "used_end", "#b91c1c");
        let download_start = get_color("network", "download_start", "#8a8a8d");
        let download_mid = get_color("network", "download_mid", "#f59e0b");
        let download_end = get_color("network", "download_end", "#b91c1c");
        let upload_start = get_color("network", "upload_start", "#8a8a8d");
        let upload_mid = get_color("network", "upload_mid", "#f59e0b");
        let upload_end = get_color("network", "upload_end", "#b91c1c");

        Ok(format!(
            r#"# ────────────────────────────────────────────────────────────
# Omarchy Custom Theme for btop
# Generated with Omarchist
# ────────────────────────────────────────────────────────────

# Main background, empty for terminal default, need to be empty if you want transparent background
theme[main_bg]="{main_bg}"

# Main text color
theme[main_fg]="{main_fg}"

# Title color for boxes
theme[title]="{title}"

# Highlight color for keyboard shortcuts
theme[hi_fg]="{hi_fg}"

# Background color of selected item in processes box
theme[selected_bg]="{selected_bg}"

# Foreground color of selected item in processes box
theme[selected_fg]="{selected_fg}"

# Color of inactive/disabled text
theme[inactive_fg]="{inactive_fg}"

# Misc colors for processes box including mini cpu graphs, details memory graph and details status text
theme[proc_misc]="{proc_misc}"

# Cpu box outline color
theme[cpu_box]="{cpu_box}"

# Memory/disks box outline color
theme[mem_box]="{mem_box}"

# Net up/down box outline color
theme[net_box]="{net_box}"

# Processes box outline color
theme[proc_box]="{proc_box}"

# Box divider line and small boxes line color
theme[div_line]="{div_line}"

# Temperature graph colors
theme[temp_start]="{temp_start}"
theme[temp_mid]="{temp_mid}"
theme[temp_end]="{temp_end}"

# CPU graph colors
theme[cpu_start]="{cpu_start}"
theme[cpu_mid]="{cpu_mid}"
theme[cpu_end]="{cpu_end}"

# Mem/Disk free meter
theme[free_start]="{free_start}"
theme[free_mid]="{free_mid}"
theme[free_end]="{free_end}"

# Mem/Disk cached meter
theme[cached_start]="{cached_start}"
theme[cached_mid]="{cached_mid}"
theme[cached_end]="{cached_end}"

# Mem/Disk available meter
theme[available_start]="{available_start}"
theme[available_mid]="{available_mid}"
theme[available_end]="{available_end}"

# Mem/Disk used meter
theme[used_start]="{used_start}"
theme[used_mid]="{used_mid}"
theme[used_end]="{used_end}"

# Download graph colors
theme[download_start]="{download_start}"
theme[download_mid]="{download_mid}"
theme[download_end]="{download_end}"

# Upload graph colors
theme[upload_start]="{upload_start}"
theme[upload_mid]="{upload_mid}"
theme[upload_end]="{upload_end}"
"#
        ))
    }

    fn get_config_schema(&self) -> Value {
        let mut properties = serde_json::Map::new();

        // Basic Colors Section
        let mut basic_properties = serde_json::Map::new();
        // Defaults mirror src-tauri/src/data/template/btop.theme
        basic_properties.insert(
            "main_bg".to_string(),
            json!({"type": "string", "format": "color", "title": "Main Background", "default": ""}),
        );
        basic_properties.insert("main_fg".to_string(), json!({"type": "string", "format": "color", "title": "Main Foreground", "default": "#EAEAEA"}));
        basic_properties.insert("title".to_string(), json!({"type": "string", "format": "color", "title": "Title Color", "default": "#8a8a8d"}));
        basic_properties.insert("hi_fg".to_string(), json!({"type": "string", "format": "color", "title": "Highlight Color", "default": "#f59e0b"}));
        basic_properties.insert("selected_bg".to_string(), json!({"type": "string", "format": "color", "title": "Selected Background", "default": "#f59e0b"}));
        basic_properties.insert("selected_fg".to_string(), json!({"type": "string", "format": "color", "title": "Selected Foreground", "default": "#EAEAEA"}));
        basic_properties.insert("inactive_fg".to_string(), json!({"type": "string", "format": "color", "title": "Inactive Text", "default": "#333333"}));
        basic_properties.insert("proc_misc".to_string(), json!({"type": "string", "format": "color", "title": "Process Misc", "default": "#8a8a8d"}));
        properties.insert(
            "basic".to_string(),
            json!({"type": "object", "title": "Basic Colors", "properties": basic_properties}),
        );

        // Box Outlines Section
        let mut box_properties = serde_json::Map::new();
        box_properties.insert("cpu_box".to_string(), json!({"type": "string", "format": "color", "title": "CPU Box Outline", "default": "#8a8a8d"}));
        box_properties.insert("mem_box".to_string(), json!({"type": "string", "format": "color", "title": "Memory Box Outline", "default": "#8a8a8d"}));
        box_properties.insert("net_box".to_string(), json!({"type": "string", "format": "color", "title": "Network Box Outline", "default": "#8a8a8d"}));
        box_properties.insert("proc_box".to_string(), json!({"type": "string", "format": "color", "title": "Process Box Outline", "default": "#8a8a8d"}));
        box_properties.insert("div_line".to_string(), json!({"type": "string", "format": "color", "title": "Divider Line", "default": "#8a8a8d"}));
        properties.insert(
            "boxes".to_string(),
            json!({"type": "object", "title": "Box Outlines", "properties": box_properties}),
        );

        // Temperature Graph Section
        let mut temp_properties = serde_json::Map::new();
        temp_properties.insert("temp_start".to_string(), json!({"type": "string", "format": "color", "title": "Start Color", "default": "#8a8a8d"}));
        temp_properties.insert("temp_mid".to_string(), json!({"type": "string", "format": "color", "title": "Mid Color", "default": "#f59e0b"}));
        temp_properties.insert("temp_end".to_string(), json!({"type": "string", "format": "color", "title": "End Color", "default": "#b91c1c"}));
        properties.insert(
            "temperature".to_string(),
            json!({"type": "object", "title": "Temperature Graph", "properties": temp_properties}),
        );

        // CPU Graph Section
        let mut cpu_properties = serde_json::Map::new();
        cpu_properties.insert("cpu_start".to_string(), json!({"type": "string", "format": "color", "title": "Start Color", "default": "#8a8a8d"}));
        cpu_properties.insert("cpu_mid".to_string(), json!({"type": "string", "format": "color", "title": "Mid Color", "default": "#f59e0b"}));
        cpu_properties.insert("cpu_end".to_string(), json!({"type": "string", "format": "color", "title": "End Color", "default": "#b91c1c"}));
        properties.insert(
            "cpu".to_string(),
            json!({"type": "object", "title": "CPU Graph", "properties": cpu_properties}),
        );

        // Memory Meters Section
        let mut memory_properties = serde_json::Map::new();
        memory_properties.insert("free_start".to_string(), json!({"type": "string", "format": "color", "title": "Free Start", "default": "#8a8a8d"}));
        memory_properties.insert(
            "free_mid".to_string(),
            json!({"type": "string", "format": "color", "title": "Free Mid", "default": "#f59e0b"}),
        );
        memory_properties.insert(
            "free_end".to_string(),
            json!({"type": "string", "format": "color", "title": "Free End", "default": "#b91c1c"}),
        );
        memory_properties.insert("cached_start".to_string(), json!({"type": "string", "format": "color", "title": "Cached Start", "default": "#8a8a8d"}));
        memory_properties.insert("cached_mid".to_string(), json!({"type": "string", "format": "color", "title": "Cached Mid", "default": "#f59e0b"}));
        memory_properties.insert("cached_end".to_string(), json!({"type": "string", "format": "color", "title": "Cached End", "default": "#b91c1c"}));
        memory_properties.insert("available_start".to_string(), json!({"type": "string", "format": "color", "title": "Available Start", "default": "#8a8a8d"}));
        memory_properties.insert("available_mid".to_string(), json!({"type": "string", "format": "color", "title": "Available Mid", "default": "#f59e0b"}));
        memory_properties.insert("available_end".to_string(), json!({"type": "string", "format": "color", "title": "Available End", "default": "#b91c1c"}));
        memory_properties.insert("used_start".to_string(), json!({"type": "string", "format": "color", "title": "Used Start", "default": "#8a8a8d"}));
        memory_properties.insert(
            "used_mid".to_string(),
            json!({"type": "string", "format": "color", "title": "Used Mid", "default": "#f59e0b"}),
        );
        memory_properties.insert(
            "used_end".to_string(),
            json!({"type": "string", "format": "color", "title": "Used End", "default": "#b91c1c"}),
        );
        properties.insert(
            "memory".to_string(),
            json!({"type": "object", "title": "Memory Meters", "properties": memory_properties}),
        );

        // Network Meters Section
        let mut network_properties = serde_json::Map::new();
        network_properties.insert("download_start".to_string(), json!({"type": "string", "format": "color", "title": "Download Start", "default": "#8a8a8d"}));
        network_properties.insert("download_mid".to_string(), json!({"type": "string", "format": "color", "title": "Download Mid", "default": "#f59e0b"}));
        network_properties.insert("download_end".to_string(), json!({"type": "string", "format": "color", "title": "Download End", "default": "#b91c1c"}));
        network_properties.insert("upload_start".to_string(), json!({"type": "string", "format": "color", "title": "Upload Start", "default": "#8a8a8d"}));
        network_properties.insert("upload_mid".to_string(), json!({"type": "string", "format": "color", "title": "Upload Mid", "default": "#f59e0b"}));
        network_properties.insert("upload_end".to_string(), json!({"type": "string", "format": "color", "title": "Upload End", "default": "#b91c1c"}));
        properties.insert(
            "network".to_string(),
            json!({"type": "object", "title": "Network Meters", "properties": network_properties}),
        );

        json!({
            "type": "object",
            "properties": {
                "colors": {
                    "type": "object",
                    "properties": properties
                }
            }
        })
    }

    fn parse_existing_config(&self, _content: &str) -> Result<Value, String> {
        // For now, return empty - could implement theme file parsing if needed
        Ok(json!({}))
    }
}
