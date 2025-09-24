use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct HyprlandGenerator;

unsafe impl Send for HyprlandGenerator {}
unsafe impl Sync for HyprlandGenerator {}

impl ConfigGenerator for HyprlandGenerator {
    fn get_app_name(&self) -> &'static str {
        "hyprland"
    }

    fn get_file_name(&self) -> &'static str {
        "hyprland.conf"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let hyprland = theme_data.get("hyprland").unwrap_or(&empty_obj);

        // Extract color values with defaults from template
        let general = hyprland.get("general").unwrap_or(&empty_obj);
        let mut active_border = general
            .get("active_border")
            .and_then(|a| a.as_str())
            .unwrap_or("8A8A8D")
            .to_string();
        // normalize to hex without leading '#'
        if active_border.starts_with('#') {
            active_border = active_border.trim_start_matches('#').to_string();
        }
        let mut inactive_border = general
            .get("inactive_border")
            .and_then(|a| a.as_str())
            .unwrap_or("5C5C5E")
            .to_string();
        // normalize to hex without leading '#'
        if inactive_border.starts_with('#') {
            inactive_border = inactive_border.trim_start_matches('#').to_string();
        }
        let border_size = general
            .get("border_size")
            .and_then(|a| a.as_u64())
            .unwrap_or(1)
            .to_string();
        let gaps_in = general
            .get("gaps_in")
            .and_then(|a| a.as_u64())
            .unwrap_or(5)
            .to_string();
        let gaps_out = general
            .get("gaps_out")
            .and_then(|a| a.as_u64())
            .unwrap_or(10)
            .to_string();

        let decoration = hyprland.get("decoration").unwrap_or(&empty_obj);
        let rounding = decoration
            .get("rounding")
            .and_then(|a| a.as_u64())
            .unwrap_or(0)
            .to_string();

        Ok(format!(
            r#"# ────────────────────────────────────────────────────────────
# Omarchy Custom Theme for Hyprland
# Generated with Omarchist
# ────────────────────────────────────────────────────────────

general {{
    col.active_border = rgb({active_border})
    col.inactive_border = rgb({inactive_border})
    border_size = {border_size}
    gaps_in = {gaps_in}
    gaps_out = {gaps_out}
}}

decoration {{
    rounding = {rounding}
}}
"#
        ))
    }

    fn get_config_schema(&self) -> Value {
        json!({
            "type": "object",
            "x-order": ["general", "decoration"],
            "properties": {
                "general": {
                    "type": "object",
                    "x-order": ["active_border", "inactive_border", "border_size", "gaps_in", "gaps_out"],
                    "properties": {
                        "active_border": {
                            "type": "string",
                            "title": "Active Border",
                            "format": "color",
                            "description": "border color for the active window",
                            "output_format": "hex-no-hash",
                            "default": "8A8A8D",
                        },
                        "inactive_border": {
                            "type": "string",
                            "title": "Inactive Border",
                            "format": "color",
                            "description": "border color for inactive windows",
                            "output_format": "hex-no-hash",
                            "default": "5C5C5E",
                        },
                        "border_size": {
                            "type": "number",
                            "title": "Border Size",
                            "description": "size of the border around windows",
                            "default": 1
                        },
                        "gaps_in": {
                            "type": "number",
                            "title": "Gaps In",
                            "description": "gaps between windows",
                            "default": 5
                        },
                        "gaps_out": {
                            "type": "number",
                            "title": "Gaps Out",
                            "description": "gaps between windows and monitor edges",
                            "default": 10
                        }
                    }
                },
                "decoration": {
                    "type": "object",
                    "properties": {
                        "rounding": {
                            "type": "number",
                            "title": "Rounding",
                            "description": "rounded corners' radius",
                            "default": 0
                        }
                    }
                }
            }
        })
    }

    fn parse_existing_config(&self, _content: &str) -> Result<Value, String> {
        // For now, return empty - could implement conf file parsing if needed
        Ok(json!({}))
    }
}
