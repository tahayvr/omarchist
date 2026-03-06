use gpui::*;
use gpui_component::{ActiveTheme, h_flex, label::Label, switch::Switch, v_flex};

use crate::system::config::config_setup::{read_settings, save_settings};

pub struct SettingsView {
    auto_apply_theme: bool,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsView {
    pub fn new() -> Self {
        let auto_apply_theme = read_settings()
            .map(|s| s.settings.auto_apply_theme)
            .unwrap_or(false);

        Self { auto_apply_theme }
    }

    fn toggle_auto_apply_theme(
        &mut self,
        checked: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.auto_apply_theme = checked;

        match save_auto_apply_theme(checked) {
            Ok(()) => {
                eprintln!("Saved auto_apply_theme: {}", checked);
            }
            Err(e) => {
                eprintln!("Failed to save auto_apply_theme: {}", e);
            }
        }

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

        v_flex()
            .size_full()
            .p_6()
            .gap_6()
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
                        // auto_apply_theme row
                        h_flex()
                            .gap_3()
                            .items_center()
                            .justify_between()
                            .p_4()
                            .rounded(theme.radius)
                            .border_1()
                            .border_color(theme.border)
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
