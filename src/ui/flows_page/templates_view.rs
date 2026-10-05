//! The Templates page: every built-in and user template as a card. Picking
//! one opens the editor on a new flow made from it.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, button::Button, h_flex, v_flex};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::Flow;
use crate::system::flows::templates::{Template, templates};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::flows_page::flow_card::template_card;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::focus;
use crate::ui::menu::app_menu;
use crate::ui::text::selectable;

const KEY_CONTEXT: &str = "FlowTemplatesPage";

pub struct TemplatesView {
    pub focus_handle: FocusHandle,
    templates: Vec<Template>,
    apps: Vec<DesktopApp>,
    /// Step summaries can name other flows; templates never do.
    no_flows: Vec<Flow>,
    loaded: bool,
    scroll: ScrollHandle,
}

impl TemplatesView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            templates: Vec::new(),
            apps: Vec::new(),
            no_flows: Vec::new(),
            loaded: false,
            scroll: ScrollHandle::new(),
        };
        view.refresh(cx);
        view
    }

    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        focus::focus_first_in(&self.focus_handle, window, cx);
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
                // Each cell, padding included, takes an equal share; a card
                // sized by its own text would widen a short row.
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .child(
                        template_card((group, ix), &template.flow, summaries, cx)
                            .min_w_0()
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.use_template(&key, cx)),
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

    fn render_group_label(&self, text: &'static str, cx: &App) -> impl IntoElement {
        div()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child(text)
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
        let built_in: Vec<&Template> = self.templates.iter().filter(|t| t.is_built_in()).collect();
        let user: Vec<&Template> = self.templates.iter().filter(|t| !t.is_built_in()).collect();

        let user_section: AnyElement = if user.is_empty() {
            div()
                .text_sm()
                .text_color(muted)
                .child(selectable(
                    "no-user-templates",
                    "No templates of your own yet",
                ))
                .into_any_element()
        } else {
            self.render_cards("user-template", &user, &summaries, columns, cx)
                .into_any_element()
        };

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
                    .child(div().text_xs().text_color(muted).child(selectable(
                        "templates-note",
                        "Pick one to start a new flow from it.",
                    ))),
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
                    .when(self.loaded, |this| {
                        this.child(
                            crate::ui::focus::scroll_area(&self.scroll).child(
                                v_flex()
                                    .gap_8()
                                    .max_w(px(1200.))
                                    .child(
                                        v_flex()
                                            .gap_3()
                                            .child(self.render_group_label("BUILT IN", cx))
                                            .child(self.render_cards(
                                                "built-in-template",
                                                &built_in,
                                                &summaries,
                                                columns,
                                                cx,
                                            )),
                                    )
                                    .child(
                                        v_flex()
                                            .gap_3()
                                            .child(self.render_group_label("YOURS", cx))
                                            .child(user_section),
                                    ),
                            ),
                        )
                    }),
            )
    }
}
