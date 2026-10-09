use crate::system::flows::share::Imported;
use crate::system::flows::store::load_flow;
use crate::system::themes::theme_file_ops::is_omarchist_theme;
use crate::ui::about_page::about_view::AboutView;
use crate::ui::app_events::{AppEvent, AppEvents, emit};
use crate::ui::config_page::config_view::ConfigView;
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::flows_page::share_ui::import_flow_from_dialog;
use crate::ui::flows_page::{FlowEditPage, FlowEditSource, FlowsView, GalleryView, TemplatesView};
use crate::ui::focus;
use crate::ui::keybinds_page::KeybindsView;
use crate::ui::menu::title_bar::MainTitleBar;
use crate::ui::notify;
use crate::ui::omarchy_page::omarchy_view::OmarchyView;
use crate::ui::palette::Area;
use crate::ui::settings_page::settings_view::SettingsView;
use crate::ui::sidebar_nav;
use crate::ui::theme_edit_page::theme_edit_view::ThemeEditPage;
use crate::ui::themes_page::themes_view::ThemesPage;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Collapsible, Icon, IconName, Root, Side, WindowExt, h_flex,
    kbd::Kbd,
    sidebar::{Sidebar, SidebarGroup, SidebarItem, SidebarMenu, SidebarMenuItem},
    tooltip::Tooltip,
    v_flex,
};

use crate::system::ui_theme_watcher;
use gpui_kit::TestSupportExt;

const KEY_CONTEXT: &str = "MainWindow";

const SIDEBAR_CONTEXT: &str = "Sidebar";

/// Sidebar entries in display order: label, icon, page.
const SIDEBAR_ITEMS: [(&str, &str); 4] = [
    ("Themes", "ctrl-1"),
    ("Configuration", "ctrl-2"),
    ("Keybinds", "ctrl-3"),
    ("Flows", "ctrl-4"),
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
    /// The templates to start a new flow from.
    FlowTemplates,
    /// The flows other people shared.
    FlowGallery,
    /// The editor on the saved flow with this id, holding the steps of a
    /// newer gallery version, not yet saved.
    FlowUpdate(String, Box<Imported>),
    Settings,
    About,
    Omarchy,
}

impl ActivePage {
    /// The page's name on the command line (`--view`) and in
    /// `settings.json`; the editors count as their list page.
    pub fn view_name(&self) -> &'static str {
        match self {
            ActivePage::Themes | ActivePage::ThemeEdit(_) => "themes",
            ActivePage::Configuration => "config",
            ActivePage::Keybinds => "keybinds",
            ActivePage::Flows
            | ActivePage::FlowEdit(_)
            | ActivePage::FlowNew(_)
            | ActivePage::FlowImport(_)
            | ActivePage::FlowUpdate(..)
            | ActivePage::FlowTemplates
            | ActivePage::FlowGallery => "flows",
            ActivePage::Settings => "settings",
            ActivePage::About => "about",
            ActivePage::Omarchy => "omarchy",
        }
    }

    pub fn from_view_name(name: &str) -> Option<Self> {
        Some(match name {
            "themes" => ActivePage::Themes,
            "config" => ActivePage::Configuration,
            "keybinds" => ActivePage::Keybinds,
            "flows" => ActivePage::Flows,
            "settings" => ActivePage::Settings,
            "about" => ActivePage::About,
            "omarchy" => ActivePage::Omarchy,
            _ => return None,
        })
    }
}

pub struct MainWindowView {
    /// Whether the Flows page polls the run registry; `main.rs` turns it
    /// on, headless tests leave it off.
    watch_runs: bool,
    title_bar: Entity<MainTitleBar>,
    active_page: ActivePage,
    // Default page — always present
    themes_root: AnyView,
    themes_view: Entity<ThemesPage>,
    // ThemeEdit is created on first navigation to a given theme
    theme_edit_root: Option<AnyView>,
    theme_edit_view: Option<Entity<ThemeEditPage>>,
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
    flow_templates_root: Option<AnyView>,
    flow_templates_view: Option<Entity<TemplatesView>>,
    flow_gallery_root: Option<AnyView>,
    flow_gallery_view: Option<Entity<GalleryView>>,
    settings_root: Option<AnyView>,
    settings_view: Option<Entity<SettingsView>>,
    about_root: Option<AnyView>,
    about_view: Option<Entity<AboutView>>,
    omarchy_root: Option<AnyView>,
    omarchy_view: Option<Entity<OmarchyView>>,
    /// The user's own choice from the toggle; `None` follows the width.
    sidebar_expanded: Option<bool>,
    /// The sidebar is one tab stop; arrow keys move `sidebar_index`.
    sidebar_focus: FocusHandle,
    sidebar_index: usize,
    focus_handle: FocusHandle,
}

