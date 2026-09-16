// The keyboard shortcuts help dialog, generated from the shortcut table.
use gpui::*;
use gpui_component::{ActiveTheme, WindowExt, h_flex, scroll::ScrollableElement, v_flex};

use crate::ui::focus;
use crate::ui::shortcuts;

/// Renders a keystroke such as `ctrl-shift-r` as keycaps.
fn key_caps(keys: &str, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let mut parts: Vec<String> = Vec::new();
    for stroke in keys.split_whitespace() {
        let Ok(keystroke) = Keystroke::parse(stroke) else {
            parts.push(stroke.to_string());
            continue;
        };
        let mods = keystroke.modifiers;
        if mods.control {
            parts.push("Ctrl".into());
        }
        if mods.alt {
            parts.push("Alt".into());
        }
        if mods.shift {
            parts.push("Shift".into());
        }
        if mods.platform {
            parts.push("Super".into());
        }
        parts.push(key_label(&keystroke.key));
    }

    let mut row = h_flex().gap_1().items_center();
    for (ix, part) in parts.into_iter().enumerate() {
        if ix > 0 {
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("+"),
            );
        }
        row = row.child(
            div()
                .px_1p5()
                .py_0p5()
                .rounded(theme.radius)
                .border_1()
                .border_color(theme.border)
                .bg(theme.secondary)
                .text_xs()
                .font_family("JetBrainsMono Nerd Font Mono")
                .child(part),
        );
    }
    row.into_any_element()
}

fn key_label(key: &str) -> String {
    match key {
        "enter" => "Enter".into(),
        "escape" => "Esc".into(),
        "tab" => "Tab".into(),
        "space" => "Space".into(),
        "backspace" => "Backspace".into(),
        "delete" => "Del".into(),
        "up" => "↑".into(),
        "down" => "↓".into(),
        "left" => "←".into(),
        "right" => "→".into(),
        "home" => "Home".into(),
        "end" => "End".into(),
        "pageup" => "PgUp".into(),
        "pagedown" => "PgDn".into(),
        other => other.to_uppercase(),
    }
}

pub fn open_shortcuts_dialog(window: &mut Window, cx: &mut App) {
    let body_focus = cx.focus_handle();
    let trap_focus = body_focus.clone();
    window.open_dialog(cx, move |dialog, _, cx| {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let border = theme.border;

        let groups = shortcuts::help_rows().into_iter().map(|(group, rows)| {
            v_flex()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(muted)
                        .child(group.to_uppercase()),
                )
                .children(rows.into_iter().map(|(label, keys)| {
                    h_flex()
                        .justify_between()
                        .items_center()
                        .gap_4()
                        .py_0p5()
                        .child(div().text_sm().child(label))
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .children(keys.iter().enumerate().flat_map(|(ix, keys)| {
                                    let mut items = Vec::new();
                                    if ix > 0 {
                                        items.push(
                                            div().text_xs().text_color(muted).child("or").into_any_element(),
                                        );
                                    }
                                    items.push(key_caps(keys, cx));
                                    items
                                })),
                        )
                })).into_any_element()
        }).collect::<Vec<_>>();

        dialog
            .title("Keyboard Shortcuts")
            .w(px(720.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(true)
            .child(
                focus::dialog_body("shortcuts-dialog", &trap_focus, |window, cx| {
                    window.close_dialog(cx);
                })
                .child(
                    v_flex()
                        .id("shortcuts-list")
                        .h(px(520.))
                        .overflow_y_scrollbar()
                        .gap_5()
                        .pr_3()
                        .children(groups)
                        .child(
                            div()
                                .pt_2()
                                .border_t_1()
                                .border_color(border)
                                .text_xs()
                                .text_color(muted)
                                .child("Tab and Shift+Tab move between controls everywhere; Enter or Space activates the focused control."),
                        ),
                ),
            )
    });
    focus::focus_first_in(&body_focus, window, cx);
}
