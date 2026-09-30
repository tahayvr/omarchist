use crate::system::themes::overrides::Category;
use crate::system::themes::overrides::entries::Entry;
use crate::system::themes::theme_management::load_theme_for_editing;
use crate::types::themes::EditingTheme;
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::focus::{self, tab_strip_container};
use crate::ui::menu::app_menu;
use crate::ui::theme_apply::apply_theme;
use crate::ui::theme_edit_page::backgrounds_tab::BackgroundsTab;
use crate::ui::theme_edit_page::colors_tab::ColorsTab;
use crate::ui::theme_edit_page::general_tab::{GeneralTab, GeneralTabEvent};
use crate::ui::theme_edit_page::override_tab::OverrideTab;
use crate::ui::theme_edit_page::shared::error_message;
use gpui::*;
use gpui_component::{
    ActiveTheme,
    button::Button,
    h_flex,
    tab::{Tab, TabBar},
    v_flex,
};

const KEY_CONTEXT: &str = "ThemeEditPage";

// Tab order of the Theme Designer: what every theme needs, then the
// optional per-app files grouped by category. UI-only, so it lives with the
// page rather than in the shared theme data types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeEditTab {
    General,
    Colors,
    Backgrounds,
    Optional(Category),
}

impl ThemeEditTab {
    /// How many tabs come before the optional ones.
    pub const CORE_COUNT: usize = 3;

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeEditTab::General => "General",
            ThemeEditTab::Colors => "Colors",
            ThemeEditTab::Backgrounds => "Backgrounds",
            ThemeEditTab::Optional(category) => category.label(),
        }
    }

    pub fn all() -> Vec<ThemeEditTab> {
        let mut tabs = vec![
            ThemeEditTab::General,
            ThemeEditTab::Colors,
            ThemeEditTab::Backgrounds,
        ];
        tabs.extend(Category::all().map(ThemeEditTab::Optional));
        tabs
    }
}

actions!(theme_edit, [ApplyTheme]);

pub struct ThemeEditPage {
    theme_name: String,
    active_tab: usize,
    tab_count: usize,
    error_message: Option<String>,
    general_tab: Entity<GeneralTab>,
    colors_tab: Entity<ColorsTab>,
    backgrounds_tab: Entity<BackgroundsTab>,
    /// One per `Category`, in `Category::all()` order.
    override_tabs: Vec<(Category, Entity<OverrideTab>)>,
    pub focus_handle: FocusHandle,
    /// The tab strip is one tab stop; left/right switch tabs.
    tabs_focus: FocusHandle,
    /// Non-tab-stop handle on the scrolling content, so `focus_first_in`
    /// can land on the active tab's first field.
    content_focus: FocusHandle,
    scroll: ScrollHandle,
}

