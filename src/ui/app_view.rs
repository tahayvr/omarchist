use crate::system::flows::share::Imported;
use crate::ui::about_page::about_view::AboutView;
use crate::ui::app_events::{AppEvent, AppEvents};
use crate::ui::config_page::config_view::ConfigView;
use crate::ui::flows_page::{FlowEditPage, FlowEditSource, FlowsView};
use crate::ui::focus;
use crate::ui::keybinds_page::KeybindsView;
use crate::ui::menu::title_bar::MainTitleBar;
use crate::ui::omarchy_page::omarchy_view::OmarchyView;
use crate::ui::settings_page::settings_view::SettingsView;
use crate::ui::sidebar_nav;
use crate::ui::theme_edit_page::theme_edit_view::ThemeEditPage;
use crate::ui::themes_page::themes_view::ThemesPage;
use gpui::*;
use gpui_component::{
    ActiveTheme, Collapsible, Icon, IconName, Root, Side, h_flex,
    kbd::Kbd,
    sidebar::{Sidebar, SidebarGroup, SidebarItem, SidebarMenu, SidebarMenuItem},
};

use crate::system::ui_theme_watcher;
use gpui_kit::TestSupportExt;

const KEY_CONTEXT: &str = "MainWindow";

const SIDEBAR_CONTEXT: &str = "Sidebar";