/// Whether a dialog is open. Safe before the window's `Root` exists (the
/// initial navigation runs while the view is being built).
fn dialog_open(window: &mut Window, cx: &mut App) -> bool {
    window.root::<Root>().flatten().is_some() && window.has_active_dialog(cx)
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
        let themes_root = themes_view.clone().into();

        let focus_handle = cx.focus_handle();
        let sidebar_focus = focus::tab_stop(cx);
        let initial_sidebar_index = Self::sidebar_index_for(&initial_page).unwrap_or(0);

        let mut view = Self {
            watch_runs: false,
            title_bar,
            active_page: ActivePage::Themes,
            themes_root,
            themes_view,
            theme_edit_root: None,
            theme_edit_view: None,
            config_root: None,
            config_view: None,
            keybinds_root: None,
            keybinds_view: None,
            flows_root: None,
            flows_view: None,
            flow_edit_root: None,
            flow_edit_view: None,
            flow_templates_root: None,
            flow_templates_view: None,
            flow_gallery_root: None,
            flow_gallery_view: None,
            settings_root: None,
            settings_view: None,
            about_root: None,
            about_view: None,
            omarchy_root: None,
            omarchy_view: None,
            sidebar_expanded: None,
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
        // Anything emitted before this view existed (the instance socket,
        // the theme watcher) would otherwise wait for the next emit; a
        // touch of the global runs the observer above.
        if AppEvents::has_pending(cx) {
            cx.defer(|cx| cx.update_global::<AppEvents, _>(|_, _| {}));
        }

        if !crate::system::omarchy_paths::is_quattro_installed() {
            window.defer(cx, |window, cx| {
                notify::warning(
                    window,
                    "Omarchist 2 needs Omarchy 4 (Quattro), which was not found; Hyprland \
                     settings, keybinds and the bar widget are not applied on this system",
                    cx,
                );
            });
        }

        // A settings.json that could not be read was set aside at startup.
        if crate::system::config::config_setup::SETTINGS_RESET
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            window.defer(cx, |window, cx| {
                notify::warning(
                    window,
                    "settings.json could not be read; it was kept as settings.json.broken and \
                     the defaults were restored",
                    cx,
                );
            });
        }

        // Closing the window (the title bar, SUPER+W) goes through the same
        // unsaved-changes check as Ctrl+Q.
        let this = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            this.update(cx, |this, cx| this.request_quit(window, cx))
                .unwrap_or(true)
        });

        // A page requested on the command line gets focus; otherwise the
        // sidebar does.
        if initial_page != ActivePage::Themes {
            view.navigate_to(initial_page, window, cx);
        } else {
            view.sidebar_focus.focus(window, cx);
        }

        view
    }

    /// Quits unless the flow editor holds unsaved changes, in which case it
    /// asks first. Pending Designer saves are written either way. Returns
    /// whether the window may close now.
    fn request_quit(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if let Some(editor) = &self.theme_edit_view {
            editor.update(cx, |editor, cx| editor.flush_pending_saves(cx));
        }
        let dirty = matches!(
            self.active_page,
            ActivePage::FlowEdit(_)
                | ActivePage::FlowNew(_)
                | ActivePage::FlowImport(_)
                | ActivePage::FlowUpdate(..)
        ) && self
            .flow_edit_view
            .as_ref()
            .is_some_and(|editor| editor.read(cx).is_dirty(cx));
        if !dirty {
            return true;
        }
        let editor = self.flow_edit_view.clone();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Discard changes and quit?",
                message: "This flow has changes that are not saved.".to_string(),
                confirm_label: "Discard and quit",
                danger: true,
            },
            move |_, cx| {
                if let Some(editor) = &editor {
                    editor.update(cx, |editor, _| editor.discard());
                }
                cx.quit();
            },
            window,
            cx,
        );
        false
    }

    fn sidebar_index_for(page: &ActivePage) -> Option<usize> {
        match page {
            ActivePage::Themes | ActivePage::ThemeEdit(_) => Some(0),
            ActivePage::Configuration => Some(1),
            ActivePage::Keybinds => Some(2),
            ActivePage::Flows
            | ActivePage::FlowEdit(_)
            | ActivePage::FlowNew(_)
            | ActivePage::FlowImport(_)
            | ActivePage::FlowUpdate(..)
            | ActivePage::FlowTemplates
            | ActivePage::FlowGallery => Some(3),
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
            // Built fresh from disk on every visit: a theme deleted and
            // recreated under the same name must not get the old editor,
            // whose tabs hold the deleted theme's snapshot.
            ActivePage::ThemeEdit(theme_name) => {
                let theme_edit_view =
                    cx.new(|cx| ThemeEditPage::new(theme_name.clone(), window, cx));
                self.theme_edit_root = Some(theme_edit_view.clone().into());
                self.theme_edit_view = Some(theme_edit_view);
            }
            ActivePage::Configuration => match &self.config_view {
                // Omarchy's own menu changes these values too: read them
                // again on every visit.
                Some(view) => view.update(cx, |view, cx| view.refresh_omarchy_values(window, cx)),
                None => {
                    let config_view = cx.new(|cx| ConfigView::new(window, cx));
                    self.config_root = Some(config_view.clone().into());
                    self.config_view = Some(config_view);
                }
            },
            ActivePage::Keybinds => match &self.keybinds_view {
                // The Flows page saves keybinds too, and bindings.lua may
                // have been edited: every visit rescans.
                Some(view) => view.update(cx, |view, cx| view.refresh(window, cx)),
                None => {
                    let keybinds_view = cx.new(|cx| KeybindsView::new(window, cx));
                    self.keybinds_root = Some(keybinds_view.clone().into());
                    self.keybinds_view = Some(keybinds_view);
                }
            },
            ActivePage::FlowTemplates => match &self.flow_templates_view {
                // A template file can be copied in while the app runs, so
                // the page reloads on every visit.
                Some(view) => view.update(cx, |view, cx| view.refresh(cx)),
                None => {
                    let view = cx.new(|cx| TemplatesView::new(window, cx));
                    self.flow_templates_root = Some(view.clone().into());
                    self.flow_templates_view = Some(view);
                }
            },
            ActivePage::FlowGallery => match &self.flow_gallery_view {
                // Every visit looks for what is new.
                Some(view) => view.update(cx, |view, cx| view.refresh(cx)),
                None => {
                    let view = cx.new(|cx| GalleryView::new(window, cx));
                    self.flow_gallery_root = Some(view.clone().into());
                    self.flow_gallery_view = Some(view);
                }
            },
            ActivePage::Flows => {
                if self.flows_root.is_none() {
                    let flows_view = cx.new(|cx| FlowsView::new(window, cx));
                    if self.watch_runs {
                        flows_view.update(cx, |view, cx| view.watch_runs(cx));
                    }
                    self.flows_root = Some(flows_view.clone().into());
                    self.flows_view = Some(flows_view);
                } else if let Some(view) = &self.flows_view {
                    // Coming back from the editor: show what it saved.
                    view.update(cx, |view, cx| view.refresh(cx));
                }
            }
            // Always rebuilt from disk, so discarded edits never resurface.
            ActivePage::FlowEdit(_)
            | ActivePage::FlowNew(_)
            | ActivePage::FlowImport(_)
            | ActivePage::FlowUpdate(..) => {
                // The editor that just saved this flow (a first save gives
                // it its id) stays, run in progress and all.
                if let ActivePage::FlowEdit(id) = page
                    && let Some(view) = &self.flow_edit_view
                    && view.read(cx).saved_id(cx) == Some(id.as_str())
                {
                    return;
                }
                let source = match page {
                    ActivePage::FlowEdit(id) => FlowEditSource::Existing(id.clone()),
                    ActivePage::FlowNew(template) => FlowEditSource::New(template.clone()),
                    ActivePage::FlowImport(imported) => FlowEditSource::Imported(imported.clone()),
                    ActivePage::FlowUpdate(id, imported) => {
                        FlowEditSource::Update(id.clone(), imported.clone())
                    }
                    _ => unreachable!(),
                };
                let view = cx.new(|cx| FlowEditPage::new(source, window, cx));
                self.flow_edit_root = Some(view.clone().into());
                self.flow_edit_view = Some(view);
            }
            ActivePage::Settings => match &self.settings_view {
                Some(view) => view.update(cx, |view, cx| view.refresh(cx)),
                None => {
                    let settings_view = cx.new(SettingsView::new);
                    self.settings_root = Some(settings_view.clone().into());
                    self.settings_view = Some(settings_view);
                }
            },
            ActivePage::About => {
                if self.about_root.is_none() {
                    let about_view = cx.new(AboutView::new);
                    self.about_root = Some(about_view.clone().into());
                    self.about_view = Some(about_view);
                }
            }
            ActivePage::Omarchy => {
                let updates = self.title_bar.read(cx).updates().clone();
                updates.update(cx, |updates, cx| updates.refresh_if_stale(cx));
                if self.omarchy_root.is_none() {
                    let omarchy_view = cx.new(|cx| OmarchyView::new(updates, cx));
                    self.omarchy_root = Some(omarchy_view.clone().into());
                    self.omarchy_view = Some(omarchy_view);
                }
            }
            // Themes is always present; a visit rescans, because the editor,
            // the CLI and Omarchy itself change the folder behind its back.
            ActivePage::Themes => {
                self.themes_view
                    .update(cx, |view, cx| view.refresh_themes(window, cx));
            }
        }
    }

    /// The page currently shown.
    /// The flow editor while it is the page on screen.
    pub fn flow_editor(&self) -> Option<Entity<FlowEditPage>> {
        self.flow_edit_view.clone()
    }

    /// The Gallery page, once it has been visited.
    pub fn flow_gallery(&self) -> Option<Entity<GalleryView>> {
        self.flow_gallery_view.clone()
    }

    pub fn active_page(&self) -> &ActivePage {
        &self.active_page
    }

    /// The sidebar entry the keyboard cursor is on.
    pub fn sidebar_index(&self) -> usize {
        self.sidebar_index
    }

    pub fn navigate_to(&mut self, page: ActivePage, window: &mut Window, cx: &mut Context<Self>) {
        // Page shortcuts reach this view from inside a dialog too (the
        // dialog layer is drawn under this view's element); a page must not
        // change behind an open dialog.
        if dialog_open(window, cx) {
            return;
        }
        let page = match page {
            ActivePage::ThemeEdit(name) if !is_omarchist_theme(&name) => {
                // Deferred: at startup this runs before the window's `Root`
                // exists, and notifications live on the `Root`.
                let message = crate::error::Error::NotOmarchistTheme(name).to_string();
                window.defer(cx, move |window, cx| notify::error(window, message, cx));
                ActivePage::Themes
            }
            page => page,
        };
        if self.active_page == page {
            return;
        }

        // Leaving the flow editor with unsaved changes asks first, whichever
        // way the user leaves (sidebar, shortcut, palette, `--view`).
        let editing_flow = matches!(
            self.active_page,
            ActivePage::FlowEdit(_)
                | ActivePage::FlowNew(_)
                | ActivePage::FlowImport(_)
                | ActivePage::FlowUpdate(..)
        );
        if editing_flow
            && let Some(editor) = self.flow_edit_view.clone()
            && editor.read(cx).is_dirty(cx)
        {
            open_confirm_dialog(
                ConfirmDialog {
                    title: "Discard changes?",
                    message: "This flow has changes that are not saved.".to_string(),
                    confirm_label: "Discard",
                    danger: true,
                },
                move |_, cx| {
                    editor.update(cx, |editor, _| editor.discard());
                    emit(cx, AppEvent::Navigate(page.clone()));
                },
                window,
                cx,
            );
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
                crate::ui::theme_apply::apply_theme(theme_name.clone(), window, cx);
            }
        }

        self.active_page = page;
        if let Some(ix) = Self::sidebar_index_for(&self.active_page) {
            self.sidebar_index = ix;
        }
        // Only written when the Settings page asks to reopen on the last page.
        if let Err(e) =
            crate::system::config::config_setup::remember_last_page(self.active_page.view_name())
        {
            eprintln!("Failed to remember the last page: {e}");
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
            ActivePage::FlowTemplates => {
                if let Some(view) = &self.flow_templates_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::FlowGallery => {
                if let Some(view) = &self.flow_gallery_view {
                    view.update(cx, |v, cx| v.focus_entry(window, cx));
                }
            }
            ActivePage::FlowEdit(_)
            | ActivePage::FlowNew(_)
            | ActivePage::FlowImport(_)
            | ActivePage::FlowUpdate(..) => {
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
                    .update(cx, |page, cx| page.refresh_themes(window, cx));
            }
            ActivePage::Keybinds => {
                if let Some(view) = &self.keybinds_view {
                    view.update(cx, |view, cx| view.refresh(window, cx));
                }
            }
            ActivePage::FlowTemplates => {
                if let Some(view) = &self.flow_templates_view {
                    view.update(cx, |view, cx| view.refresh(cx));
                }
            }
            ActivePage::FlowGallery => {
                if let Some(view) = &self.flow_gallery_view {
                    view.update(cx, |view, cx| view.refresh(cx));
                }
            }
            ActivePage::Flows
            | ActivePage::FlowEdit(_)
            | ActivePage::FlowNew(_)
            | ActivePage::FlowImport(_)
            | ActivePage::FlowUpdate(..) => {
                if let Some(view) = &self.flows_view {
                    view.update(cx, |view, cx| view.refresh(cx));
                }
            }
            ActivePage::Configuration => {
                if let Some(view) = &self.config_view {
                    view.update(cx, |view, cx| view.reload(window, cx));
                }
            }
            ActivePage::Omarchy => {
                if let Some(view) = &self.omarchy_view {
                    view.update(cx, |view, cx| view.check_again(cx));
                }
            }
            ActivePage::Settings => {
                if let Some(view) = &self.settings_view {
                    view.update(cx, |view, cx| view.refresh(cx));
                }
            }
            ActivePage::About => {}
        }
    }

    /// Runs a saved flow in the background and reports the outcome in a
    /// notification, the same way the Flows page does.
    fn run_flow(&self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let result = match load_flow(&id) {
                Ok(flow) => crate::system::flows::runner::run_in_thread(flow.clone())
                    .await
                    .map(|outcome| (flow, outcome)),
                Err(e) => Err(e),
            };
            this.update_in(cx, |_, window, cx| match result {
                Ok((flow, outcome)) => {
                    notify::result(window, outcome.is_ok(), outcome.summary(&flow), cx)
                }
                Err(e) => notify::error(window, format!("Could not run the flow: {e}"), cx),
            })
            .ok();
        })
        .detach();
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
                    themes_page.refresh_themes(window, cx);
                });
            }
            AppEvent::ToggleSidebar => {
                self.toggle_sidebar(window, cx);
            }
            AppEvent::ReloadUiTheme => {
                ui_theme_watcher::apply_ui_theme(cx);
                cx.refresh_windows();
            }
            AppEvent::Error(message) => notify::error(window, message, cx),
            AppEvent::Warning(message) => notify::warning(window, message, cx),
            AppEvent::Success(message) => notify::success(window, message, cx),
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
            ActivePage::FlowEdit(_)
            | ActivePage::FlowNew(_)
            | ActivePage::FlowImport(_)
            | ActivePage::FlowUpdate(..) => self
                .flow_edit_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::FlowTemplates => self
                .flow_templates_root
                .clone()
                .unwrap_or_else(|| self.themes_root.clone()),
            ActivePage::FlowGallery => self
                .flow_gallery_root
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
                ActivePage::FlowEdit(_)
                | ActivePage::FlowNew(_)
                | ActivePage::FlowImport(_)
                | ActivePage::FlowUpdate(..),
                ActivePage::Flows,
            ) => true,
            (ActivePage::FlowTemplates | ActivePage::FlowGallery, ActivePage::Flows) => true,
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

    /// Starts the Flows page's poll of the run registry, now and for every
    /// Flows page made later. `main.rs` calls it; tests never do.
    pub fn watch_runs(&mut self, cx: &mut Context<Self>) {
        self.watch_runs = true;
        if let Some(view) = &self.flows_view {
            view.update(cx, |view, cx| view.watch_runs(cx));
        }
    }

    /// One sidebar page entry, with its focus ring.
    fn sidebar_item(&self, ix: usize, window: &Window, cx: &mut Context<Self>) -> SidebarMenuItem {
        let (label, keys) = SIDEBAR_ITEMS[ix];
        let page = self.page_from_sidebar_index(ix);
        // Each page's icon wears the page's colour, the same one its
        // badges and section icons wear elsewhere.
        let (icon, area) = match ix {
            0 => (Icon::new(IconName::LayoutDashboard), Area::Themes),
            1 => (Icon::new(IconName::Settings), Area::Configuration),
            2 => (
                Icon::new(Icon::empty()).path("icons/keyboard.svg"),
                Area::Keybinds,
            ),
            _ => (
                Icon::new(Icon::empty()).path("icons/workflow.svg"),
                Area::Flows,
            ),
        };
        let icon = icon.text_color(area.accent(cx));
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
        // Collapsed until the user opens it; their choice then holds at
        // any window width, so the toggle never looks broken.
        let _ = window;
        !self.sidebar_expanded.unwrap_or(false)
    }

    /// Flips the sidebar from whatever it shows now; an explicit choice
    /// outlives the window's width.
    fn toggle_sidebar(&mut self, window: &Window, cx: &mut Context<Self>) {
        let collapsed = self.sidebar_should_be_collapsed(window);
        self.sidebar_expanded = Some(collapsed);
        self.themes_view.update(cx, |themes_page, cx| {
            themes_page.set_sidebar_collapsed(!collapsed, cx);
        });
        cx.notify();
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
                // Laid out as `SidebarMenu` does, so a collapsed item (an
                // icon alone) can carry its page name as a tooltip.
                v_flex()
                    .gap_2()
                    .children(self.items.into_iter().enumerate().map(|(ix, item)| {
                        let (label, keys) = SIDEBAR_ITEMS[ix];
                        let item = item.collapsed(self.collapsed).render(
                            ("sidebar-nav-menu", ix),
                            window,
                            cx,
                        );
                        div()
                            .id(("sidebar-nav-tip", ix))
                            .when(self.collapsed, |this: Stateful<Div>| {
                                this.tooltip(move |window, cx| {
                                    Tooltip::new(label)
                                        .key_binding(Some(Kbd::new(
                                            Keystroke::parse(keys).unwrap(),
                                        )))
                                        .build(window, cx)
                                })
                            })
                            .child(item)
                    })),
            )
    }
}

