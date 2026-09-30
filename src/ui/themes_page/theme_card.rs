use crate::system::themes::theme_file_ops::{delete_theme, open_theme_folder};
use crate::types::themes::ThemeEntry;
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::text::selectable;
use crate::ui::theme_apply::apply_theme;
use gpui::prelude::*;
use gpui::*;
use gpui_component::{
    ActiveTheme, IconName, Sizable, WindowExt, button::*, h_flex, menu::DropdownMenu,
    menu::PopupMenuItem, v_flex,
};
use std::path::PathBuf;

pub struct ThemeCard {
    theme: ThemeEntry,
    image_height: Pixels,
    index: usize,
    is_focused: bool,
}

impl ThemeCard {
    pub fn new(theme: ThemeEntry, image_height: Pixels, index: usize) -> Self {
        Self {
            theme,
            image_height,
            index,
            is_focused: false,
        }
    }

    pub fn set_image_height(&mut self, height: Pixels) {
        self.image_height = height;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// The folder name `omarchy-theme-set` takes.
    pub fn theme_dir(&self) -> String {
        self.theme.dir.clone()
    }

    /// Opens the Theme Designer for editable themes.
    pub fn edit(&self, cx: &mut App) {
        if self.theme.origin.is_editable() {
            emit(
                cx,
                AppEvent::Navigate(ActivePage::ThemeEdit(self.theme.dir.clone())),
            );
        }
    }

    pub fn open_folder(&self) {
        let is_system = matches!(self.theme.origin, crate::types::themes::ThemeOrigin::System);
        let _ = open_theme_folder(&self.theme.dir, is_system);
    }

    /// Asks before deleting; system themes cannot be deleted.
    pub fn confirm_delete(&self, window: &mut Window, cx: &mut App) {
        if self.theme.origin.is_deletable() {
            confirm_delete_theme(&self.theme, window, cx);
        }
    }
}

fn confirm_delete_theme(theme: &ThemeEntry, window: &mut Window, cx: &mut App) {
    let dir = theme.dir.clone();
    let title = theme.title.clone();
    let is_system = matches!(theme.origin, crate::types::themes::ThemeOrigin::System);
    let message = if theme.applied {
        format!(
            "\"{title}\" is the theme Omarchy is running. Deleting it removes all of its files \
             and leaves Omarchy on a theme that no longer exists until you apply another. This \
             cannot be undone."
        )
    } else {
        format!("Delete \"{title}\" and all of its files? This cannot be undone.")
    };
    open_confirm_dialog(
        ConfirmDialog {
            title: "Delete this theme?",
            message,
            confirm_label: "Delete",
            danger: true,
        },
        move |window, cx| {
            let handle = window.window_handle();
            let dir = dir.clone();
            let title = title.clone();
            cx.spawn(async move |cx| {
                // Removing a folder of wallpapers takes long enough to stall
                // a frame.
                let result = cx
                    .background_spawn(async move { delete_theme(&dir, is_system) })
                    .await;
                handle
                    .update(cx, |_, window, cx| match result {
                        Ok(()) => {
                            window.push_notification(format!("Deleted '{title}'"), cx);
                            emit(cx, AppEvent::RefreshThemes);
                        }
                        Err(e) => {
                            window.push_notification(format!("Could not delete '{title}': {e}"), cx)
                        }
                    })
                    .ok();
            })
            .detach();
        },
        window,
        cx,
    );
}

/// A small outlined label next to the title.
fn badge(text: &'static str, color: Hsla, radius: Pixels) -> Div {
    div()
        .px_1p5()
        .py_0p5()
        .rounded(radius)
        .border_1()
        .border_color(color.opacity(0.4))
        .text_xs()
        .text_color(color)
        .child(text)
}

fn color_palette_display(colors: &crate::types::themes::ThemeColors) -> Div {
    let all_colors = [
        (&colors.primary.background, "bg"),
        (&colors.terminal.black, "black"),
        (&colors.terminal.red, "red"),
        (&colors.terminal.green, "green"),
        (&colors.terminal.yellow, "yellow"),
        (&colors.terminal.blue, "blue"),
        (&colors.terminal.magenta, "magenta"),
        (&colors.terminal.cyan, "cyan"),
        (&colors.terminal.white, "white"),
        (&colors.primary.foreground, "fg"),
    ];

    let mut row = h_flex().gap_1().items_center().justify_center();
    for (hex, _name) in &all_colors {
        if let Some(color) = hex_to_hsla(hex) {
            row = row.child(div().w(px(16.)).h(px(120.)).bg(color));
        }
    }
    row
}

impl Render for ThemeCard {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .w_full()
            .border_1()
            .border_color(if self.is_focused {
                theme.ring
            } else {
                theme.border
            })
            .rounded(theme.radius)
            .bg(if self.is_focused {
                theme.secondary
            } else {
                theme.background
            })
            .child(
                div()
                    .p_3()
                    .bg(theme.background)
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .min_w_0()
                            .flex_1()
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_color(theme.foreground)
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .child(selectable(
                                        ("theme-title", self.index),
                                        self.theme.title.clone(),
                                    )),
                            )
                            .when(self.theme.applied, |row| {
                                row.child(badge("Applied", theme.primary, theme.radius))
                            })
                            .child(badge(
                                self.theme.origin.badge_text(),
                                theme.muted_foreground,
                                theme.radius,
                            )),
                    )
                    .child({
                        let is_editable = self.theme.origin.is_editable();
                        let is_deletable = self.theme.origin.is_deletable();
                        let is_system =
                            matches!(self.theme.origin, crate::types::themes::ThemeOrigin::System);
                        let theme_dir_clone = self.theme.dir.clone();
                        let theme_entry = self.theme.clone();
                        Button::new(("menu", self.index))
                            .icon(IconName::EllipsisVertical)
                            .xsmall()
                            .ghost()
                            .cursor_pointer()
                            .dropdown_menu(move |menu, _, _cx| {
                                let theme_dir_open = theme_dir_clone.clone();
                                let theme_dir_edit = theme_dir_clone.clone();
                                let theme_to_delete = theme_entry.clone();
                                menu.item(PopupMenuItem::new("Open Folder").on_click(
                                    move |_event, _window, _cx| {
                                        let _ = open_theme_folder(&theme_dir_open, is_system);
                                    },
                                ))
                                .when(is_editable, |this| {
                                    this.item(PopupMenuItem::new("Edit Theme").on_click(
                                        move |_event, _window, cx| {
                                            emit(
                                                cx,
                                                AppEvent::Navigate(ActivePage::ThemeEdit(
                                                    theme_dir_edit.clone(),
                                                )),
                                            );
                                        },
                                    ))
                                })
                                .separator()
                                .when(is_deletable, |this| {
                                    this.item(PopupMenuItem::new("Delete Theme").on_click(
                                        move |_event, window, cx| {
                                            confirm_delete_theme(&theme_to_delete, window, cx);
                                        },
                                    ))
                                })
                            })
                    }),
            )
            .child(div().h(px(1.)).bg(theme.border))
            .child(
                // Image 16:9 aspect ratio
                div()
                    .w(self.image_height / 9.0 * 16.0)
                    .h(self.image_height)
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(theme.muted)
                    .overflow_hidden()
                    .when(!self.theme.image.is_empty(), |this| {
                        let path = PathBuf::from(&self.theme.image);
                        // Image fills the 16:9 container
                        this.child(img(path).w_full().h_full().object_fit(ObjectFit::Cover))
                    })
                    .when(self.theme.image.is_empty(), |this| {
                        this.when_some(self.theme.colors.as_ref(), |this, colors| {
                            this.child(color_palette_display(colors))
                        })
                        .when(self.theme.colors.is_none(), |this| {
                            this.child(
                                div()
                                    .text_color(theme.muted_foreground)
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .child(selectable(
                                        ("theme-title-fallback", self.index),
                                        self.theme.title.clone(),
                                    )),
                            )
                        })
                    }),
            )
            .child(div().h(px(1.)).bg(theme.border))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .bg(theme.background)
                    .child({
                        if self.theme.origin.is_editable() {
                            let theme_dir = self.theme.dir.clone();
                            Button::new(("edit", self.index))
                                .label("Edit")
                                .small()
                                .ghost()
                                .cursor_pointer()
                                .on_click(move |_event, _window, cx| {
                                    emit(
                                        cx,
                                        AppEvent::Navigate(ActivePage::ThemeEdit(
                                            theme_dir.clone(),
                                        )),
                                    );
                                })
                        } else {
                            Button::new(("empty", self.index)).label("").hidden()
                        }
                    })
                    .child({
                        let dir = self.theme.dir.clone();
                        let index = self.index;
                        Button::new(("apply", index))
                            .label("Apply")
                            .small()
                            .primary()
                            .cursor_pointer()
                            .on_click(move |_event, window, cx| {
                                apply_theme(dir.clone(), window, cx);
                            })
                    }),
            )
    }
}
