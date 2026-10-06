//! A confirmation: the kit's alert dialog with a message and two buttons.
//! Cancel is the safe answer; the confirming button is red when the
//! action cannot be undone.
use std::rc::Rc;

use gpui::*;
use gpui_component::{WindowExt, button::ButtonVariant};

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
    let on_confirm = Rc::new(on_confirm);
    let ConfirmDialog {
        title,
        message,
        confirm_label,
        danger,
    } = dialog;
    window.open_alert_dialog(cx, move |alert, window, _| {
        let on_confirm = on_confirm.clone();
        alert
            .title(title)
            .description(message.clone())
            .confirm()
            .ok_text(confirm_label)
            .ok_variant(if danger {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Primary
            })
            .width(crate::ui::focus::dialog_width(440., window))
            .on_ok(move |_, window, cx| {
                on_confirm(window, cx);
                true
            })
    });
}
