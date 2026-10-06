//! How the app reports: a toast of one of four kinds, gone on its own.
//! `success` is work completed, `info` neutral news, `warning` a refusal
//! or a caveat, `error` a failure. Nothing is drawn into a page or a
//! dialog for a message, so the layout never shifts to make room for one.
use gpui::*;
use gpui_component::{WindowExt, notification::Notification};

pub fn success(window: &mut Window, message: impl Into<SharedString>, cx: &mut App) {
    window.push_notification(Notification::success(message), cx);
}

pub fn info(window: &mut Window, message: impl Into<SharedString>, cx: &mut App) {
    window.push_notification(Notification::info(message), cx);
}

pub fn warning(window: &mut Window, message: impl Into<SharedString>, cx: &mut App) {
    window.push_notification(Notification::warning(message), cx);
}

pub fn error(window: &mut Window, message: impl Into<SharedString>, cx: &mut App) {
    window.push_notification(Notification::error(message), cx);
}

/// How something ended: a success when it went well, an error otherwise.
pub fn result(window: &mut Window, ok: bool, message: impl Into<SharedString>, cx: &mut App) {
    if ok {
        success(window, message, cx)
    } else {
        error(window, message, cx)
    }
}
