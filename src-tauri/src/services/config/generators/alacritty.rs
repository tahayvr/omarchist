use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct AlacrittyGenerator;

unsafe impl Send for AlacrittyGenerator {}
unsafe impl Sync for AlacrittyGenerator {}

impl ConfigGenerator for AlacrittyGenerator {
    fn get_app_name(&self) -> &'static str {
        "alacritty"
    }

    fn get_file_name(&self) -> &'static str {
        "alacritty.toml"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let alacritty = theme_data.get("alacritty").unwrap_or(&empty_obj);

        // Primary colors
        let primary_bg = alacritty
            .get("colors")
            .and_then(|c| c.get("primary"))
            .and_then(|p| p.get("background"))
            .and_then(|b| b.as_str())
            .unwrap_or("#121212");
        let primary_fg = alacritty
            .get("colors")
            .and_then(|c| c.get("primary"))
            .and_then(|p| p.get("foreground"))
            .and_then(|f| f.as_str())
            .unwrap_or("#bebebe");
        let dim_fg = alacritty
            .get("colors")
            .and_then(|c| c.get("primary"))
            .and_then(|p| p.get("dim_foreground"))
            .and_then(|d| d.as_str())
            .unwrap_or("#8a8a8d");

        // Normal colors
        let empty_normal = json!({});
        let normal = alacritty
            .get("colors")
            .and_then(|c| c.get("normal"))
            .unwrap_or(&empty_normal);
        let normal_black = normal
            .get("black")
            .and_then(|b| b.as_str())
            .unwrap_or("#333333");
        let normal_red = normal
            .get("red")
            .and_then(|r| r.as_str())
            .unwrap_or("#D35F5F");
        let normal_green = normal
            .get("green")
            .and_then(|g| g.as_str())
            .unwrap_or("#FFC107");
        let normal_yellow = normal
            .get("yellow")
            .and_then(|y| y.as_str())
            .unwrap_or("#b91c1c");
        let normal_blue = normal
            .get("blue")
            .and_then(|b| b.as_str())
            .unwrap_or("#e68e0d");
        let normal_magenta = normal
            .get("magenta")
            .and_then(|m| m.as_str())
            .unwrap_or("#D35F5F");
        let normal_cyan = normal
            .get("cyan")
            .and_then(|c| c.as_str())
            .unwrap_or("#bebebe");
        let normal_white = normal
            .get("white")
            .and_then(|w| w.as_str())
            .unwrap_or("#bebebe");

        // Bright colors
        let empty_bright = json!({});
        let bright = alacritty
            .get("colors")
            .and_then(|c| c.get("bright"))
            .unwrap_or(&empty_bright);
        let bright_black = bright
            .get("black")
            .and_then(|b| b.as_str())
            .unwrap_or("#8a8a8d");
        let bright_red = bright
            .get("red")
            .and_then(|r| r.as_str())
            .unwrap_or("#B91C1C");
        let bright_green = bright
            .get("green")
            .and_then(|g| g.as_str())
            .unwrap_or("#FFC107");
        let bright_yellow = bright
            .get("yellow")
            .and_then(|y| y.as_str())
            .unwrap_or("#b90a0a");
        let bright_blue = bright
            .get("blue")
            .and_then(|b| b.as_str())
            .unwrap_or("#f59e0b");
        let bright_magenta = bright
            .get("magenta")
            .and_then(|m| m.as_str())
            .unwrap_or("#b91c1c");
        let bright_cyan = bright
            .get("cyan")
            .and_then(|c| c.as_str())
            .unwrap_or("#eaeaea");
        let bright_white = bright
            .get("white")
            .and_then(|w| w.as_str())
            .unwrap_or("#eaeaea");

        // Extract font settings
        // let font_size = alacritty
        //     .get("font")
        //     .and_then(|f| f.get("size"))
        //     .and_then(|s| s.as_f64())
        //     .unwrap_or(12.0);

        // Extract window settings
        // let empty_window = json!({});
        // let window = alacritty.get("window").unwrap_or(&empty_window);
        // let padding_x = window
        //     .get("padding")
        //     .and_then(|p| p.get("x"))
        //     .and_then(|x| x.as_i64())
        //     .unwrap_or(12);
        // let padding_y = window
        //     .get("padding")
        //     .and_then(|p| p.get("y"))
        //     .and_then(|y| y.as_i64())
        //     .unwrap_or(12);

        // Cursor colors
        let empty_cursor = json!({});
        let cursor_obj = alacritty
            .get("colors")
            .and_then(|c| c.get("cursor"))
            .unwrap_or(&empty_cursor);
        let cursor_text = cursor_obj
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("#121212");
        let cursor_cursor = cursor_obj
            .get("cursor")
            .and_then(|v| v.as_str())
            .unwrap_or("#eaeaea");

        // Vi mode cursor colors
        // let empty_vi_cursor = json!({});
        // let vi_cursor_obj = alacritty
        //     .get("colors")
        //     .and_then(|c| c.get("vi_mode_cursor"))
        //     .unwrap_or(&empty_vi_cursor);
        // let vi_cursor_text = vi_cursor_obj
        //     .get("text")
        //     .and_then(|v| v.as_str())
        //     .unwrap_or("#121212");
        // let vi_cursor_cursor = vi_cursor_obj
        //     .get("cursor")
        //     .and_then(|v| v.as_str())
        //     .unwrap_or("#eaeaea");

        // Selection
        let empty_selection = json!({});
        let selection_obj = alacritty
            .get("colors")
            .and_then(|c| c.get("selection"))
            .unwrap_or(&empty_selection);
        let selection_background = selection_obj
            .get("background")
            .and_then(|v| v.as_str())
            .unwrap_or("#333333");

        Ok(format!(
            r#"# ────────────────────────────────────────────────────────────
# Omarchy Custom Theme for Alacritty
# Generated by Omarchist
# ────────────────────────────────────────────────────────────

[colors]
[colors.primary]
background = "{primary_bg}"
foreground = "{primary_fg}"
dim_foreground = "{dim_fg}"

[colors.cursor]
text = "{cursor_text}"
cursor = "{cursor_cursor}"

[colors.selection]
text = "CellForeground"
background = "{selection_background}"

[colors.normal]
black = "{normal_black}"
red = "{normal_red}"
green = "{normal_green}"
yellow = "{normal_yellow}"
blue = "{normal_blue}"
magenta = "{normal_magenta}"
cyan = "{normal_cyan}"
white = "{normal_white}"

[colors.bright]
black = "{bright_black}"
red = "{bright_red}"
green = "{bright_green}"
yellow = "{bright_yellow}"
blue = "{bright_blue}"
magenta = "{bright_magenta}"
cyan = "{bright_cyan}"
white = "{bright_white}"
"#
        ))
    }

    fn get_config_schema(&self) -> Value {
        json!({
            "type": "object",
            // UI ordering for top-level properties
            "x-order": ["colors", "font", "window"],
            "properties": {
                "colors": {
                    "type": "object",
                    // Ensure colors sections render in intended order
                    "x-order": [
                        "primary",
                        "cursor",
                        "vi_mode_cursor",
                        "selection",
                        "normal",
                        "bright"
                    ],
                    "properties": {
                        "primary": {
                            "type": "object",
                            "x-order": ["background", "foreground", "dim_foreground"],
                            "properties": {
                                "background": {"type": "string", "format": "color", "title": "Background Color"},
                                "foreground": {"type": "string", "format": "color", "title": "Foreground Color"},
                                "dim_foreground": {"type": "string", "format": "color", "title": "Dim Foreground Color"}
                            }
                        },
                        "cursor": {
                            "type": "object",
                            "properties": {
                                "text": {"type": "string", "format": "color", "title": "Cursor Text", "default": "#121212"},
                                "cursor": {"type": "string", "format": "color", "title": "Cursor Color", "default": "#EAEAEA"}
                            }
                        },
                        "selection": {
                            "type": "object",
                            "properties": {
                                "background": {"type": "string", "format": "color", "title": "Selection Background", "default": "#333333"}
                            }
                        },
                        "normal": {
                            "type": "object",
                            "x-order": ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"],
                            "properties": {
                                "black": {"type": "string", "format": "color", "title": "Black"},
                                "red": {"type": "string", "format": "color", "title": "Red"},
                                "green": {"type": "string", "format": "color", "title": "Green"},
                                "yellow": {"type": "string", "format": "color", "title": "Yellow"},
                                "blue": {"type": "string", "format": "color", "title": "Blue"},
                                "magenta": {"type": "string", "format": "color", "title": "Magenta"},
                                "cyan": {"type": "string", "format": "color", "title": "Cyan"},
                                "white": {"type": "string", "format": "color", "title": "White"}
                            }
                        },
                        "bright": {
                            "type": "object",
                            "x-order": ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"],
                            "properties": {
                                "black": {"type": "string", "format": "color", "title": "Bright Black"},
                                "red": {"type": "string", "format": "color", "title": "Bright Red"},
                                "green": {"type": "string", "format": "color", "title": "Bright Green"},
                                "yellow": {"type": "string", "format": "color", "title": "Bright Yellow"},
                                "blue": {"type": "string", "format": "color", "title": "Bright Blue"},
                                "magenta": {"type": "string", "format": "color", "title": "Bright Magenta"},
                                "cyan": {"type": "string", "format": "color", "title": "Bright Cyan"},
                                "white": {"type": "string", "format": "color", "title": "Bright White"}
                            }
                        }
                    }
                }
            }
        })
    }

    fn parse_existing_config(&self, _content: &str) -> Result<Value, String> {
        // For now, return empty - could implement TOML parsing if needed
        Ok(json!({}))
    }
}
