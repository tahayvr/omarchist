use crate::system::omarchy::omarchy_version::{check_omarchy_update, get_local_omarchy_version};
use crate::system::omarchy::startup::PERIODIC_CHECK_INTERVAL_SECS;
use crate::ui::about_page::about_view::AboutView;
use crate::ui::config_page::config_view::ConfigView;
use crate::ui::keyboard_nav::{FocusState, FocusedSection};
use crate::ui::menu::app_menu::{
    NavigateToThemeEdit, RefreshThemes, WaybarProfileCreated, WaybarProfileManaged,
};
use crate::ui::menu::title_bar::MainTitleBar;
use crate::ui::omarchy_page::omarchy_view::OmarchyView;
use crate::ui::settings_page::settings_view::SettingsView;
use crate::ui::status_bar_page::status_bar_view::StatusBarView;
use crate::ui::system_monitor_page::system_monitor::SystemMonitorPage;
use crate::ui::theme_edit_page::theme_edit::ThemeEditPage;
use crate::ui::themes_page::themes::ThemesPage;
use gpui::*;
use gpui_component::{
    Collapsible, Icon, IconName, Root, Side, h_flex,
    kbd::Kbd,
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
};

const KEY_CONTEXT: &str = "MainWindow";

const SIDEBAR_ITEM_COUNT: usize = 3;
#[derive(PartialEq, Clone)]
pub enum ActivePage {
    Themes,
    ThemeEdit(String), // Holds the theme name being edited
    SystemMonitor,
    Configuration,
    Settings,
    StatusBar,
    About,
    Omarchy,
}

pub struct MainWindowView {
    title_bar: Entity<MainTitleBar>,
    active_page: ActivePage,
    // Default page — always present
    themes_root: AnyView,
    themes_view: Entity<ThemesPage>,
    // ThemeEdit is created on first navigation to a given theme
    theme_edit_root: Option<AnyView>,
    theme_edit_view: Option<Entity<ThemeEditPage>>,
    theme_edit_name: Option<String>,
    // All other pages are created lazily on first navigation
    system_monitor_root: Option<AnyView>,
    config_root: Option<AnyView>,
    config_view: Option<Entity<ConfigView>>,
    settings_root: Option<AnyView>,
    status_bar_root: Option<AnyView>,
    status_bar_view: Option<Entity<StatusBarView>>,
    about_root: Option<AnyView>,
    about_view: Option<Entity<AboutView>>,
    omarchy_root: Option<AnyView>,
    omarchy_view: Option<Entity<OmarchyView>>,
    sidebar_collapsed: bool,
    focus_state: FocusState,
    focus_handle: FocusHandle,
}

impl MainWindowView {
    pub fn new(
        title_bar: Entity<MainTitleBar>,
        initial_page: ActivePage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // The Themes page is the default landing page — created eagerly.
        let themes_view = cx.new(ThemesPage::new);
        let themes_root = cx
            .new(|cx| Root::new(themes_view.clone(), window, cx))
            .into();

        // Spawn a background task that checks for Omarchy updates at startup
        // and repeats every PERIODIC_CHECK_INTERVAL_SECS.  This keeps the
        // title-bar badge current without requiring the user to open the
        // Omarchy page.
        {
            let title_bar_watcher = title_bar.clone();
            cx.spawn(async move |_this, cx| {
                // Initial check
                let version = get_local_omarchy_version().unwrap_or_else(|_| "unknown".to_string());
                if let Ok(update_available) = check_omarchy_update(&version).await {
                    title_bar_watcher
                        .update(cx, |tb, _| {
                            tb.set_omarchy_update_available(update_available);
                        })
                        .ok();
                }

                // Periodic re-checks
                loop {
                    smol::Timer::after(std::time::Duration::from_secs(
                        PERIODIC_CHECK_INTERVAL_SECS,
                    ))
                    .await;

                    let version =
                        get_local_omarchy_version().unwrap_or_else(|_| "unknown".to_string());
                    if let Ok(update_available) = check_omarchy_update(&version).await {
                        title_bar_watcher
                            .update(cx, |tb, _| {
                                tb.set_omarchy_update_available(update_available);
                            })
                            .ok();
                    }
                }
            })
            .detach();
        }

        // Create focus handle for sidebar navigation; keep it focused at start
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window);

        let initial_sidebar_index = match &initial_page {
            ActivePage::Themes | ActivePage::ThemeEdit(_) => 0,
            ActivePage::Configuration => 1,
            ActivePage::StatusBar => 2,
            _ => 0,
        };

