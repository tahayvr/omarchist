use crate::system::themes::overrides;
use crate::ui::theme_edit_page::shared::{
    error_message, focus_section, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    ActiveTheme,
    input::{Editor, EditorState, InputEvent},
    v_flex,
};

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
    neovim_input: Entity<EditorState>,
    vscode_input: Entity<EditorState>,
    error_message: Option<String>,
    scroll: ScrollHandle,
}

impl EditorTab {
    pub fn new(
        theme_name: String,
        scroll: &ScrollHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let neovim_content = Self::load_override(&theme_name, NEOVIM_FILE);
        let vscode_content = Self::load_override(&theme_name, VSCODE_FILE);

        let neovim_input = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("lua")
                .line_number(false)
                .placeholder(
                    "Leave empty to use the Neovim colorscheme Omarchy generates from colors.toml",
                )
                .default_value(&neovim_content)
        });

        let vscode_input = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("json")
                .line_number(false)
                .placeholder(
                    "Leave empty to use the VS Code theme Omarchy generates from colors.toml",
                )
                .default_value(&vscode_content)
        });

        let tab = Self {
            theme_name,
            neovim_input,
            vscode_input,
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

    // Missing file means "no override" and shows as an empty editor.
    fn load_override(theme_name: &str, file_name: &str) -> String {
        overrides::find(file_name)
            .and_then(|spec| overrides::read(theme_name, spec).ok().flatten())
            .unwrap_or_default()
    }

    // A blank editor removes the override so Omarchy falls back to its
    // template-generated file on the next theme apply.
    fn save_override(&mut self, file_name: &str, content: &str, cx: &mut Context<Self>) {
        let Some(spec) = overrides::find(file_name) else {
            return;
        };
        let result = if content.trim().is_empty() {
            overrides::remove(&self.theme_name, spec)
        } else {
            overrides::write(&self.theme_name, spec, content)
        };
        self.error_message = result.err().map(|e| e.to_string());
        cx.notify();
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
                                    Editor::new(&self.neovim_input)
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
                                    Editor::new(&self.vscode_input)
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
