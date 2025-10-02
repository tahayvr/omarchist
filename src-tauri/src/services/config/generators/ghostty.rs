use super::ConfigGenerator;
use serde_json::{json, Value};

pub struct GhosttyGenerator;

unsafe impl Send for GhosttyGenerator {}
unsafe impl Sync for GhosttyGenerator {}

impl ConfigGenerator for GhosttyGenerator {
    fn get_app_name(&self) -> &'static str {
        "ghostty"
    }

    fn get_file_name(&self) -> &'static str {
        "ghostty.conf"
    }

    fn generate_config(&self, theme_data: &Value) -> Result<String, String> {
        let empty_obj = json!({});
        let ghostty = theme_data.get("ghostty").unwrap_or(&empty_obj);

        // Primary colors
        let primary_bg = ghostty
            .get("colors")
            .and_then(|c| c.get("primary"))
            .and_then(|p| p.get("background"))
            .and_then(|b| b.as_str())
            .unwrap_or("#121212");
        let primary_fg = ghostty
            .get("colors")
            .and_then(|c| c.get("primary"))
            .and_then(|p| p.get("foreground"))
            .and_then(|f| f.as_str())
            .unwrap_or("#bebebe");

        // Cursor colors
        let empty_cursor = json!({});
        let cursor_obj = ghostty
            .get("colors")
            .and_then(|c| c.get("cursor"))
            .unwrap_or(&empty_cursor);
        let cursor_color = cursor_obj
            .get("cursor")
            .and_then(|v| v.as_str())
            .unwrap_or("#eaeaea");
        let cursor_text = cursor_obj
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("#121212");

        // Selection colors
        let empty_selection = json!({});
        let selection_obj = ghostty
            .get("colors")
            .and_then(|c| c.get("selection"))
            .unwrap_or(&empty_selection);
        let selection_background = selection_obj
            .get("background")
            .and_then(|v| v.as_str())
            .unwrap_or("#333333");
        let selection_foreground = selection_obj
            .get("foreground")
            .and_then(|v| v.as_str())
            .unwrap_or("#eaeaea");

        // Normal colors
        let empty_normal = json!({});
        let normal = ghostty
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
        let bright = ghostty
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
            .unwrap_or("#ffffff");

        Ok(format!(
            r#"# ────────────────────────────────────────────────────────────
# Custom Theme for Ghostty
# Made by Omarchist
# ────────────────────────────────────────────────────────────

background = {primary_bg}
foreground = {primary_fg}

cursor-color = {cursor_color}
cursor-text = {cursor_text}

selection-background = {selection_background}
selection-foreground = {selection_foreground}

# normal colors
palette = 0={normal_black}
palette = 1={normal_red}
palette = 2={normal_green}
palette = 3={normal_yellow}
palette = 4={normal_blue}
palette = 5={normal_magenta}
palette = 6={normal_cyan}
palette = 7={normal_white}

# bright colors
palette = 8={bright_black}
palette = 9={bright_red}
palette = 10={bright_green}
palette = 11={bright_yellow}
palette = 12={bright_blue}
palette = 13={bright_magenta}
palette = 14={bright_cyan}
palette = 15={bright_white}
"#
        ))
    }

    fn get_config_schema(&self) -> Value {
        json!({
            "type": "object",
            "x-order": ["colors"],
            "properties": {
                "colors": {
                    "type": "object",
                    "x-order": [
                        "primary",
                        "cursor",
                        "selection",
                        "normal",
                        "bright"
                    ],
                    "properties": {
                        "primary": {
                            "type": "object",
                            "x-order": ["background", "foreground"],
                            "properties": {
                                "background": {"type": "string", "format": "color", "title": "Background", "description": "Background Color", "default": "#121212"},
                                "foreground": {"type": "string", "format": "color", "title": "Foreground", "description": "Foreground Color", "default": "#BEBEBE"}
                            }
                        },
                        "cursor": {
                            "type": "object",
                            "properties": {
                                "cursor": {"type": "string", "format": "color", "title": "Cursor Color", "default": "#EAEAEA"},
                                "text": {"type": "string", "format": "color", "title": "Cursor Text", "default": "#121212"}
                            }
                        },
                        "selection": {
                            "type": "object",
                            "properties": {
                                "background": {"type": "string", "format": "color", "title": "Selection Background", "default": "#333333", "description": "Background color for selected text"},
                                "foreground": {"type": "string", "format": "color", "title": "Selection Foreground", "default": "#EAEAEA", "description": "Foreground color for selected text"}
                            }
                        },
                        "normal": {
                            "type": "object",
                            "x-order": ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"],
                            "properties": {
                                "black": {"type": "string", "format": "color", "title": "Black", "default": "#333333"   },
                                "red": {"type": "string", "format": "color", "title": "Red", "default": "#D35F5F"},
                                "green": {"type": "string", "format": "color", "title": "Green", "default": "#FFC107"},
                                "yellow": {"type": "string", "format": "color", "title": "Yellow", "default": "#b91c1c"},
                                "blue": {"type": "string", "format": "color", "title": "Blue", "default": "#e68e0d"},
                                "magenta": {"type": "string", "format": "color", "title": "Magenta", "default": "#D35F5F"},
                                "cyan": {"type": "string", "format": "color", "title": "Cyan", "default": "#bebebe"},
                                "white": {"type": "string", "format": "color", "title": "White", "default": "#bebebe"}
                            }
                        },
                        "bright": {
                            "type": "object",
                            "x-order": ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"],
                            "properties": {
                                "black": {"type": "string", "format": "color", "title": "Bright Black", "default": "#8a8a8d"},
                                "red": {"type": "string", "format": "color", "title": "Bright Red", "default": "#b91c1c"},
                                "green": {"type": "string", "format": "color", "title": "Bright Green", "default": "#FFC107"},
                                "yellow": {"type": "string", "format": "color", "title": "Bright Yellow", "default": "#b90a0a"},
                                "blue": {"type": "string", "format": "color", "title": "Bright Blue", "default": "#f59e0b"},
                                "magenta": {"type": "string", "format": "color", "title": "Bright Magenta", "default": "#b91c1c"},
                                "cyan": {"type": "string", "format": "color", "title": "Bright Cyan", "default": "#eaeaea"},
                                "white": {"type": "string", "format": "color", "title": "Bright White", "default": "#FFFFFF"}
                            }
                        }
                    }
                }
            }
        })
    }

    fn parse_existing_config(&self, _content: &str) -> Result<Value, String> {
        // For now, return empty - could implement config parsing if needed
        Ok(json!({}))
    }
}
