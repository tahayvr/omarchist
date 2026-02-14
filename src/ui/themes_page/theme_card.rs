use crate::shell::theme_sh_commands::apply_theme;
use crate::system::theme_file_ops::{delete_theme, open_theme_folder};
use crate::types::themes::SysTheme;
use gpui::prelude::*;
use gpui::*;
use gpui_component::{
    ActiveTheme, IconName, Sizable, button::*, h_flex, menu::DropdownMenu, menu::PopupMenuItem,
    v_flex,
};
use smol;
use std::path::PathBuf;

pub struct ThemeCard {
    theme: SysTheme,
    image_height: Pixels,
    index: usize,
}

impl ThemeCard {
    pub fn new(theme: SysTheme, image_height: Pixels, index: usize) -> Self {
        Self {
            theme,
            image_height,
            index,
        }
    }

    pub fn set_image_height(&mut self, height: Pixels) {
        self.image_height = height;
    }
}

/// Parse hex color string (#RRGGBB) to Hsla
fn parse_hex_color(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(rgb(u32::from_be_bytes([0, r, g, b])).into())
}

/// color palette display with theme colors
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
        if let Some(color) = parse_hex_color(hex) {
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
            .border_color(theme.border)
            .rounded(theme.radius)
            .bg(theme.background)
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
                        let is_custom = self.theme.is_custom;
                        let theme_dir_clone = self.theme.dir.clone();
                        let is_system = self.theme.is_system;
                        Button::new(("menu", self.index))
                            .icon(IconName::EllipsisVertical)
                            .xsmall()
                            .ghost()
                            .dropdown_menu(move |menu, _, _cx| {
                                let theme_dir_open = theme_dir_clone.clone();
                                let theme_dir_edit = theme_dir_clone.clone();
                                let theme_dir_delete = theme_dir_clone.clone();
                                menu.item(
                                    PopupMenuItem::new("Open Folder")
                                        .on_click(move |_event, _window, _cx| {
                                            // Open theme folder in Nautilus
                                            let _ = open_theme_folder(&theme_dir_open, is_system);
                                        }),
                                )
                                .when(is_custom, |this| {
                                    this.item(
                                        PopupMenuItem::new("Edit Theme")
                                            .on_click(move |_event, _window, cx| {
                                                // Store theme name for navigation
                                                crate::ui::dialogs::create_theme_dialog::PENDING_THEME_NAVIGATION.with(|nav| {
                                                    *nav.borrow_mut() = Some(theme_dir_edit.clone());
                                                });
                                                cx.refresh_windows();
                                            }),
                                    )
                                })
                                .separator()
                                .when(is_custom, |this| {
                                    this.item(
                                        PopupMenuItem::new("Delete Theme")
                                            .on_click(move |_event, _window, cx| {
                                                // Delete the theme folder
                                                match delete_theme(&theme_dir_delete, is_system) {
                                                    Ok(()) => {
                                                        // Trigger themes refresh
                                                        crate::ui::dialogs::create_theme_dialog::PENDING_REFRESH_THEMES.with(|flag| {
                                                            *flag.borrow_mut() = true;
                                                        });
                                                        cx.refresh_windows();
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to delete theme: {}", e);
                                                    }
                                                }
                                            }),
                                    )
                                })
                            })
                    }),
            )
            .child(div().h(px(1.)).bg(theme.border))
            .child(
                // Image / preview container - 16:9 aspect ratio
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
                        // Display a vertical color palette preview
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
                        if self.theme.is_custom {
                            let theme_dir = self.theme.dir.clone();
                            Button::new(("edit", self.index))
                                .label("Edit")
                                .small()
                                .ghost()
                                .on_click(move |_event, _window, cx| {
                                    // Store theme name for navigation
                                    crate::ui::dialogs::create_theme_dialog::PENDING_THEME_NAVIGATION.with(|nav| {
                                        *nav.borrow_mut() = Some(theme_dir.clone());
                                    });
                                    cx.refresh_windows();
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
                            .on_click(move |_event, _window, _cx| {
                                eprintln!("=== Apply BUTTON CLICKED ===");
                                eprintln!("Theme: {}", dir);
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
