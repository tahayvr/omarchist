use crate::system::omarchy_paths::user_themes_dir;
use crate::system::themes::theme_management::update_theme;
use crate::types::themes::EditingTheme;
use crate::ui::theme_edit_page::shared::{
    error_message, focus_section, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    ActiveTheme,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use std::fs;
use std::path::PathBuf;

const NEOVIM_FILE: &str = "neovim.lua";
const VSCODE_FILE: &str = "vscode.json";

// Optional per-theme override files. Quattro generates both a Neovim
// colorscheme (`neovim.lua`) and a VS Code theme (`vscode-theme.json`) from
// colors.toml via `omarchy-theme-set-templates` whenever the theme folder
// doesn't ship its own, so these files only exist when the user wants to
// point at a specific plugin/extension instead. An empty editor removes the
// file so Omarchy's generated version takes over again.
pub struct EditorTab {
    theme_name: String,
    theme_data: EditingTheme,
    neovim_input: Entity<InputState>,
    vscode_input: Entity<InputState>,
    is_saving: bool,
    error_message: Option<String>,
    scroll: ScrollHandle,
}

impl EditorTab {
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        scroll: &ScrollHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let neovim_content = Self::load_override(&theme_name, NEOVIM_FILE);
        let vscode_content = Self::load_override(&theme_name, VSCODE_FILE);

        let neovim_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("lua")
                .line_number(false)
                .placeholder(
                    "Leave empty to use the Neovim colorscheme Omarchy generates from colors.toml",
                )
                .default_value(&neovim_content)
        });

        let vscode_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("json")
                .line_number(false)
                .placeholder(
                    "Leave empty to use the VS Code theme Omarchy generates from colors.toml",
                )
                .default_value(&vscode_content)
        });

        let tab = Self {
            theme_name,
            theme_data,
            neovim_input,
            vscode_input,
            is_saving: false,
            error_message: None,
            scroll: scroll.clone(),
        };

        cx.subscribe_in(
            &tab.neovim_input,
            window,
            |this, _input_state, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    let content = this.neovim_input.read(cx).value().to_string();
                    this.save_override(NEOVIM_FILE, &content, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &tab.vscode_input,
            window,
            |this, _input_state, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    let content = this.vscode_input.read(cx).value().to_string();
                    this.save_override(VSCODE_FILE, &content, cx);
                }
            },
        )
        .detach();

        tab
    }

    fn override_path(theme_name: &str, file_name: &str) -> Option<PathBuf> {
        user_themes_dir().map(|dir| dir.join(theme_name).join(file_name))
    }

    // Missing file means "no override" and shows as an empty editor.
    fn load_override(theme_name: &str, file_name: &str) -> String {
        Self::override_path(theme_name, file_name)
            .and_then(|path| fs::read_to_string(path).ok())
            .unwrap_or_default()
    }

    fn save_override(&mut self, file_name: &str, content: &str, cx: &mut Context<Self>) {
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

        let result = Self::write_override(&self.theme_name, file_name, content);

        match result {
            Ok(()) => {
                let value = if content.trim().is_empty() {
                    None
                } else {
                    Some(serde_json::Value::String(content.to_string()))
                };
                match file_name {
                    NEOVIM_FILE => self.theme_data.apps.neovim = value.clone(),
                    VSCODE_FILE => self.theme_data.apps.vscode = value.clone(),
                    _ => {}
                }
                // Persist the manifest so modified_at reflects this edit.
                let result = update_theme(&self.theme_name, |theme| match file_name {
                    NEOVIM_FILE => theme.apps.neovim = value,
                    VSCODE_FILE => theme.apps.vscode = value,
                    _ => {}
                });
                if let Err(e) = result {
                    self.error_message = Some(e.to_string());
                }
            }
            Err(e) => self.error_message = Some(e.to_string()),
        }

        self.is_saving = false;
        cx.notify();
    }

    // Writes the override, or removes it when the editor is blank so Omarchy
    // falls back to its template-generated file on the next theme apply.
    fn write_override(
        theme_name: &str,
        file_name: &str,
        content: &str,
    ) -> crate::error::Result<()> {
        let path = Self::override_path(theme_name, file_name)
            .ok_or(crate::error::Error::UnknownDirectory("themes"))?;

        if content.trim().is_empty() {
            if path.exists() {
                fs::remove_file(&path).map_err(|e| {
                    crate::error::Error::io(format!("Failed to remove {file_name}"), e)
                })?;
            }
            return Ok(());
        }

        fs::write(&path, content)
            .map_err(|e| crate::error::Error::io(format!("Failed to write {file_name}"), e))
    }

    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }
}

impl Render for EditorTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text(
                "Optional overrides. Omarchy already generates a Neovim colorscheme and a VS Code \
                 theme from this theme's colors — only fill these in to use a specific plugin or \
                 extension instead. Clearing a field removes the override.",
                cx.theme().muted_foreground,
            ))
            .children(
                self.error_message
                    .as_ref()
                    .map(|msg| error_message(msg.clone(), cx)),
            )
            .child(
                v_flex()
                    .gap_6()
                    .child(focus_section(
                        "editor-neovim",
                        &self.scroll,
                        form_section()
                            .gap_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Neovim (neovim.lua)"),
                            )
                            .child(
                                div().bg(cx.theme().background).h(px(300.)).child(
                                    Input::new(&self.neovim_input)
                                        .bg(cx.theme().background)
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .h_full()
                                        .appearance(false),
                                ),
                            ),
                    ))
                    .child(focus_section(
                        "editor-vscode",
                        &self.scroll,
                        form_section()
                            .gap_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("VS Code (vscode.json)"),
                            )
                            .child(
                                div().bg(cx.theme().background).h(px(200.)).child(
                                    Input::new(&self.vscode_input)
                                        .bg(cx.theme().background)
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .h_full()
                                        .appearance(false),
                                ),
                            ),
                    )),
            )
    }
}
