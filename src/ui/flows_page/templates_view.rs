//! The Templates page: every built-in and user template as a card. Picking
//! one opens the editor on a new flow made from it.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, button::Button, h_flex, v_flex};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::Flow;
use crate::system::flows::templates::{Template, templates, user_templates_dir};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::flows_page::flow_card::template_card;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::focus;
use crate::ui::menu::app_menu;

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
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .gap_4()
            .flex_wrap()
            .items_stretch()
            .children(templates.iter().enumerate().map(|(ix, template)| {
                let key = template.key.clone();
                template_card((group, ix), &template.flow, summaries, cx)
                    .on_click(cx.listener(move |this, _, _, cx| this.use_template(&key, cx)))
            }))
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &self.no_flows,
        };
        let built_in: Vec<&Template> = self.templates.iter().filter(|t| t.is_built_in()).collect();
        let user: Vec<&Template> = self.templates.iter().filter(|t| !t.is_built_in()).collect();
        let user_dir = user_templates_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        let user_section: AnyElement = if user.is_empty() {
            div()
                .text_sm()
                .text_color(muted)
                .child(format!(
                    "No templates of your own yet. Choose \"Save as template\" from a flow's menu in \
                     the editor, or put a .flow.toml file in {user_dir}."
                ))
                .into_any_element()
        } else {
            self.render_cards("user-template", &user, &summaries, cx)
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
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .child("Pick one to start a new flow from it."),
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
                                .child("Loading templates…"),
                        )
                    })
                    .when(self.loaded, |this| {
                        this.child(
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
                                            cx,
                                        )),
                                )
                                .child(
                                    v_flex()
                                        .gap_3()
                                        .child(self.render_group_label("YOURS", cx))
                                        .child(user_section),
                                ),
                        )
                    }),
            )
    }
}