impl ThemeEditPage {
    pub fn new(theme_name: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Only Omarchist's own themes reach the editor (`navigate_to` gates
        // it), so the theme is never one of Omarchy's, whatever it is named.
        let is_system = false;

        // A theme whose manifest cannot be read opens with the stock
        // palette; the tabs say so rather than passing it off as the
        // theme's colors.
        let (theme_data, load_error) = match load_theme_for_editing(&theme_name) {
            Ok(data) => (data, None),
            Err(e) => (
                EditingTheme::default(),
                Some(format!(
                    "'{theme_name}' could not be read, so these are default colors, not \
                     the theme's: {e}"
                )),
            ),
        };

        // Shared by every tab so focused sections can scroll into view.
        let scroll = ScrollHandle::new();

        let general_tab = cx
            .new(|cx| GeneralTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx));
        // Reopening the page under the new name rebuilds every tab from disk.
        cx.subscribe_in(
            &general_tab,
            window,
            |this, _, event: &GeneralTabEvent, window, cx| match event {
                GeneralTabEvent::Renamed(name) => {
                    emit(cx, AppEvent::Navigate(ActivePage::ThemeEdit(name.clone())));
                }
                GeneralTabEvent::OpenOverride(entry) => this.open_override(*entry, window, cx),
            },
        )
        .detach();
        let colors_tab = cx
            .new(|cx| ColorsTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx));
        let backgrounds_tab =
            cx.new(|cx| BackgroundsTab::new(theme_name.clone(), is_system, &scroll, window, cx));
        let override_tabs = Category::all()
            .into_iter()
            .map(|category| {
                let tab = cx.new(|cx| OverrideTab::new(theme_name.clone(), category, window, cx));
                (category, tab)
            })
            .collect();

        let tab_count = ThemeEditTab::all().len();

        let focus_handle = cx.focus_handle();
        let tabs_focus = focus::tab_stop(cx);
        tabs_focus.focus(window, cx);

        Self {
            theme_name,
            active_tab: 0,
            tab_count,
            error_message: load_error,
            general_tab,
            colors_tab,
            backgrounds_tab,
            override_tabs,
            focus_handle,
            tabs_focus,
            content_focus: cx.focus_handle(),
            scroll,
        }
    }

    /// Focuses the tab strip, the page's first control after Back/Apply.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.tabs_focus.focus(window, cx);
    }

    fn apply_theme(&self, window: &mut Window, cx: &mut App) {
        // A change made within the save delay must be on disk before Omarchy
        // stages the theme.
        self.flush_pending_saves(cx);
        apply_theme(self.theme_name.clone(), window, cx);
    }

    pub fn flush_pending_saves(&self, cx: &mut App) {
        self.colors_tab.update(cx, |tab, cx| tab.flush(cx));
        for (_, tab) in &self.override_tabs {
            tab.update(cx, |tab, cx| tab.flush(cx));
        }
    }

    fn set_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let index = index.min(self.tab_count.saturating_sub(1));
        if self.active_tab != index {
            self.active_tab = index;
            self.scroll.set_offset(Point::default());
            match ThemeEditTab::all().get(index).copied() {
                Some(ThemeEditTab::Optional(category)) => {
                    if let Some((_, tab)) = self.override_tabs.iter().find(|(c, _)| *c == category)
                    {
                        tab.update(cx, |tab, cx| tab.refresh(window, cx));
                    }
                }
                Some(ThemeEditTab::General) => {
                    self.general_tab.update(cx, |tab, cx| tab.refresh(cx));
                }
                _ => {}
            }
            cx.notify();
        }
    }

    pub fn theme_name(&self) -> &str {
        &self.theme_name
    }

    fn navigate_back(&self, _window: &mut Window, cx: &mut Context<Self>) {
        // The page entity is dropped on the way out; a pending save with it.
        self.flush_pending_saves(cx);
        // Refresh first so a newly created theme is in the grid on arrival.
        emit(cx, AppEvent::RefreshThemes);
        emit(cx, AppEvent::Navigate(ActivePage::Themes));
    }

    /// Shows an override's pane on its category's tab.
    fn open_override(&mut self, entry: Entry, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = ThemeEditTab::all()
            .iter()
            .position(|tab| *tab == ThemeEditTab::Optional(entry.category()))
        else {
            return;
        };
        if let Some((_, tab)) = self
            .override_tabs
            .iter()
            .find(|(c, _)| *c == entry.category())
        {
            tab.update(cx, |tab, cx| tab.select(entry.id(), window, cx));
        }
        self.set_tab(index, window, cx);
    }

    fn next_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_tab(self.active_tab + 1, window, cx);
    }

    fn prev_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_tab(self.active_tab.saturating_sub(1), window, cx);
    }

    fn render_tab_content(&self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
        let tabs = ThemeEditTab::all();
        let active_tab = tabs
            .get(self.active_tab)
            .copied()
            .unwrap_or(ThemeEditTab::General);

        match active_tab {
            ThemeEditTab::General => self.general_tab.clone().into_any_element(),
            ThemeEditTab::Colors => self.colors_tab.clone().into_any_element(),
            ThemeEditTab::Backgrounds => self.backgrounds_tab.clone().into_any_element(),
            ThemeEditTab::Optional(category) => self
                .override_tabs
                .iter()
                .find(|(c, _)| *c == category)
                .map(|(_, tab)| tab.clone().into_any_element())
                .unwrap_or_else(|| div().into_any_element()),
        }
    }
}

