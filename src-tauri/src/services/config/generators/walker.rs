use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct WalkerGenerator;

unsafe impl Send for WalkerGenerator {}
unsafe impl Sync for WalkerGenerator {}

impl ConfigGenerator for WalkerGenerator {
    fn get_app_name(&self) -> &'static str {
        "walker"
    }

    fn get_file_name(&self) -> &'static str {
        "walker.css"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let walker = theme_data.get("walker").unwrap_or(&empty_obj);

        // Extract color values with defaults from template
        let colors = walker.get("colors").unwrap_or(&empty_obj);

        let selected_text = colors
            .get("selected_text")
            .and_then(|st| st.as_str())
            .unwrap_or("#B91C1C");
        let text = colors
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("#EAEAEA");
        let base = colors
            .get("base")
            .and_then(|b| b.as_str())
            .unwrap_or("#121212");
        let border = colors
            .get("border")
            .and_then(|br| br.as_str())
            .unwrap_or("#EAEAEA88");
        let foreground = colors
            .get("foreground")
            .and_then(|fg| fg.as_str())
            .unwrap_or("#EAEAEA");
        let background = colors
            .get("background")
            .and_then(|bg| bg.as_str())
            .unwrap_or("#121212");

        Ok(format!(
            r#"/* ────────────────────────────────────────────────────────────
 * Omarchy Custom Theme for Walker
 * Generated with Omarchist
 * ────────────────────────────────────────────────────────────
 */

@define-color selected-text {selected_text};
@define-color text {text};
@define-color base {base};
@define-color border {border};
@define-color foreground {foreground};
@define-color background {background};
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
                    "selected_text": {
                        "type": "string",
                        "format": "color",
                        "default": "#B91C1C",
                        "title": "Selected Text Color",
                        "description": "Color of selected text"
                    },
                    "text": {
                        "type": "string",
                        "format": "color",
                        "default": "#EAEAEA",
                        "title": "Text Color",
                        "description": "Color of normal text"
                    },
                    "base": {
                        "type": "string",
                        "format": "color",
                        "default": "#121212",
                        "title": "Base Color",
                        "description": "Base background color"
                    },
                    "border": {
                        "type": "string",
                        "format": "color",
                        "output_format": "hex-alpha",
                        "default": "EAEAEA88",
                        "title": "Border Color",
                        "description": "Border color (can include alpha)"
                    },
                    "foreground": {
                        "type": "string",
                        "format": "color",
                        "default": "#EAEAEA",
                        "title": "Foreground Color",
                        "description": "Primary foreground color"
                    },
                    "background": {
                        "type": "string",
                        "format": "color",
                        "default": "#121212",
                        "title": "Background Color",
                        "description": "Primary background color"
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
