use gpui::*;
use gpui_component::{
    button::*, h_flex, menu::DropdownMenu, menu::PopupMenu, menu::PopupMenuItem, ActiveTheme,
    IconName, PixelsExt, Side, Sizable, TitleBar,
};

use crate::ui::menu::app_menu::SelectFont;

pub struct MainTitleBar;

impl MainTitleBar {
    pub fn new() -> Self {
        Self
    }
}

impl Render for MainTitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            .child(
                h_flex()
                    .flex_1()
                    .items_center()
                    .justify_start()
                    .child(
                        Button::new("omarchist-menu")
                            .child(
                                h_flex()
                                    .gap_1()
                                    .items_center()
                                    .child(img("logo/omarchist.png").size(px(16.)))
                                    .child("OMARCHIST")
                            )
                            .small()
                            .compact()
                            .ghost()
                            .dropdown_menu(|menu: PopupMenu, window, cx| {
                                menu.menu("About", Box::new(super::app_menu::NavigateToAbout))
                                    .menu("Settings", Box::new(super::app_menu::NavigateToSettings))
                                    .separator()
                                    .submenu("Appearance", window, cx, |submenu, _window, _cx| {
                                        submenu.menu("Light", Box::new(super::app_menu::SwitchToLight))
                                            .menu("Dark", Box::new(super::app_menu::SwitchToDark))
                                    })
                                    .separator()
                                    .menu("Quit", Box::new(super::app_menu::Quit))
                            }),
                    )
                    .child(
                        Button::new("edit-menu")
                            .label("Edit")
                            .small()
                            .compact()
                            .ghost()
                            .dropdown_menu(|menu: PopupMenu, _, _| {
                                menu.menu("Undo", Box::new(gpui_component::input::Undo))
                                    .menu("Redo", Box::new(gpui_component::input::Redo))
                                    .separator()
                                    .menu("Cut", Box::new(gpui_component::input::Cut))
                                    .menu("Copy", Box::new(gpui_component::input::Copy))
                                    .menu("Paste", Box::new(gpui_component::input::Paste))
                            }),
                    )
                    .child(
                        Button::new("themes-menu")
                            .label("Theme")
                            .small()
                            .compact()
                            .ghost()
                            .dropdown_menu(|menu: PopupMenu, _, _| {
                                menu.item(PopupMenuItem::new("Create New Theme")
                                        .on_click(|_, window, cx| {
                                            crate::ui::dialogs::create_theme_dialog::open_create_theme_dialog(window, cx);
                                        }),
                                )
                                .separator()
                                .menu("Refresh Theme", Box::new(super::app_menu::RefreshTheme))
                                .separator()
                                .item(PopupMenuItem::new("Import Theme...").disabled(true))
                                .item(PopupMenuItem::new("Export Theme...").disabled(true))
                            }),
                    ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_end()
                    .px_2()
                    .gap_2()
                    .child(
                        Button::new("settings-btn")
                            .icon(IconName::Settings2)
                            .small()
                            .ghost()
                            .dropdown_menu(|menu: PopupMenu, _window: &mut Window, cx: &mut Context<PopupMenu>| {
                                let font_size = cx.theme().font_size.as_f32() as i32;
                                menu.label("Font Size")
                                    .check_side(Side::Right)
                                    .menu_with_check("Large", font_size == 18, Box::new(SelectFont(18)))
                                    .menu_with_check("Medium", font_size == 16, Box::new(SelectFont(16)))
                                    .menu_with_check("Small", font_size == 14, Box::new(SelectFont(14)))
                            }),
                    )
                    .child(
                        Button::new("github")
                            .icon(IconName::GitHub)
                            .small()
                            .ghost()
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/tahayvr/omarchist-rs")
                            }),
                    ),
            )
    }
}

/// Handle font size selection action
pub fn handle_select_font(font_size: &SelectFont, window: &mut Window, cx: &mut App) {
    gpui_component::Theme::global_mut(cx).font_size = gpui::px(font_size.0 as f32);
    window.refresh();
}