impl Render for ThemeEditPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let tabs = ThemeEditTab::all();
        let (core, optional) = tabs.split_at(ThemeEditTab::CORE_COUNT);

        v_flex()
            .id("theme-edit-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(theme.background)
            .gap_4()
            .overflow_x_hidden()
            .on_action(
                cx.listener(|this, _: &app_menu::ThemeEditNextTab, window, cx| {
                    this.next_tab(window, cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &app_menu::ThemeEditPrevTab, window, cx| {
                    this.prev_tab(window, cx);
                }),
            )
            .on_action(cx.listener(|this, _: &app_menu::NavigateBack, window, cx| {
                this.navigate_back(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ApplyTheme, window, cx| {
                this.apply_theme(window, cx);
            }))
            .child(
                h_flex()
                    .gap_4()
                    .items_center()
                    .flex_wrap()
                    .child(
                        Button::new("back-btn")
                            .label("Back")
                            .compact()
                            .tooltip_with_action(
                                "Back to Themes",
                                &app_menu::NavigateBack,
                                Some(KEY_CONTEXT),
                            )
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_back(window, cx);
                            })),
                    )
                    .child(
                        Button::new("apply-theme-btn")
                            .label("Apply Theme")
                            .compact()
                            .tooltip_with_action(
                                "Apply this theme now",
                                &ApplyTheme,
                                Some(KEY_CONTEXT),
                            )
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.apply_theme(window, cx)),
                            ),
                    )
                    .child(
                        tab_strip_container("theme-edit-tabs-strip", &self.tabs_focus, window, cx)
                            .flex_1()
                            .min_w_0()
                            .on_action(cx.listener(
                                |this, _: &focus::tab_strip::Prev, window, cx| {
                                    this.prev_tab(window, cx);
                                },
                            ))
                            .on_action(cx.listener(
                                |this, _: &focus::tab_strip::Next, window, cx| {
                                    this.next_tab(window, cx);
                                },
                            ))
                            .on_action(cx.listener(
                                |this, _: &focus::tab_strip::First, window, cx| {
                                    this.set_tab(0, window, cx);
                                },
                            ))
                            .on_action(cx.listener(
                                |this, _: &focus::tab_strip::Last, window, cx| {
                                    this.set_tab(usize::MAX, window, cx);
                                },
                            ))
                            .on_action(cx.listener(
                                |this, _: &focus::tab_strip::Activate, window, cx| {
                                    focus::focus_first_in(&this.content_focus, window, cx);
                                },
                            ))
                            .child(
                                h_flex()
                                    .gap_3()
                                    .flex_wrap()
                                    .items_center()
                                    .child(
                                        TabBar::new("theme-edit-tabs")
                                            .cursor_pointer()
                                            .selected_index(self.active_tab)
                                            .on_click(cx.listener(|view, index, window, cx| {
                                                view.set_tab(*index, window, cx);
                                            }))
                                            .children(
                                                core.iter()
                                                    .map(|tab| Tab::new().label(tab.as_str())),
                                            ),
                                    )
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(div().w_px().h_5().bg(theme.border))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.muted_foreground)
                                                    .child("Optional"),
                                            ),
                                    )
                                    .child(
                                        TabBar::new("theme-edit-optional-tabs")
                                            .cursor_pointer()
                                            .selected_index(
                                                self.active_tab
                                                    .wrapping_sub(ThemeEditTab::CORE_COUNT),
                                            )
                                            .on_click(cx.listener(|view, index, window, cx| {
                                                view.set_tab(
                                                    ThemeEditTab::CORE_COUNT + *index,
                                                    window,
                                                    cx,
                                                );
                                            }))
                                            .children(
                                                optional
                                                    .iter()
                                                    .map(|tab| Tab::new().label(tab.as_str())),
                                            ),
                                    ),
                            ),
                    ),
            )
            .children(
                self.error_message
                    .as_ref()
                    .map(|error| error_message(error.clone(), cx)),
            )
            .child(
                div()
                    .id("tab-content")
                    .track_focus(&self.content_focus)
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .pt_4()
                    .pb_8()
                    .child(self.render_tab_content(window, cx)),
            )
    }
}
