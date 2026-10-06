use crate::system::themes::theme_management::load_theme_for_editing;
use crate::types::themes::EditingTheme;
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::editable_title::{Title, TitleState};
use crate::ui::focus::{self, tab_strip_container};
use crate::ui::menu::app_menu;
use crate::ui::theme_apply::apply_theme;
use crate::ui::theme_edit_page::backgrounds_tab::BackgroundsTab;
use crate::ui::theme_edit_page::colors_tab::ColorsTab;
use crate::ui::theme_edit_page::general_tab::{GeneralTab, GeneralTabEvent};
use crate::ui::theme_edit_page::icons_tab::IconsTab;
use crate::ui::theme_edit_page::shared::error_message;
use gpui::*;
use gpui_component::{
    ActiveTheme,
    button::Button,
    h_flex,
    input::{InputEvent, InputState},
    tab::{Tab, TabBar},
    v_flex,
};

const KEY_CONTEXT: &str = "ThemeEditPage";

// Tab order of the Theme Designer. A theme is its colors.toml, its
// backgrounds, and its icon color; Omarchy generates every app's files from
// those. UI-only, so it lives with the page rather than in the shared theme
// data types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeEditTab {
    General,
    Colors,
    Backgrounds,
    Icons,
}

impl ThemeEditTab {
    pub const ALL: [ThemeEditTab; 4] = [
        ThemeEditTab::General,
        ThemeEditTab::Colors,
        ThemeEditTab::Backgrounds,
        ThemeEditTab::Icons,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeEditTab::General => "General",
            ThemeEditTab::Colors => "Colors",
            ThemeEditTab::Backgrounds => "Backgrounds",
            ThemeEditTab::Icons => "Icons",
        }
    }
}

actions!(theme_edit, [ApplyTheme, LeaveField]);

pub struct ThemeEditPage {
    theme_name: String,
    /// The name in the header, a title until it is clicked.
    title: TitleState,
    active_tab: usize,
    tab_count: usize,
    error_message: Option<String>,
    general_tab: Entity<GeneralTab>,
    colors_tab: Entity<ColorsTab>,
    backgrounds_tab: Entity<BackgroundsTab>,
    icons_tab: Entity<IconsTab>,
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

