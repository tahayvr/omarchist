use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct HyprlockGenerator;

unsafe impl Send for HyprlockGenerator {}
unsafe impl Sync for HyprlockGenerator {}

impl ConfigGenerator for HyprlockGenerator {
    fn get_app_name(&self) -> &'static str {
        "hyprlock"
    }

    fn get_file_name(&self) -> &'static str {
        "hyprlock.conf"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let hyprlock = theme_data.get("hyprlock").unwrap_or(&empty_obj);

        // Extract color values with defaults from template
        let colors = hyprlock.get("colors").unwrap_or(&empty_obj);
        let color = colors
            .get("color")
            .and_then(|c| c.as_str())
            .unwrap_or("12,12,12,1.0");
        let inner_color = colors
            .get("inner_color")
            .and_then(|i| i.as_str())
            .unwrap_or("138,138,141,0.3");
        let outer_color = colors
            .get("outer_color")
            .and_then(|o| o.as_str())
            .unwrap_or("234,234,234,0.5");
        let font_color = colors
            .get("font_color")
            .and_then(|f| f.as_str())
            .unwrap_or("234,234,234,1.0");
        let check_color = colors
            .get("check_color")
            .and_then(|c| c.as_str())
            .unwrap_or("245,158,11,1.0");

        Ok(format!(
            r#"# ────────────────────────────────────────────────────────────
# Omarchy Custom Theme for Hyprlock
# Generated with Omarchist
# ────────────────────────────────────────────────────────────

$color = rgba({color})
$inner_color = rgba({inner_color})
$outer_color = rgba({outer_color})
$font_color = rgba({font_color})
$check_color = rgba({check_color})
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
                        "color": {
                            "type": "string",
                            "output_format": "rgba-comma",
                            "default": "12,12,12,1.0",
                            "title": "Background Color",
                            "description": "Main background color (RGBA format)"
                        },
                        "inner_color": {
                            "type": "string",
                            "output_format": "rgba-comma",
                            "default": "138,138,141,0.3",
                            "title": "Inner Color",
                            "description": "Inner element color (RGBA format)"
                        },
                        "outer_color": {
                            "type": "string",
                            "output_format": "rgba-comma",
                            "default": "234,234,234,0.5",
                            "title": "Outer Color",
                            "description": "Outer element color (RGBA format)"
                        },
                        "font_color": {
                            "type": "string",
                            "output_format": "rgba-comma",
                            "default": "234,234,234,1.0",
                            "title": "Font Color",
                            "description": "Text color (RGBA format)"
                        },
                        "check_color": {
                            "type": "string",
                            "output_format": "rgba-comma",
                            "default": "245,158,11,1.0",
                            "title": "Check Color",
                            "description": "Check/accent color (RGBA format)"
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
