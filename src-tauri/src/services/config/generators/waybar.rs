use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct WaybarGenerator;

unsafe impl Send for WaybarGenerator {}
unsafe impl Sync for WaybarGenerator {}

impl ConfigGenerator for WaybarGenerator {
    fn get_app_name(&self) -> &'static str {
        "waybar"
    }

    fn get_file_name(&self) -> &'static str {
        "waybar.css"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let waybar = theme_data.get("waybar").unwrap_or(&empty_obj);

        // Extract color variables with defaults from template
        let empty_colors = json!({});
        let colors = waybar.get("colors").unwrap_or(&empty_colors);
        let main = colors.get("main").unwrap_or(&empty_colors);
        let bg = main
            .get("background")
            .and_then(|b| b.as_str())
            .unwrap_or("#1e1e1e");
        let fg = main
            .get("foreground")
            .and_then(|f| f.as_str())
            .unwrap_or("#8a8a8d");
        Ok(format!(
            r#"/* ────────────────────────────────────────────────────────────
 * Omarchy Custom Theme for Waybar
 * Generated with Omarchist
 * ────────────────────────────────────────────────────────────
 */

@define-color background {bg};
@define-color foreground {fg};
"#
        ))
    }

    fn get_config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "colors": {
                    "type": "object",
                    "properties": {
                        "main": {
                            "type": "object",
                            "properties": {
                                "background": {"type": "string", "format": "color", "title": "Background", "default": "#1e1e1e"},
                                "foreground": {"type": "string", "format": "color", "title": "Foreground", "default": "#8a8a8d"}
                            }
                        },
                    }
                }
            }
        })
    }

    fn parse_existing_config(&self, _content: &str) -> Result<Value, String> {
        // For now, return empty - could implement CSS parsing if needed
        Ok(json!({}))
    }
}
