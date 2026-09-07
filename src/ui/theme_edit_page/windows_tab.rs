use crate::types::themes::EditingTheme;
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::theme_edit_page::shared::{help_text, tab_container};
use gpui::*;
use gpui_component::{ActiveTheme, h_flex, label::Label, v_flex};

pub struct WindowsTab {
    theme_data: EditingTheme,
}

impl WindowsTab {
    pub fn new(
        _theme_name: String,
        theme_data: EditingTheme,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self { theme_data }
    }

    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    fn swatch(color_hex: &str, label: &'static str, cx: &App) -> impl IntoElement {
        let color = hex_to_hsla(color_hex).unwrap_or(gpui::rgb(0x33A1FF).into());

        v_flex()
            .gap_2()
            .items_center()
            .child(
                div()
                    .size_16()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(color),
            )
            .child(
                Label::new(label)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground),
            )
    }
}

impl Render for WindowsTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text(
                "Hyprland window border colors are generated from this theme's Accent color \
                 (set on the General tab) — Omarchy no longer stores them separately per theme.",
                cx.theme().muted_foreground,
            ))
            .child(h_flex().gap_24().flex_wrap().child(Self::swatch(
                &self.theme_data.colors.accent,
                "Accent",
                cx,
            )))
    }
}