/// Sidebar entries in display order: label, icon, page.
const SIDEBAR_ITEMS: [(&str, &str); 4] = [
    ("THEMES", "ctrl-1"),
    ("CONFIGURATION", "ctrl-2"),
    ("KEYBINDS", "ctrl-3"),
    ("FLOWS", "ctrl-4"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivePage {
    Themes,
    ThemeEdit(String), // Holds the theme name being edited
    Configuration,
    Keybinds,
    Flows,
    /// The editor for an existing flow, by id.
    FlowEdit(String),
    /// The editor for a new flow, optionally started from a template id.
    FlowNew(Option<String>),
    /// The editor reviewing a flow read from a file or URL, not yet saved.
    FlowImport(Box<Imported>),
    Settings,
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
    config_root: Option<AnyView>,
    config_view: Option<Entity<ConfigView>>,
    keybinds_root: Option<AnyView>,
    keybinds_view: Option<Entity<KeybindsView>>,
    flows_root: Option<AnyView>,
    flows_view: Option<Entity<FlowsView>>,
    // The flow editor is rebuilt every time it is opened.
    flow_edit_root: Option<AnyView>,
    flow_edit_view: Option<Entity<FlowEditPage>>,
    settings_root: Option<AnyView>,
    settings_view: Option<Entity<SettingsView>>,
    about_root: Option<AnyView>,
    about_view: Option<Entity<AboutView>>,
    omarchy_root: Option<AnyView>,
    omarchy_view: Option<Entity<OmarchyView>>,
    sidebar_collapsed: bool,
    /// The sidebar is one tab stop; arrow keys move `sidebar_index`.
    sidebar_focus: FocusHandle,
    sidebar_index: usize,
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
        let themes_view = cx.new(|cx| ThemesPage::new(window, cx));
        let themes_root = cx
            .new(|cx| Root::new(themes_view.clone(), window, cx))
            .into();

        let focus_handle = cx.focus_handle();
        let sidebar_focus = focus::tab_stop(cx);
        let initial_sidebar_index = Self::sidebar_index_for(&initial_page).unwrap_or(0);

        let mut view = Self {
            title_bar,
            active_page: ActivePage::Themes,
            themes_root,
            themes_view,
            theme_edit_root: None,
            theme_edit_view: None,
            theme_edit_name: None,
            config_root: None,
            config_view: None,
            keybinds_root: None,
            keybinds_view: None,
            flows_root: None,
            flows_view: None,
            flow_edit_root: None,
            flow_edit_view: None,
            settings_root: None,
            settings_view: None,
            about_root: None,
            about_view: None,
            omarchy_root: None,
            omarchy_view: None,
            sidebar_collapsed: true,
            sidebar_focus,
            sidebar_index: initial_sidebar_index,
            focus_handle,
        };

        // Cross-component requests (dialogs, cards, title bar, background
        // tasks) arrive through the AppEvents global.
        cx.observe_global_in::<AppEvents>(window, |this, window, cx| {
            for event in AppEvents::drain(cx) {
                this.handle_app_event(event, window, cx);
            }
        })
        .detach();

        // A page requested on the command line gets focus; otherwise the
        // sidebar does.
        if initial_page != ActivePage::Themes {
            view.navigate_to(initial_page, window, cx);
        } else {
            view.sidebar_focus.focus(window, cx);
        }

        view
    }

    fn sidebar_index_for(page: &ActivePage) -> Option<usize> {
        match page {
            ActivePage::Themes | ActivePage::ThemeEdit(_) => Some(0),
            ActivePage::Configuration => Some(1),
            ActivePage::Keybinds => Some(2),
            ActivePage::Flows
            | ActivePage::FlowEdit(_)
            | ActivePage::FlowNew(_)
            | ActivePage::FlowImport(_) => Some(3),
            ActivePage::Settings | ActivePage::About | ActivePage::Omarchy => None,
        }
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
            ActivePage::Keybinds => {
                if self.keybinds_root.is_none() {
                    let keybinds_view = cx.new(|cx| KeybindsView::new(window, cx));
                    self.keybinds_root = Some(
                        cx.new(|cx| Root::new(keybinds_view.clone(), window, cx))
                            .into(),
                    );
                    self.keybinds_view = Some(keybinds_view);
                }
            }
            ActivePage::Flows => {
                if self.flows_root.is_none() {
                    let flows_view = cx.new(|cx| FlowsView::new(window, cx));
                    self.flows_root = Some(
                        cx.new(|cx| Root::new(flows_view.clone(), window, cx))
                            .into(),
                    );
                    self.flows_view = Some(flows_view);
                } else if let Some(view) = &self.flows_view {
                    // Coming back from the editor: show what it saved.
                    view.update(cx, |view, cx| view.refresh(cx));
                }
            }
            // Always rebuilt from disk, so discarded edits never resurface.
            ActivePage::FlowEdit(_) | ActivePage::FlowNew(_) | ActivePage::FlowImport(_) => {
                let source = match page {
                    ActivePage::FlowEdit(id) => FlowEditSource::Existing(id.clone()),
                    ActivePage::FlowNew(template) => FlowEditSource::New(template.clone()),
                    ActivePage::FlowImport(imported) => FlowEditSource::Imported(imported.clone()),
                    _ => unreachable!(),
                };
                let view = cx.new(|cx| FlowEditPage::new(source, window, cx));
                self.flow_edit_root = Some(cx.new(|cx| Root::new(view.clone(), window, cx)).into());
                self.flow_edit_view = Some(view);
            }
            ActivePage::Settings => {
                if self.settings_root.is_none() {
                    let settings_view = cx.new(SettingsView::new);
                    self.settings_root = Some(
                        cx.new(|cx| Root::new(settings_view.clone(), window, cx))
                            .into(),
                    );
                    self.settings_view = Some(settings_view);
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
                let updates = self.title_bar.read(cx).updates().clone();
                updates.update(cx, |updates, cx| updates.refresh_if_stale(cx));
                if self.omarchy_root.is_none() {
                    let omarchy_view = cx.new(|cx| OmarchyView::new(updates, cx));
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

    /// The page currently shown.
    pub fn active_page(&self) -> &ActivePage {
        &self.active_page
    }

    /// The sidebar entry the keyboard cursor is on.
    pub fn sidebar_index(&self) -> usize {
        self.sidebar_index
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
        if let Some(ix) = Self::sidebar_index_for(&self.active_page) {
            self.sidebar_index = ix;
        }

        // Keyboard users land on the page's first control.
        self.focus_page_entry(window, cx);

        cx.notify();
    }

    /// Focuses the active page's entry control (search box, tab strip, …).
    fn focus_page_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        match &self.active_page {
            ActivePage::Themes => self
                .themes_view
                .update(cx, |v, cx| v.focus_entry(window, cx)),
            ActivePage::ThemeEdit(_) => {
                if let Some(view) = &self.theme_edit_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::About => {
                if let Some(view) = &self.about_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::Omarchy => {
                if let Some(view) = &self.omarchy_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::Configuration => {
                if let Some(view) = &self.config_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::Settings => {
                if let Some(view) = &self.settings_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::Keybinds => {
                if let Some(view) = &self.keybinds_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::Flows => {
                if let Some(view) = &self.flows_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::FlowEdit(_) | ActivePage::FlowNew(_) | ActivePage::FlowImport(_) => {
                if let Some(view) = &self.flow_edit_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
        }
    }

    /// Escape toggles between the sidebar and the page.
    fn handle_escape(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sidebar_focus.is_focused(window) {
            self.focus_page_entry(window, cx);
        } else {
            self.sidebar_focus.focus(window, cx);
        }
        cx.notify();
    }

    /// Ctrl+R: reload whatever the active page shows.
    fn reload_page(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match &self.active_page {
            ActivePage::Themes | ActivePage::ThemeEdit(_) => {
                self.themes_view
                    .update(cx, |page, cx| page.refresh_themes(cx));
            }
            ActivePage::Keybinds => {
                if let Some(view) = &self.keybinds_view {
                    view.update(cx, |view, cx| view.refresh(cx));
                }
            }
            ActivePage::Flows
            | ActivePage::FlowEdit(_)
            | ActivePage::FlowNew(_)
            | ActivePage::FlowImport(_) => {
                if let Some(view) = &self.flows_view {
                    view.update(cx, |view, cx| view.refresh(cx));
                }
            }
            ActivePage::Configuration => {
                if let Some(view) = &self.config_view {
                    view.update(cx, |view, cx| view.reload(window, cx));
                }
            }
            ActivePage::Settings | ActivePage::About | ActivePage::Omarchy => {}
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

    fn handle_app_event(&mut self, event: AppEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            AppEvent::Navigate(page) => self.navigate_to(page, window, cx),
            AppEvent::RefreshThemes => {
                self.themes_view.update(cx, |themes_page, cx| {
                    themes_page.refresh_themes(cx);
                });
            }
            AppEvent::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
                let collapsed = self.sidebar_collapsed;
                self.themes_view.update(cx, |themes_page, cx| {
                    themes_page.set_sidebar_collapsed(collapsed, cx);
                });
                cx.notify();
            }
            AppEvent::ReloadUiTheme => {
                ui_theme_watcher::load_and_apply_omarchy_theme(cx);
                cx.refresh_windows();
            }
        }
    }

    fn current_page_view(&self) -> AnyView {
        match &self.active_page {
            ActivePage::Themes => self.themes_root.clone(),
            ActivePage::ThemeEdit(_) => self
                .theme_edit_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::Configuration => self
                .config_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::Keybinds => self
                .keybinds_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::Flows => self
                .flows_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::FlowEdit(_) | ActivePage::FlowNew(_) | ActivePage::FlowImport(_) => self
                .flow_edit_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::Settings => self
                .settings_root
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
            (ActivePage::Configuration, ActivePage::Configuration) => true,
            (ActivePage::Keybinds, ActivePage::Keybinds) => true,
            (ActivePage::Flows, ActivePage::Flows) => true,
            (
                ActivePage::FlowEdit(_) | ActivePage::FlowNew(_) | ActivePage::FlowImport(_),
                ActivePage::Flows,
            ) => true,
            (ActivePage::Settings, ActivePage::Settings) => true,
            (ActivePage::About, ActivePage::About) => true,
            (ActivePage::Omarchy, ActivePage::Omarchy) => true,
            _ => false,
        }
    }

    fn page_from_sidebar_index(&self, index: usize) -> ActivePage {
        match index {
            0 => ActivePage::Themes,
            1 => ActivePage::Configuration,
            2 => ActivePage::Keybinds,
            3 => ActivePage::Flows,
            _ => ActivePage::Themes,
        }
    }

    fn activate_sidebar_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let page = self.page_from_sidebar_index(self.sidebar_index);
        if self.active_page == page {
            self.focus_page_entry(window, cx);
        } else {
            self.navigate_to(page, window, cx);
        }
    }

    fn move_sidebar_index(&mut self, index: usize, cx: &mut Context<Self>) {
        self.sidebar_index = index.min(SIDEBAR_ITEMS.len() - 1);
        cx.notify();
    }

    /// One sidebar page entry, with its focus ring.
    fn sidebar_item(&self, ix: usize, window: &Window, cx: &mut Context<Self>) -> SidebarMenuItem {
        let (label, keys) = SIDEBAR_ITEMS[ix];
        let page = self.page_from_sidebar_index(ix);
        let icon = match ix {
            0 => Icon::new(IconName::LayoutDashboard),
            1 => Icon::new(IconName::Settings),
            2 => Icon::new(Icon::empty()).path("icons/keyboard.svg"),
            _ => Icon::new(Icon::empty()).path("icons/workflow.svg"),
        };
        let focused = self.sidebar_focus.is_focused(window) && self.sidebar_index == ix;
        let border = focus::focus_border(focused, cx.theme().transparent, cx);

        SidebarMenuItem::new(label)
            .icon(icon)
            .border_1()
            .border_color(border)
            .active(self.is_page_active(page.clone()))
            .suffix(move |_, _| Kbd::new(Keystroke::parse(keys).unwrap()))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.sidebar_index = ix;
                this.navigate_to(page.clone(), window, cx);
            }))
    }

    fn sidebar_should_be_collapsed(&self, window: &Window) -> bool {
        // Responsive sidebar: auto-collapse on small windows (< 768px)
        let is_small_window = window.viewport_size().width < px(768.0);
        is_small_window || self.sidebar_collapsed
    }
}

/// The sidebar page list as one focusable composite: a `SidebarMenu` whose
/// container carries the `Sidebar` key context and the roving focus handle.
#[derive(Clone)]
struct SidebarNav {
    focus: FocusHandle,
    collapsed: bool,
    items: Vec<SidebarMenuItem>,
}

impl Collapsible for SidebarNav {
    fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
}

impl SidebarItem for SidebarNav {
    fn render(
        self,
        _id: impl Into<ElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        // Fixed id: the sidebar's test target.
        div()
            .id("sidebar-nav")
            .test_support()
            .key_context(SIDEBAR_CONTEXT)
            .track_focus(&self.focus)
            .cursor_pointer()
            .child(
                SidebarMenu::new()
                    .collapsed(self.collapsed)
                    .children(self.items)
                    .render("sidebar-nav-menu", window, cx),
            )
    }
}

impl Render for MainWindowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar_should_be_collapsed = self.sidebar_should_be_collapsed(window);

        self.themes_view.update(cx, |themes_page, cx| {
            themes_page.set_sidebar_collapsed(sidebar_should_be_collapsed, cx);
        });

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
            .on_action(
                cx.listener(|_, _: &crate::ui::menu::app_menu::NewTheme, window, cx| {
                    crate::ui::dialogs::create_theme_dialog::open_create_theme_dialog(window, cx);
                }),
            )
            // Global page shortcuts
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToThemes, window, cx| {
                    this.navigate_to(ActivePage::Themes, window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToConfig, window, cx| {
                    this.navigate_to(ActivePage::Configuration, window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToKeybinds, window, cx| {
                    this.navigate_to(ActivePage::Keybinds, window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToFlows, window, cx| {
                    this.navigate_to(ActivePage::Flows, window, cx);
                },
            ))
            // Focus traversal (native GPUI tab stops)
            .on_action(cx.listener(|_, _: &focus::FocusNext, window, cx| {
                focus::focus_next_trapped(true, window, cx);
            }))
            .on_action(cx.listener(|_, _: &focus::FocusPrev, window, cx| {
                focus::focus_next_trapped(false, window, cx);
            }))
            .on_action(cx.listener(|this, _: &focus::EscapeToSidebar, window, cx| {
                this.handle_escape(window, cx);
            }))
            .on_action(cx.listener(|this, _: &focus::ReloadPage, window, cx| {
                this.reload_page(window, cx);
            }))
            .on_action(cx.listener(|_, _: &focus::ShowShortcuts, window, cx| {
                crate::ui::dialogs::shortcuts_dialog::open_shortcuts_dialog(window, cx);
            }))
            .on_action(cx.listener(|this, _: &focus::ShowCommands, window, cx| {
                crate::ui::dialogs::command_palette::open_command_palette(
                    this.focus_handle.clone(),
                    window,
                    cx,
                );
            }))
            // Sidebar composite
            .on_action(cx.listener(|this, _: &sidebar_nav::Next, _, cx| {
                this.move_sidebar_index(this.sidebar_index + 1, cx);
            }))
            .on_action(cx.listener(|this, _: &sidebar_nav::Prev, _, cx| {
                this.move_sidebar_index(this.sidebar_index.saturating_sub(1), cx);
            }))
            .on_action(cx.listener(|this, _: &sidebar_nav::First, _, cx| {
                this.move_sidebar_index(0, cx);
            }))
            .on_action(cx.listener(|this, _: &sidebar_nav::Last, _, cx| {
                this.move_sidebar_index(SIDEBAR_ITEMS.len() - 1, cx);
            }))
            .on_action(cx.listener(|this, _: &sidebar_nav::Activate, window, cx| {
                this.activate_sidebar_item(window, cx);
            }))
            .child(self.title_bar.clone())
            .child(
                h_flex()
                    .flex_1()
                    .size_full()
                    .overflow_hidden()
                    .child(
                        Sidebar::new("main-sidebar")
                            .side(Side::Left)
                            .collapsed(sidebar_should_be_collapsed)
                            .child(
                                SidebarGroup::new("Navigation").child(SidebarNav {
                                    focus: self.sidebar_focus.clone(),
                                    collapsed: sidebar_should_be_collapsed,
                                    items: (0..SIDEBAR_ITEMS.len())
                                        .map(|ix| self.sidebar_item(ix, window, cx))
                                        .collect(),
                                }),
                            )
                            .footer(
                                SidebarMenu::new()
                                    .cursor_pointer()
                                    .collapsed(sidebar_should_be_collapsed)
                                    .child(
                                        SidebarMenuItem::new("Toggle Sidebar")
                                            .icon(Icon::new(IconName::PanelLeft))
                                            .suffix(|_, _| {
                                                Kbd::new(Keystroke::parse("ctrl-b").unwrap())
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.sidebar_collapsed = !this.sidebar_collapsed;
                                                this.themes_view.update(cx, |themes_page, cx| {
                                                    themes_page.set_sidebar_collapsed(
                                                        this.sidebar_collapsed,
                                                        cx,
                                                    );
                                                });
                                                cx.notify();
                                            })),
                                    )
                                    .render("sidebar-footer-menu", window, cx),
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
