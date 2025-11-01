use super::ConfigGenerator;
use serde_json::{json, Value};

/// Unified terminal generator that provides a single schema for all terminal emulators
/// The actual config generation is delegated to individual generators
pub struct TerminalGenerator;

unsafe impl Send for TerminalGenerator {}
unsafe impl Sync for TerminalGenerator {}

impl ConfigGenerator for TerminalGenerator {
    fn get_app_name(&self) -> &'static str {
        "terminal"
    }

    fn get_file_name(&self) -> &'static str {
        "terminal.json" // Not actually used, just for trait compliance
    }

    fn generate_config(&self, _theme_data: &Value) -> Result<String, String> {
        // This generator doesn't produce a config file itself
        // Individual terminal generators handle that
        Ok(String::new())
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
                                "background": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Background",
                                    "description": "Background Color",
                                    "default": "#121212"
                                },
                                "foreground": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Foreground",
                                    "description": "Foreground Color",
                                    "default": "#bebebe"
                                }
                            }
                        },
                        "cursor": {
                            "type": "object",
                            "properties": {
                                "cursor": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Cursor Color",
                                    "default": "#EAEAEA"
                                },
                                "text": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Cursor Text",
                                    "default": "#121212"
                                }
                            }
                        },
                        "selection": {
                            "type": "object",
                            "properties": {
                                "background": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Selection Background",
                                    "default": "#333333",
                                    "description": "Background color for selected text"
                                },
                                "foreground": {
                                    "type": "string",
                                    "format": "color",
                                    "title": "Selection Foreground",
                                    "default": "#eaeaea",
                                    "description": "Foreground color for selected text"
                                }
                            }
                        },
                        "normal": {
                            "type": "object",
                            "x-order": ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"],
                            "properties": {
                                "black": {"type": "string", "format": "color", "title": "Black", "default": "#333333"},
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
        Ok(json!({}))
    }
}
