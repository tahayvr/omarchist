use gpui::FontWeight;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, button::*, h_flex, v_flex};

use crate::ui::menu::app_menu;

const KEY_CONTEXT: &str = "AboutView";

/// Focusable link buttons on the About page (in tab order)
const ABOUT_BUTTON_COUNT: usize = 3;
const ABOUT_BUTTON_URLS: [&str; ABOUT_BUTTON_COUNT] = [
    "https://x.com/tahayvr/",
    "https://github.com/tahayvr/omarchist",
    "https://www.omarchist.com/docs/",
];

pub struct AboutView {
    focus_handle: FocusHandle,
    /// Currently keyboard-focused button index (None = no keyboard focus)
    focused_button: Option<usize>,
}

impl AboutView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            focused_button: None,
        }
    }

    fn cycle_focus_forward(&mut self, cx: &mut Context<Self>) {
        self.focused_button = Some(match self.focused_button {
            None => 0,
            Some(i) => (i + 1) % ABOUT_BUTTON_COUNT,
        });
        cx.notify();
    }

    fn cycle_focus_backward(&mut self, cx: &mut Context<Self>) {
        self.focused_button = Some(match self.focused_button {
            None => ABOUT_BUTTON_COUNT - 1,
            Some(0) => ABOUT_BUTTON_COUNT - 1,
            Some(i) => i - 1,
        });
        cx.notify();
    }

    fn activate_focused(&self, cx: &mut Context<Self>) {
        if let Some(i) = self.focused_button {
            cx.open_url(ABOUT_BUTTON_URLS[i]);
        }
    }

    fn clear_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_button = None;
        cx.notify();
    }
}

impl Render for AboutView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let focused_border = theme.ring;

        let make_btn_wrapper = |index: usize, focused_button: Option<usize>| {
            let is_focused = focused_button == Some(index);
            div().rounded_md().when(is_focused, move |this: gpui::Div| {
                this.border_2().border_color(focused_border)
            })
        };

        v_flex()
            .id("about-view")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .gap_0()
            .size_full()
            .items_center()
            .justify_center()
            .text_center()
            .on_action(cx.listener(|this, _: &app_menu::NextFocus, _window, cx| {
                this.cycle_focus_forward(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::PrevFocus, _window, cx| {
                this.cycle_focus_backward(cx);
            }))
            .on_action(
                cx.listener(|this, _: &app_menu::ActivateItem, _window, cx| {
                    this.activate_focused(cx);
                }),
            )
            .on_action(cx.listener(|this, _: &app_menu::EscapeFocus, _window, cx| {
                this.clear_focus(cx);
            }))
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
                            .child("OMARCHIST"),
                    )
                    .child(
                        div()
                            .id("omarchist-version")
                            .text_color(theme.muted_foreground)
                            .child("v0.5.0"),
                    ),
            )
            .child(
                h_flex()
                    .mt_12()
                    .gap_4()
                    .child(
                        make_btn_wrapper(0, self.focused_button).child(
                            Button::new("x-com")
                                .icon(Icon::new(Icon::empty()).path("icons/x.svg").size_8())
                                .ghost()
                                .cursor_pointer()
                                .large()
                                .on_click(|_, _, cx| cx.open_url("https://x.com/tahayvr/")),
                        ),
                    )
                    .child(
                        make_btn_wrapper(1, self.focused_button).child(
                            Button::new("github")
                                .icon(Icon::new(IconName::GitHub).size_8())
                                .ghost()
                                .cursor_pointer()
                                .large()
                                .on_click(|_, _, cx| {
                                    cx.open_url("https://github.com/tahayvr/omarchist")
                                }),
                        ),
                    )
                    .child(
                        make_btn_wrapper(2, self.focused_button).child(
                            Button::new("docs")
                                .label("DOCS")
                                .ghost()
                                .cursor_pointer()
                                .large()
                                .on_click(|_, _, cx| {
                                    cx.open_url("https://www.omarchist.com/docs/")
                                }),
                        ),
                    ),
            )
    }
}
