use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct SwayosdGenerator;

unsafe impl Send for SwayosdGenerator {}
unsafe impl Sync for SwayosdGenerator {}

impl ConfigGenerator for SwayosdGenerator {
    fn get_app_name(&self) -> &'static str {
        "swayosd"
    }

    fn get_file_name(&self) -> &'static str {
        "swayosd.css"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let swayosd = theme_data.get("swayosd").unwrap_or(&empty_obj);

        // Extract color values with defaults from template
        let colors = swayosd.get("colors").unwrap_or(&empty_obj);
        let background_color = colors
            .get("background_color")
            .and_then(|bg| bg.as_str())
            .unwrap_or("#121212");
        let border_color = colors
            .get("border_color")
            .and_then(|bc| bc.as_str())
            .unwrap_or("#8A8A8D");
        let label = colors
            .get("label")
            .and_then(|l| l.as_str())
            .unwrap_or("#8A8A8D");
        let image = colors
            .get("image")
            .and_then(|i| i.as_str())
            .unwrap_or("#8A8A8D");
        let progress = colors
            .get("progress")
            .and_then(|p| p.as_str())
            .unwrap_or("#8A8A8D");

        Ok(format!(
            r#"/* ────────────────────────────────────────────────────────────
 * Omarchy Custom Theme for SwayOSD
 * Generated with Omarchist
 * ────────────────────────────────────────────────────────────
 */

@define-color background-color {background_color};
@define-color border-color {border_color};
@define-color label {label};
@define-color image {image};
@define-color progress {progress};
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
                            "background_color": {
                                "type": "string",
                                "format": "color",
                                "default": "#121212",
                                "title": "Background Color",
                                "description": "Background color of OSD"
                            },
                            "border_color": {
                                "type": "string",
                                "format": "color",
                                "default": "#8A8A8D",
                                "title": "Border Color",
                                "description": "Border color of OSD"
                            },
                            "label": {
                                "type": "string",
                                "format": "color",
                                "default": "#8A8A8D",
                                "title": "Label Color",
                                "description": "Color of text labels"
                            },
                            "image": {
                                "type": "string",
                                "format": "color",
                                "default": "#8A8A8D",
                                "title": "Image Color",
                                "description": "Color of icons/images"
                            },
                            "progress": {
                                "type": "string",
                                "format": "color",
                                "default": "#8A8A8D",
                                "title": "Progress Color",
                                "description": "Color of progress bars"
                            }
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
