// A small confirmation dialog: Cancel is focused first, Tab moves between
// the two buttons, Escape cancels, Ctrl+Enter confirms.
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};

use crate::ui::focus;

type ConfirmHandler = Rc<dyn Fn(&mut Window, &mut App)>;

pub struct ConfirmDialog {
    pub title: &'static str,
    pub message: String,
    pub confirm_label: &'static str,
    pub danger: bool,
}

pub fn open_confirm_dialog(
    dialog: ConfirmDialog,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) {
    let on_confirm: ConfirmHandler = Rc::new(on_confirm);
    let body_focus = cx.focus_handle();
    let ConfirmDialog {
        title,
        message,
        confirm_label,
        danger,
    } = dialog;

    let trap_focus = body_focus.clone();
    window.open_dialog(cx, move |d, _, cx| {
        let confirm = on_confirm.clone();
        let submit = on_confirm.clone();
        let muted = cx.theme().muted_foreground;
        d.title(title)
            .w(px(440.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(false)
            .child(
                focus::dialog_body("confirm-dialog", &trap_focus, move |window, cx| {
                    submit(window, cx);
                    window.close_dialog(cx);
                })
                .child(
                    v_flex()
                        .gap_4()
                        .child(div().text_sm().text_color(muted).child(message.clone()))
                        .child(
                            h_flex()
                                .justify_end()
                                .gap_2()
                                .child(
                                    Button::new("confirm-cancel")
                                        .outline()
                                        .small()
                                        .label("Cancel")
                                        .cursor_pointer()
                                        .on_click(|_, window, cx| window.close_dialog(cx)),
                                )
                                .child(
                                    Button::new("confirm-ok")
                                        .small()
                                        .map(|this| {
                                            if danger {
                                                this.danger()
                                            } else {
                                                this.primary()
                                            }
                                        })
                                        .label(confirm_label)
                                        .cursor_pointer()
                                        .on_click(move |_, window, cx| {
                                            confirm(window, cx);
                                            window.close_dialog(cx);
                                        }),
                                ),
                        ),
                ),
            )
    });
    focus::focus_first_in(&body_focus, window);
}
