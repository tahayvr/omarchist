// Keycap-style chips for a chord: [Super] + [Shift] + [K].
//
// gpui-component's `Kbd` labels the platform modifier "Win" on Linux, which
// is wrong for Hyprland, so chords get their own renderer.
use gpui::*;
use gpui_component::{ActiveTheme, h_flex};

use crate::system::keybinds::chord::{Chord, ModMask};
use crate::system::keybinds::keymap::chord_display_parts;

pub fn chord_chips(chord: &Chord, muted: bool, cx: &App) -> AnyElement {
    chips(chord_display_parts(chord), muted, cx)
}

/// Chips for modifiers only, used while the recorder waits for a key.
pub fn modifier_chips(mods: ModMask, cx: &App) -> AnyElement {
    let labels = mods
        .names()
        .into_iter()
        .map(|name| match name {
            "SUPER" => "Super".to_string(),
            "SHIFT" => "Shift".to_string(),
            "CTRL" => "Ctrl".to_string(),
            "ALT" => "Alt".to_string(),
            other => other.to_string(),
        })
        .collect();
    chips(labels, true, cx)
}

fn chips(labels: Vec<String>, muted: bool, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let text_color = if muted {
        theme.muted_foreground
    } else {
        theme.foreground
    };
    let count = labels.len();

    h_flex()
        .gap_1()
        .items_center()
        .flex_wrap()
        .children(labels.into_iter().enumerate().flat_map(|(ix, label)| {
            let chip = div()
                .px_1p5()
                .py_0p5()
                .min_w(px(22.))
                .text_center()
                .rounded(theme.radius)
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(text_color)
                .child(label)
                .into_any_element();
            let separator = (ix + 1 < count).then(|| {
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("+")
                    .into_any_element()
            });
            std::iter::once(chip).chain(separator)
        }))
        .into_any_element()
}
