use crate::ui::about_page::about_view::AboutView;
use crate::ui::menu::title_bar::MainTitleBar;
use crate::ui::settings_page::settings_view::SettingsView;
use crate::ui::system_monitor_page::system_monitor::SystemMonitorPage;
use crate::ui::theme_edit_page::theme_edit::ThemeEditPage;
use crate::ui::themes_page::themes::ThemesPage;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    Collapsible, Icon, IconName, Root, h_flex,
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};

/// Represents the currently active page in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivePage {
    Themes,
    ThemeEdit(String), // Holds the theme name being edited
    SystemMonitor,
    Settings,
    About,
}

pub struct MainWindowView {
    title_bar: Entity<MainTitleBar>,
    active_page: ActivePage,
    themes_root: AnyView,
    themes_view: Entity<ThemesPage>,
    theme_edit_root: Option<AnyView>,
    theme_edit_name: Option<String>,
    system_monitor_root: AnyView,
    settings_root: AnyView,
    about_root: AnyView,
    sidebar_collapsed: bool,
}

impl MainWindowView {
    pub fn new(
        title_bar: Entity<MainTitleBar>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let themes_view = cx.new(|cx| ThemesPage::new(cx));
        let themes_root = cx
            .new(|cx| Root::new(themes_view.clone(), window, cx))
            .into();

        let system_monitor_view = cx.new(|cx| SystemMonitorPage::new(window, cx));
        let system_monitor_root = cx
            .new(|cx| Root::new(system_monitor_view, window, cx))
            .into();

        let settings_view = cx.new(|_| SettingsView);
        let settings_root = cx.new(|cx| Root::new(settings_view, window, cx)).into();

        let about_view = cx.new(|_| AboutView);
        let about_root = cx.new(|cx| Root::new(about_view, window, cx)).into();

        Self {
            title_bar,
            active_page: ActivePage::Themes,
            themes_root,
            themes_view,
            theme_edit_root: None,
            theme_edit_name: None,
            system_monitor_root,
            settings_root,
            about_root,
            sidebar_collapsed: false,
        }
    }

    pub fn navigate_to(&mut self, page: ActivePage, window: &mut Window, cx: &mut Context<Self>) {
        if self.active_page == page {
            return;
        }

        // Handle ThemeEdit page creation/update
        if let ActivePage::ThemeEdit(ref theme_name) = page {
            // Check if we need to create or update the theme edit view
            let should_create = self.theme_edit_name.as_ref() != Some(theme_name);

            if should_create {
                let theme_edit_view =
                    cx.new(|cx| ThemeEditPage::new(theme_name.clone(), window, cx));
                self.theme_edit_root =
                    Some(cx.new(|cx| Root::new(theme_edit_view, window, cx)).into());
                self.theme_edit_name = Some(theme_name.clone());
            }
        }

        self.active_page = page;
        cx.notify();
    }

    pub fn navigate_to_theme_edit(
        &mut self,
        theme_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        eprintln!("navigate_to_theme_edit called with: {}", theme_name);
        self.navigate_to(ActivePage::ThemeEdit(theme_name), window, cx);
    }

    fn current_page_view(&self) -> AnyView {
        match &self.active_page {
            ActivePage::Themes => self.themes_root.clone(),
            ActivePage::ThemeEdit(_) => self
                .theme_edit_root
                .clone()
                .unwrap_or(self.themes_root.clone()),
            ActivePage::SystemMonitor => self.system_monitor_root.clone(),
            ActivePage::Settings => self.settings_root.clone(),
            ActivePage::About => self.about_root.clone(),
        }
    }

    fn is_page_active(&self, page: ActivePage) -> bool {
        match (&self.active_page, &page) {
            (ActivePage::Themes, ActivePage::Themes) => true,
            (ActivePage::ThemeEdit(a), ActivePage::ThemeEdit(b)) => a == b,
            (ActivePage::SystemMonitor, ActivePage::SystemMonitor) => true,
            (ActivePage::Settings, ActivePage::Settings) => true,
            (ActivePage::About, ActivePage::About) => true,
            _ => false,
        }
    }
}