        let mut view = Self {
            title_bar,
            active_page: ActivePage::Themes,
            themes_root,
            themes_view,
            theme_edit_root: None,
            theme_edit_view: None,
            theme_edit_name: None,
            system_monitor_root: None,
            config_root: None,
            config_view: None,
            settings_root: None,
            status_bar_root: None,
            status_bar_view: None,
            about_root: None,
            about_view: None,
            omarchy_root: None,
            omarchy_view: None,
            sidebar_collapsed: true,
            focus_state: FocusState {
                focused_section: FocusedSection::Sidebar,
                sidebar_index: initial_sidebar_index,
                sidebar_count: SIDEBAR_ITEM_COUNT,
            },
            focus_handle,
        };

        // Navigate to the initial page if it's not the default Themes page
        if initial_page != ActivePage::Themes {
            view.navigate_to(initial_page, window, cx);
        }

        view
    }

    /// Ensures the view and root for `page` have been created.  Called at the
    /// start of every `navigate_to` so that render always sees a valid root.
    fn ensure_page_created(
        &mut self,
        page: &ActivePage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match page {
            ActivePage::ThemeEdit(theme_name) => {
                if self.theme_edit_name.as_deref() != Some(theme_name.as_str()) {
                    let theme_edit_view =
                        cx.new(|cx| ThemeEditPage::new(theme_name.clone(), window, cx));
                    self.theme_edit_root = Some(
                        cx.new(|cx| Root::new(theme_edit_view.clone(), window, cx))
                            .into(),
                    );
                    self.theme_edit_view = Some(theme_edit_view);
                    self.theme_edit_name = Some(theme_name.clone());
                }
            }
            ActivePage::SystemMonitor => {
                if self.system_monitor_root.is_none() {
                    let view = cx.new(|cx| SystemMonitorPage::new(window, cx));
                    self.system_monitor_root =
                        Some(cx.new(|cx| Root::new(view, window, cx)).into());
                }
            }
            ActivePage::Configuration => {
                if self.config_root.is_none() {
                    let config_view = cx.new(|cx| ConfigView::new(window, cx));
                    self.config_root = Some(
                        cx.new(|cx| Root::new(config_view.clone(), window, cx))
                            .into(),
                    );
                    self.config_view = Some(config_view);
                }
            }
            ActivePage::Settings => {
                if self.settings_root.is_none() {
                    let settings_view = cx.new(|_| SettingsView::new());
                    self.settings_root =
                        Some(cx.new(|cx| Root::new(settings_view, window, cx)).into());
                }
            }
            ActivePage::StatusBar => {
                if self.status_bar_root.is_none() {
                    let status_bar_view = cx.new(|cx| StatusBarView::new(window, cx));
                    self.status_bar_root = Some(
                        cx.new(|cx| Root::new(status_bar_view.clone(), window, cx))
                            .into(),
                    );
                    self.status_bar_view = Some(status_bar_view);
                }
            }
            ActivePage::About => {
                if self.about_root.is_none() {
                    let about_view = cx.new(AboutView::new);
                    self.about_root = Some(
                        cx.new(|cx| Root::new(about_view.clone(), window, cx))
                            .into(),
                    );
                    self.about_view = Some(about_view);
                }
            }
            ActivePage::Omarchy => {
                if self.omarchy_root.is_none() {
                    // Read local version once when the page is first opened.
                    let local_version = get_local_omarchy_version()
                        .ok()
                        .filter(|v| v != "unknown" && !v.is_empty());
                    let omarchy_view =
                        cx.new(|cx| OmarchyView::new(local_version, self.title_bar.clone(), cx));
                    self.omarchy_root = Some(
                        cx.new(|cx| Root::new(omarchy_view.clone(), window, cx))
                            .into(),
                    );
                    self.omarchy_view = Some(omarchy_view);
                }
            }
            // Themes is always present.
            ActivePage::Themes => {}
        }
    }

    pub fn navigate_to(&mut self, page: ActivePage, window: &mut Window, cx: &mut Context<Self>) {
        if self.active_page == page {
            return;
        }

        // Create the page entity on first visit.
        self.ensure_page_created(&page, window, cx);

        // ThemeEdit-specific: auto-apply theme when editing starts.
        if let ActivePage::ThemeEdit(ref theme_name) = page {
            let auto_apply = crate::system::config::config_setup::read_settings()
                .map(|s| s.settings.auto_apply_theme)
                .unwrap_or(false);

            if auto_apply {
                let dir = theme_name.clone();
                cx.spawn(async move |_this, _cx| {
                    if let Err(e) = crate::shell::theme_sh_commands::apply_theme(dir).await {
                        eprintln!("auto_apply_theme failed: {}", e);
                    }
                })
                .detach();
            }
        }

        self.active_page = page;

        // Transfer GPUI focus to the newly active page so its key_context
        // and on_action handlers are in the dispatch chain.
        self.transfer_focus_to_active_page(window, cx);

        cx.notify();
    }

    fn transfer_focus_to_active_page(&self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = match &self.active_page {
            ActivePage::Themes => Some(self.themes_view.read(cx).focus_handle.clone()),
            ActivePage::ThemeEdit(_) => self
                .theme_edit_view
                .as_ref()
                .map(|v| v.read(cx).focus_handle.clone()),
            ActivePage::StatusBar => self
                .status_bar_view
                .as_ref()
                .map(|v| v.read(cx).focus_handle.clone()),
            ActivePage::About => self
                .about_view
                .as_ref()
                .map(|v| v.read(cx).focus_handle.clone()),
            ActivePage::Omarchy => self
                .omarchy_view
                .as_ref()
                .map(|v| v.read(cx).focus_handle.clone()),
            ActivePage::Configuration => self
                .config_view
                .as_ref()
                .map(|v| v.read(cx).focus_handle.clone()),
            // Settings and SystemMonitor have no custom focus handle — keep main window focus
            ActivePage::Settings | ActivePage::SystemMonitor => None,
        };

        if let Some(fh) = handle {
            fh.focus(window);
        } else {
            // Return focus to the main window (sidebar)
            self.focus_handle.focus(window);
        }
    }

    pub fn navigate_to_theme_edit(
        &mut self,
        theme_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate_to(ActivePage::ThemeEdit(theme_name), window, cx);
    }

    fn current_page_view(&self) -> AnyView {
        match &self.active_page {
            ActivePage::Themes => self.themes_root.clone(),
            ActivePage::ThemeEdit(_) => self
                .theme_edit_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::SystemMonitor => self
                .system_monitor_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::Configuration => self
                .config_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::Settings => self
                .settings_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::StatusBar => self
                .status_bar_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::About => self
                .about_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::Omarchy => self
                .omarchy_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
        }
    }

    fn is_page_active(&self, page: ActivePage) -> bool {
        match (&self.active_page, &page) {
            (ActivePage::Themes, ActivePage::Themes) => true,
            (ActivePage::ThemeEdit(_), ActivePage::Themes) => true, // ThemeEdit is under Themes in sidebar
            (ActivePage::ThemeEdit(a), ActivePage::ThemeEdit(b)) => a == b,
            (ActivePage::SystemMonitor, ActivePage::SystemMonitor) => true,
            (ActivePage::Configuration, ActivePage::Configuration) => true,
            (ActivePage::Settings, ActivePage::Settings) => true,
            (ActivePage::StatusBar, ActivePage::StatusBar) => true,
            (ActivePage::About, ActivePage::About) => true,
            (ActivePage::Omarchy, ActivePage::Omarchy) => true,
            _ => false,
        }
    }

    fn page_from_sidebar_index(&self, index: usize) -> ActivePage {
        match index {
            0 => ActivePage::Themes,
            1 => ActivePage::Configuration,
            2 => ActivePage::StatusBar,
            _ => ActivePage::Themes,
        }
    }

    fn activate_focused_sidebar_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let page = self.page_from_sidebar_index(self.focus_state.sidebar_index);
        self.focus_state.focused_section = FocusedSection::Content;
        self.navigate_to(page, window, cx);
    }

    fn is_sidebar_item_focused(&self, index: usize) -> bool {
        self.focus_state.focused_section == FocusedSection::Sidebar
            && self.focus_state.sidebar_index == index
    }

    fn handle_next_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.focus_state.focused_section {
            FocusedSection::Sidebar => {
                self.focus_state.focused_section = FocusedSection::Content;
                // Transfer GPUI focus to the active page
                self.transfer_focus_to_active_page(window, cx);
            }
            FocusedSection::Content => {
                self.focus_state.focused_section = FocusedSection::Sidebar;
                // Return GPUI focus to the main window for sidebar navigation
                self.focus_handle.focus(window);
            }
        }
        cx.notify();
    }

    fn handle_prev_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.focus_state.focused_section {
            FocusedSection::Sidebar => {
                self.focus_state.focused_section = FocusedSection::Content;
                // Transfer GPUI focus to the active page
                self.transfer_focus_to_active_page(window, cx);
            }
            FocusedSection::Content => {
                self.focus_state.focused_section = FocusedSection::Sidebar;
                // Return GPUI focus to the main window for sidebar navigation
                self.focus_handle.focus(window);
            }
        }
        cx.notify();
    }

    fn handle_next_item(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_state.focused_section == FocusedSection::Sidebar {
            self.focus_state.next_sidebar_item();
            cx.notify();
        }
        // Content navigation is handled by the child page views directly
    }

    fn handle_prev_item(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_state.focused_section == FocusedSection::Sidebar {
            self.focus_state.prev_sidebar_item();
            cx.notify();
        }
        // Content navigation is handled by the child page views directly
    }

    fn handle_select_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_state.focused_section == FocusedSection::Sidebar {
            self.activate_focused_sidebar_item(window, cx);
        }
        // Content navigation is handled by the child page views directly
    }

    fn handle_select_prev(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_state.focused_section == FocusedSection::Content {
            // Child views consume this if they still have internal items to navigate left.
            // If they bubble it up (e.g. ThemesPage at Tabs level), we move to sidebar.
            self.focus_state.focused_section = FocusedSection::Sidebar;
            self.focus_handle.focus(window);
            cx.notify();
        }
    }

    fn handle_activate_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_state.focused_section == FocusedSection::Sidebar {
            self.activate_focused_sidebar_item(window, cx);
        }
        // Content activation is handled by the child page views directly
    }

    fn handle_escape_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_state.focused_section == FocusedSection::Content {
            self.focus_state.focused_section = FocusedSection::Sidebar;
            self.focus_handle.focus(window);
            cx.notify();
        }
    }
}

