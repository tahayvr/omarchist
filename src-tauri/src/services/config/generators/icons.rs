use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct IconsGenerator;

unsafe impl Send for IconsGenerator {}
unsafe impl Sync for IconsGenerator {}

impl ConfigGenerator for IconsGenerator {
    fn get_app_name(&self) -> &'static str {
        "icons"
    }

    fn get_file_name(&self) -> &'static str {
        "icons.theme"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let icons = theme_data.get("icons").unwrap_or(&empty_obj);

        // Extract icon theme name with default
        let theme_name = icons
            .get("theme_name")
            .and_then(|tn| tn.as_str())
            .unwrap_or("Yaru-red");

        Ok(format!(
            r#"{theme_name}
"#
        ))
    }

    fn get_config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "theme_name": {
                    "type": "string",
                    "title": "Icon Theme Name",
                    "description": "Name of the icon theme to use",
                    "enum": [
                        "Yaru-red",
                        "Yaru-blue",
                        "Yaru-olive",
                        "Yaru-yellow",
                        "Yaru-purple",
                        "Yaru-magenta",
                        "Yaru-sage",
                    ]
                }
            }
        })
    }

    fn parse_existing_config(&self, content: &str) -> Result<Value, String> {
        let theme_name = content.trim();
        Ok(json!({
            "theme_name": theme_name
        }))
    }
}