impl Render for MainWindowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check for pending theme navigation
        let pending_theme = crate::ui::dialogs::create_theme_dialog::PENDING_THEME_NAVIGATION
            .with(|nav| nav.borrow_mut().take());
        if let Some(theme_name) = pending_theme {
            self.navigate_to_theme_edit(theme_name, window, cx);
        }

        let pending_navigate = crate::ui::theme_edit_page::theme_edit::PENDING_NAVIGATE_TO_THEMES
            .with(|flag| {
                let value = *flag.borrow();
                if value {
                    *flag.borrow_mut() = false;
                }
                value
            });
        if pending_navigate {
            self.navigate_to(ActivePage::Themes, window, cx);
        }

        let pending_refresh =
            crate::ui::dialogs::create_theme_dialog::PENDING_REFRESH_THEMES.with(|flag| {
                let value = *flag.borrow();
                if value {
                    *flag.borrow_mut() = false;
                }
                value
            });
        if pending_refresh {
            // Refresh the themes list
            self.themes_view.update(cx, |themes_page, cx| {
                themes_page.refresh_themes(cx);
            });
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToSettings, window, cx| {
                    this.navigate_to(ActivePage::Settings, window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToAbout, window, cx| {
                    this.navigate_to(ActivePage::About, window, cx);
                },
            ))
            .on_action(cx.listener(
                |_, _: &crate::ui::menu::app_menu::RefreshTheme, _window, cx| {
                    cx.spawn(async move |_this, _cx| {
                        if let Err(e) = crate::shell::theme_sh_commands::refresh_theme() {
                            eprintln!("Failed to refresh theme: {e}");
                        }
                    })
                    .detach();
                },
            ))
            .child(self.title_bar.clone())
            .child(
                h_flex()
                    .flex_1()
                    .size_full()
                    .overflow_hidden()
                    .child(
                        Sidebar::left()
                            .collapsed(self.sidebar_collapsed)
                            .header(
                                SidebarHeader::new()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size_4()
                                            .flex_shrink_0()
                                            .child(Icon::empty().path("icons/layout-grid.svg")),
                                    )
                                    .when(!self.sidebar_collapsed, |this| {
                                        this.child(
                                            h_flex()
                                                .flex_1()
                                                .text_sm()
                                                .line_height(relative(1.25))
                                                .overflow_hidden()
                                                .text_ellipsis()
                                                .child("Dashboard"),
                                        )
                                    }),
                            )
                            .child(
                                SidebarGroup::new("Navigation").child(
                                    SidebarMenu::new()
                                        .child(
                                            SidebarMenuItem::new("THEMES")
                                                .icon(Icon::new(IconName::LayoutDashboard))
                                                .active(self.is_page_active(ActivePage::Themes))
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.navigate_to(
                                                        ActivePage::Themes,
                                                        window,
                                                        cx,
                                                    );
                                                })),
                                        )
                                        .child(
                                            SidebarMenuItem::new("SYSTEM MONITOR")
                                                .icon(Icon::new(IconName::ChartPie))
                                                .active(
                                                    self.is_page_active(ActivePage::SystemMonitor),
                                                )
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.navigate_to(
                                                        ActivePage::SystemMonitor,
                                                        window,
                                                        cx,
                                                    );
                                                })),
                                        ),
                                ),
                            )
                            .footer(
                                SidebarGroup::new("")
                                    .collapsed(self.sidebar_collapsed)
                                    .child(
                                        SidebarMenu::new()
                                            .child(
                                                SidebarMenuItem::new("Omarchy")
                                                    .icon(
                                                        Icon::empty().path("logo/omarchy-icon.svg"),
                                                    )
                                                    .on_click(cx.listener(|_, _, _, _| {
                                                        // No-op for now
                                                    })),
                                            )
                                            .child(
                                                SidebarMenuItem::new("Toggle Sidebar")
                                                    .icon(Icon::new(IconName::PanelLeft))
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.sidebar_collapsed = !this.sidebar_collapsed;
                                                        // Update themes page with new sidebar state
                                                        this.themes_view.update(cx, |themes_page, cx| {
                                                            themes_page.set_sidebar_collapsed(this.sidebar_collapsed, cx);
                                                        });
                                                        cx.notify();
                                                    })),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .size_full()
                            .overflow_hidden()
                            .p_4()
                            .child(self.current_page_view()),
                    ),
            )
            .children(Root::render_dialog_layer(window, cx))
    }
}
