use crate::system::omarchy::omarchy_version::{check_omarchy_update, get_local_omarchy_version};
use crate::terminal::PENDING_TERMINAL_NAVIGATION;
use crate::ui::about_page::about_view::AboutView;
use crate::ui::config_page::config_view::ConfigView;
use crate::ui::keyboard_nav::{FocusState, FocusedSection};
use crate::ui::menu::title_bar::MainTitleBar;
use crate::ui::omarchy_page::omarchy_view::OmarchyView;
use crate::ui::settings_page::settings_view::SettingsView;
use crate::ui::system_monitor_page::system_monitor::SystemMonitorPage;
use crate::ui::terminal_page::terminal_page::TerminalPage;
use crate::ui::theme_edit_page::theme_edit::ThemeEditPage;
use crate::ui::themes_page::themes::ThemesPage;
use gpui::*;
use gpui_component::{
    Collapsible, Icon, IconName, Root, Side, h_flex,
    kbd::Kbd,
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
};
use std::cell::RefCell;

const KEY_CONTEXT: &str = "MainWindow";

thread_local! {
    pub static PENDING_TOGGLE_SIDEBAR: RefCell<bool> = const { RefCell::new(false) };
    pub static PENDING_NAVIGATE_TO_OMARCHY: RefCell<bool> = const { RefCell::new(false) };
}

/// Represents the currently active page in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivePage {
    Themes,
    ThemeEdit(String), // Holds the theme name being edited
    SystemMonitor,
    Configuration,
    Settings,
    About,
    Omarchy,
    Terminal(String), // Command being run in terminal
}

pub struct MainWindowView {
    title_bar: Entity<MainTitleBar>,
    active_page: ActivePage,
    themes_root: AnyView,
    themes_view: Entity<ThemesPage>,
    theme_edit_root: Option<AnyView>,
    theme_edit_name: Option<String>,
    system_monitor_root: AnyView,
    config_root: AnyView,
    settings_root: AnyView,
    about_root: AnyView,
    omarchy_root: AnyView,
    terminal_root: Option<AnyView>,
    terminal_command: Option<String>,
    sidebar_collapsed: bool,
    /// Keyboard navigation focus state
    focus_state: FocusState,
    /// Focus handle for the main window
    focus_handle: FocusHandle,
}

impl MainWindowView {
    pub fn new(
        title_bar: Entity<MainTitleBar>,
        initial_page: ActivePage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let themes_view = cx.new(ThemesPage::new);
        let themes_root = cx
            .new(|cx| Root::new(themes_view.clone(), window, cx))
            .into();

        let system_monitor_view = cx.new(|cx| SystemMonitorPage::new(window, cx));
        let system_monitor_root = cx
            .new(|cx| Root::new(system_monitor_view, window, cx))
            .into();

        let config_view = cx.new(ConfigView::new);
        let config_root = cx.new(|cx| Root::new(config_view, window, cx)).into();

        let settings_view = cx.new(|_| SettingsView);
        let settings_root = cx.new(|cx| Root::new(settings_view, window, cx)).into();

        let about_view = cx.new(|_| AboutView);
        let about_root = cx.new(|cx| Root::new(about_view, window, cx)).into();

        // Get Omarchy version once (silently fail if unavailable)
        let omarchy_version = get_local_omarchy_version()
            .ok()
            .filter(|v| v != "unknown" && !v.is_empty());

        // Pass version to OmarchyView to avoid redundant git calls
        let omarchy_view = cx.new(|cx| OmarchyView::new(omarchy_version.clone(), cx));
        let omarchy_root = cx.new(|cx| Root::new(omarchy_view, window, cx)).into();

        // Spawn async task to check for Omarchy updates
        let version_for_check = omarchy_version.clone().unwrap_or_default();
        let title_bar_for_update = title_bar.clone();
        cx.spawn(async move |_this, cx| {
            if !version_for_check.is_empty() {
                match check_omarchy_update(&version_for_check).await {
                    Ok(update_available) => {
                        title_bar_for_update
                            .update(cx, |title_bar, _cx| {
                                title_bar.set_omarchy_update_available(update_available);
                            })
                            .ok();
                    }
                    Err(e) => {
                        eprintln!("Failed to check for Omarchy updates: {e}");
                    }
                }
            }
        })
        .detach();

        // Create focus handle for keyboard navigation
        let focus_handle = cx.focus_handle();

        let mut view = Self {
            title_bar,
            active_page: ActivePage::Themes,
            themes_root,
            themes_view,
            theme_edit_root: None,
            theme_edit_name: None,
            system_monitor_root,
            config_root,
            settings_root,
            about_root,
            omarchy_root,
            terminal_root: None,
            terminal_command: None,
            sidebar_collapsed: true,
            focus_state: FocusState::with_section(FocusedSection::Sidebar),
            focus_handle,
        };

        // Navigate to the initial page if it's not the default Themes page
        if initial_page != ActivePage::Themes {
            view.navigate_to(initial_page, window, cx);
        }

        view
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

        // Handle Terminal page creation/update
        if let ActivePage::Terminal(ref command) = page {
            let should_create = self.terminal_command.as_ref() != Some(command);

            if should_create {
                let terminal_view = cx.new(|cx| TerminalPage::new(command.clone(), window, cx));
                self.terminal_root = Some(cx.new(|cx| Root::new(terminal_view, window, cx)).into());
                self.terminal_command = Some(command.clone());
            }
        }

        self.active_page = page;
        cx.notify();
    }

