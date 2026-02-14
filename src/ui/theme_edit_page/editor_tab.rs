//! Editor tab for theme editing
//!
//! Provides UI for editing editor configurations using textareas:
//! - Neovim: edits neovim.lua file
//! - VSCode:: edits vscode.json file

use crate::system::theme_management::save_theme_data;
use crate::types::themes::EditingTheme;
use crate::ui::theme_edit_page::shared::{form_section, help_text, tab_container};
use gpui::*;
use gpui_component::{
    input::{Input, InputEvent, InputState},
    v_flex,
};
use std::fs;
use std::path::PathBuf;

/// Editor tab content for editing neovim.lua and vscode.json
pub struct EditorTab {
    theme_name: String,
    theme_data: EditingTheme,
    neovim_input: Entity<InputState>,
    vscode_input: Entity<InputState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl EditorTab {
    /// Create a new EditorTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Load file contents
        let neovim_content = Self::load_neovim_content(&theme_name);
        let vscode_content = Self::load_vscode_content(&theme_name);

        // Create input states with code editor mode
        let neovim_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("lua")
                .line_number(true)
                .searchable(true)
                .default_value(&neovim_content)
        });

        let vscode_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("json")
                .line_number(true)
                .searchable(true)
                .default_value(&vscode_content)
        });

        let tab = Self {
            theme_name,
            theme_data,
            neovim_input,
            vscode_input,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to neovim input changes
        cx.subscribe_in(
            &tab.neovim_input,
            window,
            |this, _input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event {
                    let content = this.neovim_input.read(cx).value().to_string();
                    this.save_neovim(&content, window, cx);
                }
            },
        )
        .detach();

        // Subscribe to vscode input changes
        cx.subscribe_in(
            &tab.vscode_input,
            window,
            |this, _input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event {
                    let content = this.vscode_input.read(cx).value().to_string();
                    this.save_vscode(&content, window, cx);
                }
            },
        )
        .detach();

        tab
    }

    /// Load neovim.lua content from theme folder
    fn load_neovim_content(theme_name: &str) -> String {
        let themes_dir = dirs::home_dir()
            .map(|h| h.join(".config").join("omarchy").join("themes"))
            .unwrap_or_else(|| PathBuf::from("."));

        let file_path = themes_dir.join(theme_name).join("neovim.lua");
        fs::read_to_string(&file_path).unwrap_or_else(|_| {
            // Return default content if file doesn't exist
            r#"return {
    { "tahayvr/sunset-drive.nvim", lazy = false, priority = 1000 },
    {
        "LazyVim/LazyVim",
        opts = {
            colorscheme = "sunsetdrive",
        },
    },
}"#
            .to_string()
        })
    }

    /// Load vscode.json content from theme folder
    fn load_vscode_content(theme_name: &str) -> String {
        let themes_dir = dirs::home_dir()
            .map(|h| h.join(".config").join("omarchy").join("themes"))
            .unwrap_or_else(|| PathBuf::from("."));

        let file_path = themes_dir.join(theme_name).join("vscode.json");
        fs::read_to_string(&file_path).unwrap_or_else(|_| {
            // Return default content if file doesn't exist
            r#"{
	"name": "Sunset Drive",
	"extension": "TahaYVR.sunset-drive"
}"#
            .to_string()
        })
    }

    /// Save neovim.lua content
    fn save_neovim(&mut self, content: &str, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        if self.theme_name.is_empty() {
            self.error_message = Some("Theme name cannot be empty".to_string());
            cx.notify();
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        // Save to neovim.lua file directly
        let themes_dir = dirs::home_dir()
            .map(|h| h.join(".config").join("omarchy").join("themes"))
            .unwrap_or_else(|| PathBuf::from("."));

        let file_path = themes_dir.join(&self.theme_name).join("neovim.lua");

        match fs::write(&file_path, content) {
            Ok(()) => {
                // Also update theme_data.apps.neovim with the content
                match serde_json::to_value(content) {
                    Ok(value) => {
                        self.theme_data.apps.neovim = Some(value);
                        // Save theme data to update modified_at timestamp
                        let _ = save_theme_data(&self.theme_name, &self.theme_data);
                    }
                    Err(e) => {
                        self.error_message =
                            Some(format!("Failed to serialize neovim content: {}", e));
                    }
                }
                self.is_saving = false;
            }
            Err(e) => {
                self.is_saving = false;
                self.error_message = Some(format!("Failed to write neovim.lua: {}", e));
            }
        }

        cx.notify();
    }

    /// Save vscode.json content
    fn save_vscode(&mut self, content: &str, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        if self.theme_name.is_empty() {
            self.error_message = Some("Theme name cannot be empty".to_string());
            cx.notify();
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        // Save to vscode.json file directly
        let themes_dir = dirs::home_dir()
            .map(|h| h.join(".config").join("omarchy").join("themes"))
            .unwrap_or_else(|| PathBuf::from("."));

        let file_path = themes_dir.join(&self.theme_name).join("vscode.json");

        match fs::write(&file_path, content) {
            Ok(()) => {
                // Also update theme_data.apps.vscode with the content
                match serde_json::to_value(content) {
                    Ok(value) => {
                        self.theme_data.apps.vscode = Some(value);
                        // Save theme data to update modified_at timestamp
                        let _ = save_theme_data(&self.theme_name, &self.theme_data);
                    }
                    Err(e) => {
                        self.error_message =
                            Some(format!("Failed to serialize vscode content: {}", e));
                    }
                }
                self.is_saving = false;
            }
            Err(e) => {
                self.is_saving = false;
                self.error_message = Some(format!("Failed to write vscode.json: {}", e));
            }
        }

        cx.notify();
    }

    /// Get the current theme data
    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }
}

impl Render for EditorTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text(
                "Changes auto-save. Edit the raw configuration files for Neovim and VSCode:.",
            ))
            .child(
                v_flex()
                    .gap_6()
                    .child(
                        // Neovim section
                        form_section()
                            .gap_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Neovim (neovim.lua)"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(gpui::rgb(0x888888))
                                    .child("Lua configuration for Neovim theme"),
                            )
                            .child(
                                div()
                                    .h(px(300.))
                                    .child(Input::new(&self.neovim_input).h_full()),
                            ),
                    )
                    .child(
                        // VSCode: section
                        form_section()
                            .gap_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("VSCode: (vscode.json)"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(gpui::rgb(0x888888))
                                    .child("JSON configuration for VSCode: theme"),
                            )
                            .child(
                                div()
                                    .h(px(200.))
                                    .child(Input::new(&self.vscode_input).h_full()),
                            ),
                    ),
            )
    }
}