impl Render for MainWindowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Responsive sidebar: auto-collapse on small windows (< 768px)
        let viewport_width = window.viewport_size().width;
        let is_small_window = viewport_width < px(768.0);
        let sidebar_should_be_collapsed = is_small_window || self.sidebar_collapsed;

        div()
            .id("main-window-root")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .on_action(
                cx.listener(|this, action: &NavigateToThemeEdit, window, cx| {
                    this.navigate_to_theme_edit(action.0.clone(), window, cx);
                }),
            )
            .on_action(cx.listener(|this, _: &RefreshThemes, _window, cx| {
                this.themes_view.update(cx, |themes_page, cx| {
                    themes_page.refresh_themes(cx);
                });
            }))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::ToggleSidebar, _window, cx| {
                    this.sidebar_collapsed = !this.sidebar_collapsed;
                    let collapsed = this.sidebar_collapsed;
                    this.themes_view.update(cx, |themes_page, cx| {
                        themes_page.set_sidebar_collapsed(collapsed, cx);
                    });
                    cx.notify();
                },
            ))
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
            // Global page shortcuts
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToThemes, window, cx| {
                    this.focus_state.sidebar_index = 0;
                    this.focus_state.focused_section = FocusedSection::Content;
                    this.navigate_to(ActivePage::Themes, window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToConfig, window, cx| {
                    this.focus_state.sidebar_index = 1;
                    this.focus_state.focused_section = FocusedSection::Content;
                    this.navigate_to(ActivePage::Configuration, window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToStatusBar, window, cx| {
                    this.focus_state.sidebar_index = 2;
                    this.focus_state.focused_section = FocusedSection::Content;
                    this.navigate_to(ActivePage::StatusBar, window, cx);
                },
            ))
            // Sidebar keyboard navigation actions
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
                |this, _: &crate::ui::menu::app_menu::EscapeFocus, window, cx| {
                    this.handle_escape_focus(window, cx);
                },
            ))
            // Status bar profile actions (bubble up from status_bar_view dialogs).
            // The status_bar_view is only present after the user has first visited
            // the Status Bar page, so we guard with if-let.
            .on_action(
                cx.listener(|this, action: &WaybarProfileCreated, window, cx| {
                    if let Some(ref sv) = this.status_bar_view {
                        sv.update(cx, |view, cx| {
                            view.switch_profile(action.0.clone(), window, cx);
                        });
                    }
                }),
            )
            .on_action(
                cx.listener(|this, action: &WaybarProfileManaged, window, cx| {
                    if let Some(ref sv) = this.status_bar_view {
                        sv.update(cx, |view, cx| {
                            view.handle_profile_managed(action.0.clone(), window, cx);
                        });
                    }
                }),
            )
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
                                                    this.focus_state.sidebar_index = 0;
                                                    this.focus_state.focused_section =
                                                        FocusedSection::Content;
                                                    this.navigate_to(
                                                        ActivePage::Themes,
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
                                                        || self.is_sidebar_item_focused(1),
                                                )
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.focus_state.sidebar_index = 1;
                                                    this.focus_state.focused_section =
                                                        FocusedSection::Content;
                                                    this.navigate_to(
                                                        ActivePage::Configuration,
                                                        window,
                                                        cx,
                                                    );
                                                })),
                                        )
                                        .child(
                                            SidebarMenuItem::new("STATUS BAR")
                                                .icon(Icon::new(IconName::PanelBottom))
                                                .active(
                                                    self.is_page_active(ActivePage::StatusBar)
                                                        || self.is_sidebar_item_focused(2),
                                                )
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.focus_state.sidebar_index = 2;
                                                    this.focus_state.focused_section =
                                                        FocusedSection::Content;
                                                    this.navigate_to(
                                                        ActivePage::StatusBar,
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
            .children(Root::render_sheet_layer(window, cx))
    }
}
