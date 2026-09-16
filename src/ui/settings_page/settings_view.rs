use gpui::*;
use gpui_component::{ActiveTheme, h_flex, label::Label, v_flex};

use crate::system::config::config_setup::{read_settings, save_settings};
use crate::ui::focus::FocusableSwitch;

const KEY_CONTEXT: &str = "SettingsPage";

pub struct SettingsView {
    auto_apply_theme: bool,
    pub focus_handle: FocusHandle,
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
        }
    }

    /// Focuses the first control on the page.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        crate::ui::focus::focus_first_in(&self.focus_handle, window, cx);
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
}

fn save_auto_apply_theme(value: bool) -> crate::error::Result<()> {
    let mut settings = read_settings()?;
    settings.settings.auto_apply_theme = value;
    save_settings(&settings)
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let auto_apply_theme = self.auto_apply_theme;

        v_flex()
            .id("settings-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
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
                                FocusableSwitch::new("auto-apply-theme")
                                    .checked(auto_apply_theme)
                                    .on_change(cx.listener(|this, checked, window, cx| {
                                        this.toggle_auto_apply_theme(*checked, window, cx);
                                    })),
                            ),
                    ),
            )
    }
}
