use crate::shell::theme_sh_commands::apply_theme;
use crate::system::themes::theme_file_ops::{delete_theme, open_theme_folder};
use crate::types::themes::ThemeEntry;
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::menu::app_menu::{NavigateToThemeEdit, RefreshThemes};
use gpui::prelude::*;
use gpui::*;
use gpui_component::{
    ActiveTheme, IconName, Sizable, button::*, h_flex, menu::DropdownMenu, menu::PopupMenuItem,
    v_flex,
};
use smol;
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

    pub fn activate(&mut self) {
        let dir = self.theme.dir.clone();
        smol::spawn(async move {
            if let Err(e) = apply_theme(dir).await {
                eprintln!("Failed to apply theme: {}", e);
            }
        })
        .detach();
    }
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
                        div()
                            .text_color(theme.foreground)
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .child(self.theme.title.clone()),
                    )
                    .child({
                        let is_editable = self.theme.origin.is_editable();
                        let is_deletable = self.theme.origin.is_deletable();
                        let is_system =
                            matches!(self.theme.origin, crate::types::themes::ThemeOrigin::System);
                        let theme_dir_clone = self.theme.dir.clone();
                        Button::new(("menu", self.index))
                            .icon(IconName::EllipsisVertical)
                            .xsmall()
                            .ghost()
                            .cursor_pointer()
                            .dropdown_menu(move |menu, _, _cx| {
                                let theme_dir_open = theme_dir_clone.clone();
                                let theme_dir_edit = theme_dir_clone.clone();
                                let theme_dir_delete = theme_dir_clone.clone();
                                menu.item(PopupMenuItem::new("Open Folder").on_click(
                                    move |_event, _window, _cx| {
                                        let _ = open_theme_folder(&theme_dir_open, is_system);
                                    },
                                ))
                                .when(is_editable, |this| {
                                    this.item(PopupMenuItem::new("Edit Theme").on_click(
                                        move |_event, _window, cx| {
                                            let action =
                                                NavigateToThemeEdit(theme_dir_edit.clone());
                                            cx.dispatch_action(&action);
                                        },
                                    ))
                                })
                                .separator()
                                .when(is_deletable, |this| {
                                    this.item(PopupMenuItem::new("Delete Theme").on_click(
                                        move |_event, _window, cx| match delete_theme(
                                            &theme_dir_delete,
                                            is_system,
                                        ) {
                                            Ok(()) => {
                                                cx.dispatch_action(&RefreshThemes);
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to delete theme: {}", e);
                                            }
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
                                    .child(self.theme.title.clone()),
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
                                    let action = NavigateToThemeEdit(theme_dir.clone());
                                    cx.dispatch_action(&action);
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
                            .on_click(move |_event, _window, _cx| {
                                let dir_clone = dir.clone();
                                smol::spawn(async move {
                                    if let Err(e) = apply_theme(dir_clone).await {
                                        eprintln!("Failed: {}", e);
                                    }
                                })
                                .detach();
                            })
                    }),
            )
    }
}
