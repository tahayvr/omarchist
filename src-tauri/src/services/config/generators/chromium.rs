use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct ChromiumGenerator;

unsafe impl Send for ChromiumGenerator {}
unsafe impl Sync for ChromiumGenerator {}

impl ConfigGenerator for ChromiumGenerator {
    fn get_app_name(&self) -> &'static str {
        "chromium"
    }

    fn get_file_name(&self) -> &'static str {
        "chromium.theme"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let chromium = theme_data.get("chromium").unwrap_or(&empty_obj);

        // Extract background color with default
        let theme_color = chromium
            .get("theme_color")
            .and_then(|bg| bg.as_str())
            .unwrap_or("#1e1e1e");

        // Convert hex color to RGB values
        let rgb = hex_to_rgb(theme_color)?;

        Ok(format!("{},{},{}\n", rgb.0, rgb.1, rgb.2))
    }

    fn get_config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "theme_color": {
                    "type": "string",
                    "format": "color",
                    "title": "Theme Color",
                    "description": "Base color for Chromium theme",
                    "default": "#1e1e1e"
                }
            }
        })
    }

    fn parse_existing_config(&self, content: &str) -> Result<Value, String> {
        let rgb_values: Vec<&str> = content.trim().split(',').collect();

        if rgb_values.len() != 3 {
            return Err("Invalid RGB format. Expected format: r,g,b".to_string());
        }

        let r: u8 = rgb_values[0].parse().map_err(|_| "Invalid red value")?;
        let g: u8 = rgb_values[1].parse().map_err(|_| "Invalid green value")?;
        let b: u8 = rgb_values[2].parse().map_err(|_| "Invalid blue value")?;

        let hex_color = format!("#{:02x}{:02x}{:02x}", r, g, b);

        Ok(json!({
            "theme_color": hex_color
        }))
    }
}

/// Convert hex color to RGB tuple
fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), String> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err("Invalid hex color format. Expected format: #RRGGBB".to_string());
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component in hex color")?;
    let g =
        u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component in hex color")?;
    let b =
        u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component in hex color")?;

    Ok((r, g, b))
}