    /// Navigate to terminal page with a specific command
    pub fn navigate_to_terminal(
        &mut self,
        command: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate_to(ActivePage::Terminal(command), window, cx);
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
            ActivePage::Configuration => self.config_root.clone(),
            ActivePage::Settings => self.settings_root.clone(),
            ActivePage::About => self.about_root.clone(),
            ActivePage::Omarchy => self.omarchy_root.clone(),
            ActivePage::Terminal(_) => self
                .terminal_root
                .clone()
                .unwrap_or(self.themes_root.clone()),
        }
    }

    fn is_page_active(&self, page: ActivePage) -> bool {
        match (&self.active_page, &page) {
            (ActivePage::Themes, ActivePage::Themes) => true,
            (ActivePage::ThemeEdit(a), ActivePage::ThemeEdit(b)) => a == b,
            (ActivePage::SystemMonitor, ActivePage::SystemMonitor) => true,
            (ActivePage::Configuration, ActivePage::Configuration) => true,
            (ActivePage::Settings, ActivePage::Settings) => true,
            (ActivePage::About, ActivePage::About) => true,
            (ActivePage::Omarchy, ActivePage::Omarchy) => true,
            (ActivePage::Terminal(a), ActivePage::Terminal(b)) => a == b,
            _ => false,
        }
    }

    /// Get the active page based on sidebar index
    fn page_from_sidebar_index(&self, index: usize) -> ActivePage {
        match index {
            0 => ActivePage::Themes,
            1 => ActivePage::SystemMonitor,
            2 => ActivePage::Configuration,
            _ => ActivePage::Themes,
        }
    }

    /// Navigate to the currently focused sidebar item
    fn activate_focused_sidebar_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let page = self.page_from_sidebar_index(self.focus_state.sidebar_index);
        self.navigate_to(page, window, cx);
    }

    /// Check if sidebar item at index is focused
    fn is_sidebar_item_focused(&self, index: usize) -> bool {
        self.focus_state.focused_section == FocusedSection::Sidebar
            && self.focus_state.sidebar_index == index
    }

    /// Handle NextFocus action (Tab)
    fn handle_next_focus(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.focus_state.next_section();
        cx.notify();
    }

    /// Handle PrevFocus action (Shift+Tab)
    fn handle_prev_focus(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.focus_state.prev_section();
        cx.notify();
    }

    /// Handle NextItem action (Down arrow)
    fn handle_next_item(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        match self.focus_state.focused_section {
            FocusedSection::Sidebar => {
                self.focus_state.next_sidebar_item();
                cx.notify();
            }
            FocusedSection::Content => {
                if self.active_page == ActivePage::Themes {
                    self.themes_view.update(cx, |page, cx| {
                        page.handle_next_item(cx);
                    });
                }
            }
        }
    }

    /// Handle PrevItem action (Up arrow)
    fn handle_prev_item(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        match self.focus_state.focused_section {
            FocusedSection::Sidebar => {
                self.focus_state.prev_sidebar_item();
                cx.notify();
            }
            FocusedSection::Content => {
                if self.active_page == ActivePage::Themes {
                    self.themes_view.update(cx, |page, cx| {
                        page.handle_prev_item(cx);
                    });
                }
            }
        }
    }

    /// Handle SelectNext action (Right arrow)
    fn handle_select_next(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        match self.focus_state.focused_section {
            FocusedSection::Sidebar => {
                // Move to content
                self.focus_state.focused_section = FocusedSection::Content;
                cx.notify();
            }
            FocusedSection::Content => {
                if self.active_page == ActivePage::Themes {
                    self.themes_view.update(cx, |page, cx| {
                        page.handle_select_next(cx);
                    });
                }
            }
        }
    }

    /// Handle SelectPrev action (Left arrow)
    fn handle_select_prev(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        match self.focus_state.focused_section {
            FocusedSection::Sidebar => {
                // Already at sidebar, do nothing
            }
            FocusedSection::Content => {
                if self.active_page == ActivePage::Themes {
                    self.themes_view.update(cx, |page, cx| {
                        page.handle_select_prev(cx);
                    });
                }
            }
        }
    }

    /// Handle ActivateItem action (Enter/Space)
    fn handle_activate_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.focus_state.focused_section {
            FocusedSection::Sidebar => {
                self.activate_focused_sidebar_item(window, cx);
            }
            FocusedSection::Content => {
                if self.active_page == ActivePage::Themes {
                    self.themes_view.update(cx, |page, cx| {
                        page.handle_activate(cx);
                    });
                }
            }
        }
    }

    /// Handle EscapeFocus action
    fn handle_escape_focus(&mut self, cx: &mut Context<Self>) {
        self.focus_state.focused_section = FocusedSection::Sidebar;
        cx.notify();
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

        // Check for pending sidebar toggle
        let pending_toggle = PENDING_TOGGLE_SIDEBAR.with(|flag| {
            let value = *flag.borrow();
            if value {
                *flag.borrow_mut() = false;
            }
            value
        });
        if pending_toggle {
            self.sidebar_collapsed = !self.sidebar_collapsed;
            self.themes_view.update(cx, |themes_page, cx| {
                themes_page.set_sidebar_collapsed(self.sidebar_collapsed, cx);
            });
            cx.notify();
        }

        // Check for pending terminal navigation
        let pending_terminal = PENDING_TERMINAL_NAVIGATION.with(|nav| nav.borrow_mut().take());
        if let Some(command) = pending_terminal {
            self.navigate_to_terminal(command, window, cx);
        }

        // Check for pending Omarchy navigation from title bar
        let pending_omarchy = PENDING_NAVIGATE_TO_OMARCHY.with(|flag| {
            let value = *flag.borrow();
            if value {
                *flag.borrow_mut() = false;
            }
            value
        });
        if pending_omarchy {
            self.navigate_to(ActivePage::Omarchy, window, cx);
        }

        // Responsive sidebar: auto-collapse on small windows (< 768px)
        let viewport_width = window.viewport_size().width;
        let is_small_window = viewport_width < px(768.0);
        let sidebar_should_be_collapsed = is_small_window || self.sidebar_collapsed;

        // Update themes_page with collapsed state and global focus state
        let content_has_focus = self.focus_state.focused_section == FocusedSection::Content;
        self.themes_view.update(cx, |themes_page, cx| {
            themes_page.set_sidebar_collapsed(sidebar_should_be_collapsed, cx);
            themes_page.set_global_focus(content_has_focus, cx);
        });

        // Check if this view has focus
        let _has_focus = self.focus_handle.is_focused(window);

        div()
            .id("main-window-root")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
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
                |this, _: &crate::ui::menu::app_menu::NavigateToOmarchy, window, cx| {
                    this.navigate_to(ActivePage::Omarchy, window, cx);
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
            // Keyboard navigation actions
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NextFocus, window, cx| {
                    this.handle_next_focus(window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::PrevFocus, window, cx| {
                    this.handle_prev_focus(window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NextItem, window, cx| {
                    this.handle_next_item(window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::PrevItem, window, cx| {
                    this.handle_prev_item(window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::SelectNext, window, cx| {
                    this.handle_select_next(window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::SelectPrev, window, cx| {
                    this.handle_select_prev(window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::ActivateItem, window, cx| {
                    this.handle_activate_item(window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::EscapeFocus, _window, cx| {
                    this.handle_escape_focus(cx);
                },
            ))
            .child(self.title_bar.clone())
            .child(
                h_flex()
                    .flex_1()
                    .size_full()
                    .overflow_hidden()
                    .child(
                        Sidebar::new(Side::Left)
                            .collapsed(sidebar_should_be_collapsed)
                            .child(
                                SidebarGroup::new("Navigation").child(
                                    SidebarMenu::new()
                                        .cursor_pointer()
                                        .child(
                                            SidebarMenuItem::new("THEMES")
                                                .icon(Icon::new(IconName::LayoutDashboard))
                                                .active(
                                                    self.is_page_active(ActivePage::Themes)
                                                        || self.is_sidebar_item_focused(0),
                                                )
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
                                                    self.is_page_active(ActivePage::SystemMonitor)
                                                        || self.is_sidebar_item_focused(1),
                                                )
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.navigate_to(
                                                        ActivePage::SystemMonitor,
                                                        window,
                                                        cx,
                                                    );
                                                })),
                                        )
                                        .child(
                                            SidebarMenuItem::new("CONFIGURATION")
                                                .icon(Icon::new(IconName::Settings))
                                                .active(
                                                    self.is_page_active(ActivePage::Configuration)
                                                        || self.is_sidebar_item_focused(2),
                                                )
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.navigate_to(
                                                        ActivePage::Configuration,
                                                        window,
                                                        cx,
                                                    );
                                                })),
                                        ),
                                ),
                            )
                            .footer(
                                SidebarGroup::new("")
                                    .collapsed(sidebar_should_be_collapsed)
                                    .child(
                                        SidebarMenu::new().cursor_pointer().child(
                                            SidebarMenuItem::new("Toggle Sidebar")
                                                .icon(Icon::new(IconName::PanelLeft))
                                                .suffix(Kbd::new(
                                                    Keystroke::parse("ctrl-b").unwrap(),
                                                ))
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.sidebar_collapsed =
                                                        !this.sidebar_collapsed;
                                                    // Update themes page with new sidebar state
                                                    this.themes_view.update(
                                                        cx,
                                                        |themes_page, cx| {
                                                            themes_page.set_sidebar_collapsed(
                                                                this.sidebar_collapsed,
                                                                cx,
                                                            );
                                                        },
                                                    );
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