impl Render for MainWindowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar_should_be_collapsed = self.sidebar_should_be_collapsed(window);

        self.themes_view.update(cx, |themes_page, cx| {
            themes_page.set_sidebar_collapsed(sidebar_should_be_collapsed, cx);
        });
        if let Some(view) = &self.keybinds_view {
            view.update(cx, |view, cx| {
                view.set_sidebar_collapsed(sidebar_should_be_collapsed, cx)
            });
        }

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
            .on_action(
                cx.listener(|this, _: &crate::ui::menu::app_menu::Quit, window, cx| {
                    if this.request_quit(window, cx) {
                        cx.quit();
                    }
                }),
            )
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateToOmarchy, window, cx| {
                    this.navigate_to(ActivePage::Omarchy, window, cx);
                },
            ))
            .on_action(cx.listener(
                |_, _: &crate::ui::menu::app_menu::RefreshTheme, window, cx| {
                    crate::ui::theme_apply::refresh_theme(window, cx);
                },
            ))
            .on_action(
                cx.listener(|_, _: &crate::ui::menu::app_menu::NewTheme, window, cx| {
                    if dialog_open(window, cx) {
                        return;
                    }
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
            // Title-bar menus and palette commands that act on a page
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NewKeybind, window, cx| {
                    this.navigate_to(ActivePage::Keybinds, window, cx);
                    if let Some(view) = &this.keybinds_view {
                        view.update(cx, |view, cx| view.open_add(window, cx));
                    }
                },
            ))
            .on_action(
                cx.listener(|this, _: &crate::ui::menu::app_menu::NewFlow, window, cx| {
                    this.navigate_to(ActivePage::FlowNew(None), window, cx);
                }),
            )
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NewFlowFromTemplate, window, cx| {
                    this.navigate_to(ActivePage::FlowTemplates, window, cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::OpenGallery, window, cx| {
                    if !dialog_open(window, cx) {
                        this.navigate_to(ActivePage::FlowGallery, window, cx);
                    }
                },
            ))
            .on_action(
                cx.listener(|_, _: &crate::ui::menu::app_menu::ImportFlow, window, cx| {
                    if dialog_open(window, cx) {
                        return;
                    }
                    import_flow_from_dialog(window, cx);
                }),
            )
            .on_action(cx.listener(
                |this, action: &crate::ui::menu::app_menu::RunFlow, window, cx| {
                    this.run_flow(action.0.clone(), window, cx);
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
                if dialog_open(window, cx) {
                    return;
                }
                crate::ui::dialogs::shortcuts_dialog::open_shortcuts_dialog(window, cx);
            }))
            .on_action(cx.listener(|this, _: &focus::ShowCommands, window, cx| {
                if dialog_open(window, cx) {
                    return;
                }
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
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.toggle_sidebar(window, cx);
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
    }
}
