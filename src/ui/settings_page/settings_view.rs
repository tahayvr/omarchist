use gpui::*;
use gpui_component::{ActiveTheme, h_flex, label::Label, switch::Switch, v_flex};

use crate::system::config::config_setup::{read_settings, save_settings};
use crate::ui::menu::app_menu;

const KEY_CONTEXT: &str = "SettingsPage";
/// Number of keyboard-navigable settings rows (currently just one).
const SETTINGS_ITEM_COUNT: usize = 1;

pub struct SettingsView {
    auto_apply_theme: bool,
    pub focus_handle: FocusHandle,
    /// Which settings row currently has keyboard focus (`None` = none).
    focused_index: Option<usize>,
}

impl SettingsView {
    /// Constructor intended to be passed directly to `cx.new(...)`.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let auto_apply_theme = read_settings()
            .map(|s| s.settings.auto_apply_theme)
            .unwrap_or(false);

        Self {
            auto_apply_theme,
            focus_handle: cx.focus_handle(),
            focused_index: None,
        }
    }

    fn toggle_auto_apply_theme(
        &mut self,
        checked: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.auto_apply_theme = checked;

        if let Err(e) = save_auto_apply_theme(checked) {
            eprintln!("Failed to save auto_apply_theme: {}", e);
        }

        cx.notify();
    }

    fn handle_next_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_index = Some(match self.focused_index {
            None => 0,
            Some(i) => (i + 1).min(SETTINGS_ITEM_COUNT - 1),
        });
        cx.notify();
    }

    fn handle_prev_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_index = Some(match self.focused_index {
            None | Some(0) => 0,
            Some(i) => i - 1,
        });
        cx.notify();
    }

    fn handle_activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(0) = self.focused_index {
            let new_val = !self.auto_apply_theme;
            self.toggle_auto_apply_theme(new_val, window, cx);
        }
    }

    fn handle_escape(&mut self, cx: &mut Context<Self>) {
        self.focused_index = None;
        cx.notify();
    }
}

fn save_auto_apply_theme(value: bool) -> Result<(), String> {
    let mut settings = read_settings()?;
    settings.settings.auto_apply_theme = value;
    save_settings(&settings)
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let auto_apply_theme = self.auto_apply_theme;
        let row_focused = self.focused_index == Some(0);

        v_flex()
            .id("settings-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .p_6()
            .gap_6()
            .on_action(cx.listener(|this, _: &app_menu::NextFocus, _window, cx| {
                this.handle_next_focus(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::PrevFocus, _window, cx| {
                this.handle_prev_focus(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::ActivateItem, window, cx| {
                this.handle_activate(window, cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::EscapeFocus, _window, cx| {
                this.handle_escape(cx);
            }))
            .child(
                // Page header
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.foreground)
                            .child("Settings"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("Configure application preferences"),
                    ),
            )
            .child(
                // Settings section: Themes
                v_flex()
                    .gap_4()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.muted_foreground)
                            .child("Themes"),
                    )
                    .child(
                        // auto_apply_theme row — keyboard focus ring when focused_index == 0
                        h_flex()
                            .gap_3()
                            .items_center()
                            .justify_between()
                            .p_4()
                            .rounded(theme.radius)
                            .border_1()
                            .border_color(if row_focused {
                                theme.ring
                            } else {
                                theme.border
                            })
                            .child(
                                v_flex()
                                    .gap_1()
                                    .flex_1()
                                    .child(
                                        Label::new("Auto-apply theme on edit")
                                            .font_weight(FontWeight::MEDIUM),
                                    )
                                    .child(
                                        div().text_sm().text_color(theme.muted_foreground).child(
                                            "Automatically apply a theme when you open its editor",
                                        ),
                                    ),
                            )
                            .child(
                                Switch::new("auto-apply-theme")
                                    .checked(auto_apply_theme)
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, checked, window, cx| {
                                        this.toggle_auto_apply_theme(*checked, window, cx);
                                    })),
                            ),
                    ),
            )
    }
}
