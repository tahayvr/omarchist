use gpui::*;
use gpui_component::{
    IconName, Sizable, TitleBar, button::*, h_flex, menu::DropdownMenu, menu::PopupMenu,
    menu::PopupMenuItem,
};

// For image-based logo
use gpui::img;

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
                                menu.item(
                                    PopupMenuItem::new("Create New Theme")
                                        .on_click(|_, window, cx| {
                                            crate::ui::dialogs::create_theme_dialog::open_create_theme_dialog(window, cx);
                                        }),
                                )
                                .separator()
                                .item(PopupMenuItem::new("Refresh Theme"))
                                .separator()
                                .item(PopupMenuItem::new("Import Theme..."))
                                .item(PopupMenuItem::new("Export Theme..."))
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
