use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct VscodeGenerator;

unsafe impl Send for VscodeGenerator {}
unsafe impl Sync for VscodeGenerator {}

impl ConfigGenerator for VscodeGenerator {
    fn get_app_name(&self) -> &'static str {
        "vscode"
    }

    fn get_file_name(&self) -> &'static str {
        "vscode.json"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let vscode = theme_data.get("vscode").unwrap_or(&empty_obj);

        // Check if raw_config is provided
        if let Some(raw_config) = vscode.get("raw_config").and_then(|rc| rc.as_str()) {
            if !raw_config.trim().is_empty() {
                return Ok(raw_config.to_string());
            }
        }

        // Fallback to default template if no raw config provided
        Ok(r#"{
  "name": "Matte Black",
  "extension": "TahaYVR.matteblack"
}"#
        .to_string())
    }

    fn get_config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "raw_config": {
                    "type": "string",
                    "title": "VSCode Configuration",
                    "description": "Raw JSON configuration for VSCode theme settings",
                    "x-component": "textarea"
                }
            }
        })
    }

    fn parse_existing_config(&self, content: &str) -> Result<Value, String> {
        Ok(json!({
            "raw_config": content.trim()
        }))
    }
}
