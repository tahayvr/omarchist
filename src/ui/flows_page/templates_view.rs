//! The Templates page: the user's own templates and the built-in ones as
//! cards. Picking one opens the editor on a new flow made from it.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::Flow;
use crate::system::flows::templates::{Template, delete_user_template, templates};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::flows_page::flow_card::template_card;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::focus;
use crate::ui::menu::app_menu;
use crate::ui::text::selectable;

const KEY_CONTEXT: &str = "FlowTemplatesPage";
/// Wraps the search box so Escape clears it before it leaves the page.
pub const SEARCH_CONTEXT: &str = "TemplatesSearch";
/// Wraps one of the user's templates: Delete removes it.
pub const USER_TEMPLATE_CONTEXT: &str = "UserTemplate";

pub mod templates_nav {
    gpui::actions!(flow_templates, [ClearSearch, DeleteTemplate]);
}
use templates_nav::*;

pub struct TemplatesView {
    pub focus_handle: FocusHandle,
    search: Entity<InputState>,
    query: String,
    templates: Vec<Template>,
    apps: Vec<DesktopApp>,
    /// Step summaries can name other flows; templates never do.
    no_flows: Vec<Flow>,
    loaded: bool,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl TemplatesView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search templates"));
        let subscriptions = vec![
            cx.subscribe(&search, |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.query = input.read(cx).value().to_string();
                    cx.notify();
                }
            }),
        ];
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            search,
            query: String::new(),
            templates: Vec::new(),
            apps: Vec::new(),
            no_flows: Vec::new(),
            loaded: false,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        view.refresh(cx);
        view
    }

    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }

    /// Reloads the templates and the installed apps (for step icons) off
    /// the UI thread.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async { (templates(), installed_apps()) })
                .await;
            this.update(cx, |this, cx| {
                let (templates, apps) = loaded;
                this.templates = templates;
                this.apps = apps;
                this.loaded = true;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn use_template(&self, key: &str, cx: &mut Context<Self>) {
        emit(
            cx,
            AppEvent::Navigate(ActivePage::FlowNew(Some(key.to_string()))),
        );
    }

    fn back(&self, cx: &mut Context<Self>) {
        emit(cx, AppEvent::Navigate(ActivePage::Flows));
    }

    /// Escape in the search box: an empty box leaves the page.
    fn clear_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.query.is_empty() {
            self.back(cx);
            return;
        }
        // Setting the value tells nobody, so the query follows by hand.
        self.search
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.query.clear();
        cx.notify();
    }

    fn matches(&self, template: &Template) -> bool {
        let query = self.query.trim().to_lowercase();
        query.is_empty()
            || template.flow.name.to_lowercase().contains(&query)
            || template.flow.description.to_lowercase().contains(&query)
    }

    fn confirm_delete(&mut self, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(template) = self.templates.iter().find(|t| t.key == key) else {
            return;
        };
        let view = cx.entity();
        let key = key.to_string();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Delete this template?",
                message: format!(
                    "Delete the template '{}'? Flows made from it stay.",
                    template.flow.name
                ),
                confirm_label: "Delete",
                danger: true,
            },
            move |window, cx| {
                view.update(cx, |this, cx| {
                    match delete_user_template(&key) {
                        Ok(()) => this.templates.retain(|t| t.key != key),
                        Err(e) => window
                            .push_notification(format!("Could not delete the template: {e}"), cx),
                    }
                    // The card that had the keyboard is gone, and closing
                    // the dialog would hand the keyboard back to it.
                    cx.defer_in(window, |this, window, cx| this.focus_entry(window, cx));
                    cx.notify();
                });
            },
            window,
            cx,
        );
    }

    fn render_cards(
        &self,
        group: &'static str,
        templates: &[&Template],
        summaries: &SummaryContext,
        columns: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // Rows of equal cells, the last one padded, so a section with one
        // or two templates keeps the same card width as a full one.
        let columns = columns.max(1);
        let mut cards: Vec<AnyElement> = templates
            .iter()
            .enumerate()
            .map(|(ix, template)| {
                let key = template.key.clone();
                let card = template_card((group, ix), &template.flow, summaries, cx)
                    .min_w_0()
                    .on_click(cx.listener(move |this, _, _, cx| this.use_template(&key, cx)));
                // Each cell, padding included, takes an equal share; a card
                // sized by its own text would widen a short row.
                let cell = div().relative().flex_1().min_w_0().flex();
                if template.is_built_in() {
                    return cell.child(card).into_any_element();
                }
                let (delete_key, action_key) = (template.key.clone(), template.key.clone());
                cell.key_context(USER_TEMPLATE_CONTEXT)
                    .on_action(cx.listener(move |this, _: &DeleteTemplate, window, cx| {
                        this.confirm_delete(&action_key, window, cx)
                    }))
                    .child(card)
                    .child(
                        div().absolute().top_2().right_2().child(
                            Button::new(("template-delete", ix))
                                .ghost()
                                .xsmall()
                                .tab_stop(false)
                                .icon(Icon::new(Icon::empty()).path("icons/trash.svg"))
                                .tooltip_with_action(
                                    "Delete template",
                                    &DeleteTemplate,
                                    Some(USER_TEMPLATE_CONTEXT),
                                )
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.confirm_delete(&delete_key, window, cx);
                                })),
                        ),
                    )
                    .into_any_element()
            })
            .collect();
        while !cards.len().is_multiple_of(columns) {
            cards.push(div().flex_1().min_w_0().into_any_element());
        }
        let mut rows = Vec::new();
        let mut cards = cards.into_iter();
        loop {
            let row: Vec<AnyElement> = cards.by_ref().take(columns).collect();
            if row.is_empty() {
                break;
            }
            rows.push(h_flex().gap_4().items_stretch().children(row));
        }
        v_flex().gap_4().children(rows)
    }

    fn render_group(
        &self,
        label: &'static str,
        group: &'static str,
        templates: &[&Template],
        summaries: &SummaryContext,
        columns: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(self.render_cards(group, templates, summaries, columns, cx))
    }
}

