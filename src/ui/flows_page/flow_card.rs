//! Pieces shared by the flow cards and the editor: the icon tile, the
//! trigger chips, and the strip of step icons.
use gpui::*;
use gpui_component::{ActiveTheme, Icon, Sizable, h_flex, tag::Tag};

use crate::system::flows::{Flow, icon_path};
use crate::system::keybinds::chord::Chord;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::keybinds_page::chord_chips::chord_chips;

/// The flow's icon on a tinted square.
pub fn icon_tile(icon: &str, size: Pixels, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .size(size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(theme.radius)
        .bg(theme.primary.opacity(0.12))
        .text_color(theme.primary)
        .child(
            Icon::new(Icon::empty())
                .path(icon_path(icon))
                .size(size * 0.5),
        )
}

/// How the flow can be started: its chord, launcher, and startup tags, or
/// a note that only the command runs it.
pub fn trigger_chips(flow: &Flow, chord: Option<&Chord>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let mut row = h_flex().gap_2().flex_wrap().items_center();
    let mut any = false;
    if let Some(chord) = chord {
        row = row.child(chord_chips(chord, false, cx));
        any = true;
    }
    if flow.triggers.launcher {
        row = row.child(Tag::secondary().small().child("App launcher"));
        any = true;
    }
    if flow.triggers.startup {
        row = row.child(Tag::secondary().small().child("At startup"));
        any = true;
    }
    if !any {
        row = row.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Runs from the command line"),
        );
    }
    row
}

/// The steps as a row of small icons joined by chevrons, so a card shows
/// the shape of the flow at a glance.
pub fn step_strip(flow: &Flow, summaries: &SummaryContext, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let mut row = h_flex().gap_1().items_center().flex_wrap();
    let count = flow.steps.len();
    for (ix, step) in flow.steps.iter().enumerate() {
        let summary = summaries.summarize(&step.kind);
        row = row.child(
            div()
                .size_6()
                .flex()
                .items_center()
                .justify_center()
                .rounded(theme.radius)
                .bg(theme.secondary)
                .text_color(if step.enabled {
                    theme.foreground
                } else {
                    theme.muted_foreground
                })
                .opacity(if step.enabled { 1. } else { 0.5 })
                .child(summary.icon.render(px(14.))),
        );
        if ix + 1 < count {
            row = row.child(
                Icon::new(Icon::empty())
                    .path("icons/chevron-right.svg")
                    .size_3()
                    .text_color(theme.muted_foreground),
            );
        }
    }
    if count == 0 {
        row = row.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("No steps yet"),
        );
    }
    row
}

pub fn step_count_label(flow: &Flow) -> String {
    let n = flow.steps.len();
    let enabled = flow.enabled_steps();
    let mut label = format!("{n} step{}", if n == 1 { "" } else { "s" });
    if enabled < n {
        label.push_str(&format!(", {} off", n - enabled));
    }
    label
}
