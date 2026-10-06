use crate::ui::app_events::{AppEvent, emit};
use gpui::*;
use gpui_component::{h_flex, radio::Radio};

use crate::system::themes::icons::{self, ICON_THEMES};
use crate::ui::theme_edit_page::shared::{
    field_grid, section_title, tab_container, tab_grid_columns,
};

/// The Yaru icon color, one radio per variant, written to `icons.theme` as
/// soon as it is picked.
pub struct IconsTab {
    theme_name: String,
    /// What `icons.theme` holds, if the theme ships one.
    selected: Option<String>,
}

impl IconsTab {
    pub fn new(theme_name: String, cx: &mut Context<Self>) -> Self {
        let mut tab = Self {
            theme_name,
            selected: None,
        };
        tab.refresh(cx);
        tab
    }

    /// Re-reads `icons.theme`; called when the tab is shown.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        match icons::read(&self.theme_name) {
            Ok(selected) => {
                self.selected = selected;
            }
            Err(e) => emit(cx, AppEvent::Error(e.to_string())),
        }
        cx.notify();
    }

    fn select(&mut self, icon_theme: &'static str, cx: &mut Context<Self>) {
        match icons::write(&self.theme_name, icon_theme) {
            Ok(()) => {
                self.selected = Some(icon_theme.to_string());
            }
            Err(e) => emit(
                cx,
                AppEvent::Error(format!("Could not save the icon color: {e}")),
            ),
        }
        cx.notify();
    }
}

impl Render for IconsTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cells = ICON_THEMES
            .iter()
            .map(|icon_theme| {
                let (r, g, b) = icon_theme.color;
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Radio::new(icon_theme.name)
                            .label(icon_theme.label)
                            .checked(self.selected.as_deref() == Some(icon_theme.name))
                            .on_click(cx.listener(move |this, _: &bool, _, cx| {
                                this.select(icon_theme.name, cx);
                            })),
                    )
                    .child(
                        div()
                            .size_5()
                            .rounded_sm()
                            .bg(rgb((r as u32) << 16 | (g as u32) << 8 | b as u32)),
                    )
                    .into_any_element()
            })
            .collect();

        tab_container()
            .child(section_title("Icon color"))
            .child(field_grid(tab_grid_columns(window).min(4), cells))
    }
}