        let name_input = cx.new(|cx| InputState::new(window, cx).default_value(&theme_data.name));
        // Leaving the field ends the rename (Enter is a binding).
        cx.subscribe_in(
            &name_input,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Blur) && TitleState::blur_ends(window) {
                    this.finish_rename(window, cx);
                }
            },
        )
        .detach();
        let general_tab = cx
            .new(|cx| GeneralTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx));
        // Reopening the page under the new name rebuilds every tab from disk.
        cx.subscribe(
            &general_tab,
            |_, _, event: &GeneralTabEvent, cx| match event {
                GeneralTabEvent::Renamed(name) => {
                    emit(cx, AppEvent::Navigate(ActivePage::ThemeEdit(name.clone())));
                }
            },
        )
        .detach();
        let colors_tab = cx
            .new(|cx| ColorsTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx));
        let backgrounds_tab =
            cx.new(|cx| BackgroundsTab::new(theme_name.clone(), is_system, &scroll, window, cx));
        let icons_tab = cx.new(|cx| IconsTab::new(theme_name.clone(), cx));

        let tab_count = ThemeEditTab::ALL.len();

        let focus_handle = cx.focus_handle();
        let tabs_focus = focus::tab_stop(cx);
        tabs_focus.focus(window, cx);

        Self {
            theme_name,
            title: TitleState::new(name_input, cx),
            active_tab: 0,
            tab_count,
            error_message: load_error,
            general_tab,
            colors_tab,
            backgrounds_tab,
            icons_tab,
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

    fn start_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.title.start(&self.focus_handle, window, cx);
        cx.notify();
    }

    /// Keeps what was typed: the General tab renames the theme's folder,
    /// and the page reopens under the new name. An empty field changes
    /// nothing.
    fn finish_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.title.renaming {
            return;
        }
        self.title.stop(&self.focus_handle, window, cx);
        let name = self.title.input.read(cx).value().trim().to_string();
        if name.is_empty() {
            let current = self.general_tab.read(cx).theme_data().name.clone();
            self.title
                .input
                .update(cx, |input, cx| input.set_value(current, window, cx));
        } else {
            self.general_tab
                .update(cx, |tab, cx| tab.rename(&name, window, cx));
        }
        cx.notify();
    }

    fn apply_theme(&self, window: &mut Window, cx: &mut App) {
        // A change made within the save delay must be on disk before Omarchy
        // stages the theme.
        self.flush_pending_saves(cx);
        apply_theme(self.theme_name.clone(), window, cx);
    }

    pub fn flush_pending_saves(&self, cx: &mut App) {
        self.colors_tab.update(cx, |tab, cx| tab.flush(cx));
    }

    fn set_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let index = index.min(self.tab_count.saturating_sub(1));
        if self.active_tab != index {
            self.active_tab = index;
            self.scroll.set_offset(Point::default());
            if ThemeEditTab::ALL.get(index) == Some(&ThemeEditTab::Icons) {
                self.icons_tab.update(cx, |tab, cx| tab.refresh(cx));
            }
            let _ = window;
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

    fn next_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_tab(self.active_tab + 1, window, cx);
    }

    fn prev_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_tab(self.active_tab.saturating_sub(1), window, cx);
    }

    fn render_tab_content(&self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
        let active_tab = ThemeEditTab::ALL
            .get(self.active_tab)
            .copied()
            .unwrap_or(ThemeEditTab::General);

        match active_tab {
            ThemeEditTab::General => self.general_tab.clone().into_any_element(),
            ThemeEditTab::Colors => self.colors_tab.clone().into_any_element(),
            ThemeEditTab::Backgrounds => self.backgrounds_tab.clone().into_any_element(),
            ThemeEditTab::Icons => self.icons_tab.clone().into_any_element(),
        }
    }
}

impl Render for ThemeEditPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let on_start = {
            let view = view.clone();
            move |window: &mut Window, cx: &mut App| {
                view.update(cx, |this, cx| this.start_rename(window, cx))
            }
        };
        let on_stop = move |window: &mut Window, cx: &mut App| {
            view.update(cx, |this, cx| this.finish_rename(window, cx))
        };
        let name = self.general_tab.read(cx).theme_data().name.clone();
        let title = self.title.render(
            Title {
                id: "theme-title",
                field_id: "theme-name",
                text: &name,
                placeholder: "Untitled theme",
                on_start: Box::new(on_start),
                on_stop: Box::new(on_stop),
            },
            window,
            cx,
        );
        let theme = cx.theme();

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
            // Escape in a text field steps out to the tab strip; the
            // page's own Escape (Back to Themes) is one press further.
            .on_action(cx.listener(|this, _: &LeaveField, window, cx| {
                this.tabs_focus.focus(window, cx);
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
                    .child(title)
                    .child(
                        Button::new("apply-theme-btn")
                            .label("Apply theme")
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
                            // Wide enough for the four tabs, so in a narrow
                            // window the strip drops under the buttons
                            // instead of clipping its end.
                            .min_w(px(440.))
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
                                TabBar::new("theme-edit-tabs")
                                    .cursor_pointer()
                                    .selected_index(self.active_tab)
                                    .on_click(cx.listener(|view, index, window, cx| {
                                        view.set_tab(*index, window, cx);
                                    }))
                                    .children(
                                        ThemeEditTab::ALL
                                            .iter()
                                            .map(|tab| Tab::new().label(tab.as_str())),
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
                    .child(
                        focus::scroll_area(&self.scroll).child(self.render_tab_content(window, cx)),
                    ),
            )
    }
}
