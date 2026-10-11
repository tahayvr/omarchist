use gpui::*;
use gpui_component::{h_flex, v_flex};

use crate::ui::dashboard_page::flows_card::FlowsCard;
use crate::ui::focus;

const KEY_CONTEXT: &str = "DashboardView";

/// The page the app opens on: cards that each give a glance at one part of
/// the system, wrapping into fewer columns as the window narrows.
pub struct DashboardView {
    pub focus_handle: FocusHandle,
    flows: Entity<FlowsCard>,
}

impl DashboardView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            flows: cx.new(FlowsCard::new),
        }
    }

    /// Focuses the first control on the page.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        focus::focus_first_in(&self.focus_handle, window, cx);
    }

    /// Reads every card's data again: a visit shows what changed meanwhile.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.flows.update(cx, |card, cx| card.refresh(cx));
    }

    /// Starts the cards' polls; see `MainWindowView::watch_runs`.
    pub fn watch_runs(&self, cx: &mut Context<Self>) {
        self.flows.update(cx, |card, cx| card.watch_runs(cx));
    }
}

impl Render for DashboardView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("dashboard-view")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .child(
                h_flex().w_full().gap_4().flex_wrap().items_start().child(
                    div()
                        .flex_basis(px(520.))
                        .flex_grow(1.)
                        .min_w_0()
                        .child(self.flows.clone()),
                ),
            )
    }
}
