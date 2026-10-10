use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, Side, Sizable, TitleBar,
    button::*,
    h_flex,
    menu::{DropdownMenu, PopupMenu},
};

use crate::system::flows::icon_path;
use crate::system::flows::store::load_flows;
use crate::ui::menu::app_menu::{self, SelectFont};
use crate::ui::omarchy_page::updates::OmarchyUpdates;

pub struct MainTitleBar {
    updates: Entity<OmarchyUpdates>,
    _updates_observer: Subscription,
}

impl MainTitleBar {
    /// Owns the Omarchy update state so the badge and the Omarchy page read
    /// the same result. Nothing is checked until `OmarchyUpdates::start_periodic`
    /// runs, which `main.rs` does once the window exists.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let updates = cx.new(|_| OmarchyUpdates::new());
        let observer = cx.observe(&updates, |_, _, cx| cx.notify());
        Self {
            updates,
            _updates_observer: observer,
        }
    }

    pub fn updates(&self) -> &Entity<OmarchyUpdates> {
        &self.updates
    }
}

impl Render for MainTitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // A tab group with a later index than the sidebar and page (0), so
        // Tab reaches the title-bar menus last instead of first.
        div().tab_index(1).tab_group().child(
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
                            .cursor_pointer()
                            .dropdown_menu(|menu: PopupMenu, _window, _cx| {
                                menu.menu("Command palette", Box::new(crate::ui::focus::ShowCommands))
                                    .menu("Keyboard shortcuts", Box::new(crate::ui::focus::ShowShortcuts))
                                    .separator()
                                    .menu("Settings", Box::new(super::app_menu::NavigateToSettings))
                                    .menu("About", Box::new(super::app_menu::NavigateToAbout))
                                    .separator()
                                    .menu("Quit", Box::new(super::app_menu::Quit))
                            }),
                    )
                    .child(
                        Button::new("themes-menu")
                            .label("Themes")
                            .small()
                            .compact()
                            .ghost()
                            .cursor_pointer()
                            .dropdown_menu(|menu: PopupMenu, _, _| {
                                menu.menu("New theme", Box::new(app_menu::NewTheme))
                                    .separator()
                                    .menu("Re-apply theme", Box::new(super::app_menu::RefreshTheme))
                            }),
                    )
                    .child(
                        Button::new("keybinds-menu")
                            .label("Keybinds")
                            .small()
                            .compact()
                            .ghost()
                            .cursor_pointer()
                            .dropdown_menu(|menu: PopupMenu, _, _| {
                                menu.menu("Add keybind…", Box::new(app_menu::NewKeybind))
                            }),
                    )
                    .child(
                        Button::new("flows-menu")
                            .label("Flows")
                            .small()
                            .compact()
                            .ghost()
                            .cursor_pointer()
                            .dropdown_menu(|menu: PopupMenu, window, cx| {
                                // Read on every open so the list matches the flows folder.
                                let flows = load_flows().unwrap_or_default();
                                let menu = menu.menu("New flow", Box::new(app_menu::NewFlow))
                                    .menu("New from template", Box::new(app_menu::NewFlowFromTemplate))
                                    .menu("Catalog", Box::new(app_menu::OpenCatalog))
                                    .menu("Import flow…", Box::new(app_menu::ImportFlow));
                                // Nothing to run means no Run submenu, not a
                                // disabled line in one.
                                if flows.is_empty() {
                                    return menu;
                                }
                                menu.separator()
                                    .submenu("Run", window, cx, move |menu, _, _| {
                                        flows.iter().fold(menu, |menu, flow| {
                                            menu.menu_with_icon(
                                                crate::ui::text::title_case(&flow.name),
                                                Icon::empty().path(icon_path(&flow.icon)),
                                                Box::new(app_menu::RunFlow(flow.id.clone())),
                                            )
                                        })
                                    })
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
                        div()
                            .relative()
                            .child(
                                Button::new("omarchy-btn")
                                    .icon(Icon::empty().path("logo/omarchy-icon.svg"))
                                    .small()
                                    .ghost()
                                    .cursor_pointer()
                                    .on_click(|_, _, cx| {
                                        emit(cx, AppEvent::Navigate(ActivePage::Omarchy));
                                    }),
                            )
                            .when(self.updates.read(cx).available(), |this| {
                                this.child(
                                    div()
                                        .absolute()
                                        .top_0()
                                        .right_0()
                                        .size_2()
                                        .rounded_full()
                                        .bg(cx.theme().red)
                                )
                            })
                    )
                    .child(
                        Button::new("settings-btn")
                            .icon(IconName::Settings2)
                            .small()
                            .ghost()
                            .cursor_pointer()
                            .dropdown_menu(|menu: PopupMenu, _window: &mut Window, cx: &mut Context<PopupMenu>| {
                                let font_size = f32::from(cx.theme().font_size) as i32;
                                let look = crate::system::config::config_setup::settings().theme_mode;
                                let follows = look != "light" && look != "dark";
                                menu.label("Font size")
                                    .check_side(Side::Right)
                                    .menu_with_check("Large", font_size == 18, Box::new(SelectFont(18)))
                                    .menu_with_check("Medium", font_size == 16, Box::new(SelectFont(16)))
                                    .menu_with_check("Small", font_size == 14, Box::new(SelectFont(14)))
                                    .separator()
                                    .label("Appearance")
                                    .check_side(Side::Right)
                                    .menu_with_check("Omarchy", follows, Box::new(super::app_menu::FollowOmarchy))
                                    .menu_with_check("Light", look == "light", Box::new(super::app_menu::SwitchToLight))
                                    .menu_with_check("Dark", look == "dark", Box::new(super::app_menu::SwitchToDark))
                            }),
                    )
                    .child(
                        Button::new("github")
                            .icon(Icon::new(Icon::empty()).path("icons/github.svg"))
                            .small()
                            .ghost()
                            .cursor_pointer()
                            .tooltip("Star the repo")
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/tahayvr/omarchist")
                            }),
                    ),
            ),
        )
    }
}

pub fn handle_select_font(font_size: &SelectFont, window: &mut Window, cx: &mut App) {
    gpui_component::Theme::global_mut(cx).font_size = gpui::px(font_size.0 as f32);
    window.refresh();
}
