use gpui::FontWeight;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, Sizable, WindowExt, button::*, h_flex, v_flex};

use gpui_base::TestSupportExt;

use crate::ui::text::selectable;

const KEY_CONTEXT: &str = "AboutView";
const ISSUES_URL: &str = "https://github.com/tahayvr/omarchist/issues/new";

/// What a bug report needs: the versions of Omarchist, Omarchy, and
/// Hyprland, and whether this is a Quattro install. Blocking on two short
/// commands, so it is gathered when the button is pressed.
pub fn debug_info() -> String {
    let omarchy = crate::system::omarchy::updates::installed_version()
        .unwrap_or_else(|| "not found".to_string());
    let hyprland = std::process::Command::new("hyprctl")
        .args(["-j", "version"])
        .output()
        .ok()
        .and_then(|out| serde_json::from_slice::<serde_json::Value>(&out.stdout).ok())
        .and_then(|v| v["tag"].as_str().map(str::to_string))
        .unwrap_or_else(|| "not running".to_string());
    format!(
        "Omarchist {}\nOmarchy {omarchy}{}\nHyprland {hyprland}\nOMARCHY_PATH {}",
        env!("CARGO_PKG_VERSION"),
        if crate::system::omarchy_paths::is_quattro_installed() {
            ""
        } else {
            " (not Quattro)"
        },
        crate::system::omarchy_paths::omarchy_install_dir().display(),
    )
}

pub struct AboutView {
    pub focus_handle: FocusHandle,
}

impl AboutView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    /// Focuses the first link button.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        crate::ui::focus::focus_first_in(&self.focus_handle, window, cx);
    }
}

impl Render for AboutView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .id("about-view")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .gap_0()
            .size_full()
            .items_center()
            .justify_center()
            .text_center()
            .child(
                img("logo/omarchist.png")
                    .w(px(128.))
                    .h(px(128.))
                    .object_fit(ObjectFit::Contain),
            )
            .child(
                v_flex()
                    .mt_6()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(42.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.foreground)
                            .line_height(relative(1.0))
                            .child(selectable("app-name", "OMARCHIST")),
                    )
                    .child(
                        div()
                            .id("omarchist-version")
                            .test_support()
                            .text_color(theme.muted_foreground)
                            .child(selectable(
                                "version",
                                concat!("v", env!("CARGO_PKG_VERSION")),
                            )),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(selectable(
                                "license",
                                "Apache-2.0 · © 2026 Taha Hossein Nejad",
                            )),
                    ),
            )
            .child(
                h_flex()
                    .mt_8()
                    .gap_2()
                    .flex_wrap()
                    .justify_center()
                    .child(
                        Button::new("report-issue")
                            .label("Report an issue")
                            .outline()
                            .cursor_pointer()
                            .on_click(|_, _, cx| cx.open_url(ISSUES_URL)),
                    )
                    .child(
                        Button::new("copy-debug-info")
                            .label("Copy debug info")
                            .outline()
                            .cursor_pointer()
                            .tooltip("Versions to paste into a bug report")
                            .on_click(|_, window, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(debug_info()));
                                window.push_notification("Debug info copied", cx);
                            }),
                    ),
            )
            .child(
                h_flex()
                    .mt_12()
                    .gap_4()
                    .child(
                        Button::new("x-com")
                            .icon(Icon::new(Icon::empty()).path("icons/x.svg").size_8())
                            .ghost()
                            .cursor_pointer()
                            .large()
                            .tooltip("X")
                            .on_click(|_, _, cx| cx.open_url("https://x.com/tahayvr/")),
                    )
                    .child(
                        Button::new("github")
                            .icon(Icon::new(Icon::empty()).path("icons/github.svg").size_8())
                            .ghost()
                            .cursor_pointer()
                            .large()
                            .tooltip("GitHub")
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/tahayvr/omarchist")
                            }),
                    )
                    .child(
                        Button::new("docs")
                            .label("Docs")
                            .ghost()
                            .cursor_pointer()
                            .large()
                            .on_click(|_, _, cx| cx.open_url("https://omarchist.com/")),
                    ),
            )
    }
}