impl Render for TemplatesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let width = window.viewport_size().width;
        let columns = if width < px(700.) {
            1
        } else if width < px(1100.) {
            2
        } else {
            3
        };
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &self.no_flows,
        };
        let shown = |built_in: bool| -> Vec<&Template> {
            self.templates
                .iter()
                .filter(|t| t.is_built_in() == built_in && self.matches(t))
                .collect()
        };
        let (user, built_in) = (shown(false), shown(true));
        let nothing = self.loaded && user.is_empty() && built_in.is_empty();

        v_flex()
            .id("templates-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &app_menu::NavigateBack, _, cx| this.back(cx)))
            .size_full()
            .gap_6()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .flex_wrap()
                    .child(
                        Button::new("templates-back")
                            .label("Back")
                            .compact()
                            .tooltip_with_action(
                                "Back to Flows",
                                &app_menu::NavigateBack,
                                Some(KEY_CONTEXT),
                            )
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.back(cx))),
                    )
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Templates"))
                    .child(
                        div()
                            .key_context(SEARCH_CONTEXT)
                            .on_action(cx.listener(|this, _: &ClearSearch, window, cx| {
                                this.clear_search(window, cx)
                            }))
                            .flex_1()
                            .min_w(px(200.))
                            .max_w(px(420.))
                            .child(
                                Input::new(&self.search)
                                    .id("templates-search")
                                    .small()
                                    .cleanable(true)
                                    .prefix(
                                        Icon::new(Icon::empty())
                                            .path("icons/search.svg")
                                            .size_4()
                                            .text_color(muted),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .id("templates-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .pb_8()
                    .when(!self.loaded, |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(muted)
                                .child(selectable("templates-loading", "Loading templates…")),
                        )
                    })
                    .when(nothing, |this| {
                        this.child(
                            div()
                                .id("templates-none")
                                .test_support()
                                .text_sm()
                                .text_color(muted)
                                .child(selectable("templates-no-match", "No template matches")),
                        )
                    })
                    .when(self.loaded && !nothing, |this| {
                        this.child(
                            focus::scroll_area(&self.scroll).child(
                                v_flex()
                                    .gap_8()
                                    .max_w(px(1200.))
                                    // The user's own come first: they are
                                    // the ones made for this machine.
                                    .when(!user.is_empty(), |this| {
                                        this.child(self.render_group(
                                            "YOURS",
                                            "user-template",
                                            &user,
                                            &summaries,
                                            columns,
                                            cx,
                                        ))
                                    })
                                    .when(!built_in.is_empty(), |this| {
                                        this.child(self.render_group(
                                            "BUILT IN",
                                            "built-in-template",
                                            &built_in,
                                            &summaries,
                                            columns,
                                            cx,
                                        ))
                                    }),
                            ),
                        )
                    }),
            )
    }
}
