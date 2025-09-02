use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct MakoGenerator;

unsafe impl Send for MakoGenerator {}
unsafe impl Sync for MakoGenerator {}

impl ConfigGenerator for MakoGenerator {
    fn get_app_name(&self) -> &'static str {
        "mako"
    }

    fn get_file_name(&self) -> &'static str {
        "mako.ini"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let mako = theme_data.get("mako").unwrap_or(&empty_obj);

        // Extract color values with defaults from template
        let colors = mako.get("colors").unwrap_or(&empty_obj);
        let normal = colors.get("normal").unwrap_or(&empty_obj);
        let text_color = normal
            .get("text_color")
            .and_then(|t| t.as_str())
            .unwrap_or("#8A8A8D");
        let border_color = normal
            .get("border_color")
            .and_then(|b| b.as_str())
            .unwrap_or("#8A8A8D");
        let background_color = normal
            .get("background_color")
            .and_then(|bg| bg.as_str())
            .unwrap_or("#1E1E1E");

        Ok(format!(
            r#"# ────────────────────────────────────────────────────────────
# Omarchy Custom Theme for Mako
# Generated with Omarchist
# ────────────────────────────────────────────────────────────

text-color={text_color}
border-color={border_color}
background-color={background_color}
width=420
height=110
padding=10
border-size=2
font=Liberation Sans 11
anchor=top-right
outer-margin=20
default-timeout=5000
max-icon-size=32

[app-name=Spotify]
invisible=1

[mode=do-not-disturb]
invisible=true

[mode=do-not-disturb app-name=notify-send]
invisible=false
"#,
        ))
    }

    fn get_config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "colors": {
                    "type": "object",
                    "properties": {
                        "normal": {
                            "type": "object",
                            "properties": {
                                "border_color": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Border Color",
                                    "description": "Color of notification border",
                                    "default": "#8A8A8D",
                                },
                                "background_color": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Background Color",
                                    "description": "Background color of notifications",
                                    "default": "#1E1E1E",
                                },
                                "text_color": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Text Color",
                                    "description": "Color of notification text",
                                    "default": "#8A8A8D",
                                },
                            }
                        },                      
                    }
                }
            }
        })
    }

    fn parse_existing_config(&self, _content: &str) -> Result<Value, String> {
        // For now, return empty - could implement ini file parsing if needed
        Ok(json!({}))
    }
}
