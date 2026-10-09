//! Toasts above dialogs. gpui-kit paints a dialog as a deferred element
//! over its own notification layer, so a toast raised from a dialog (a
//! refused save) would sit under the dialog's overlay, unreadable. This
//! root plugin draws the window's toasts once more while a dialog is up,
//! deferred above every dialog and laid out as the kit lays them out, so
//! the copy covers the original exactly; with no dialog it draws nothing.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_base::{RootPlugin, ToastStack, ToastStackState};
use gpui_component::{ActiveTheme, StyledExt, WindowExt};

pub struct ToastsAboveDialogs {
    state: ToastStackState,
    focus: FocusHandle,
}

impl Render for ToastsAboveDialogs {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !window.has_active_dialog(cx) {
            return div().into_any_element();
        }
        let toasts = window.notifications(cx);
        if toasts.is_empty() {
            return div().into_any_element();
        }
        let size = window.viewport_size();
        let settings = &cx.theme().notification;
        let (placement, margins, width) =
            (settings.placement, settings.margins.clone(), settings.width);
        let stack = toasts
            .iter()
            .fold(
                ToastStack::new("toasts-above-dialogs", self.state.clone()),
                |stack, toast| stack.item(("toast", toast.entity_id().as_u64()), toast.clone()),
            )
            .placement(placement)
            .focus_handle(self.focus.clone())
            .v_flex()
            .w(width)
            .max_h(size.height)
            .absolute()
            .map(|this| match placement {
                Anchor::TopLeft => this.top(margins.top).left(margins.left),
                Anchor::TopRight => this.top(margins.top).right(margins.right),
                Anchor::TopCenter => this.top(margins.top).left(relative(0.5)).ml(-width / 2.),
                Anchor::BottomLeft => this.bottom(margins.bottom).left(margins.left),
                Anchor::BottomRight => this.bottom(margins.bottom).right(margins.right),
                Anchor::BottomCenter => this
                    .bottom(margins.bottom)
                    .left(relative(0.5))
                    .ml(-width / 2.),
                Anchor::LeftCenter => this.left(margins.left).top_0().bottom_0().my_auto(),
                Anchor::RightCenter => this.right(margins.right).top_0().bottom_0().my_auto(),
            });
        deferred(div().absolute().inset_0().child(stack))
            // Dialogs are 10 and up, one per layer.
            .with_priority(100)
            .into_any_element()
    }
}

impl RootPlugin for ToastsAboveDialogs {}

/// Where toasts rise from: the bottom right, away from every page's
/// toolbar and the editors' Save and Run, which sit top right. A toast at
/// the top covered those buttons for as long as it showed.
pub const PLACEMENT: Anchor = Anchor::BottomRight;

/// Registers the plugin for every window opened afterwards and sets the
/// toast corner; call it after the kit's own `init`, which registers the
/// layers it draws over. The app and the headless tests both call it, so a
/// test window's toasts rise where the app's do.
pub fn install(cx: &mut App) {
    gpui_component::Theme::global_mut(cx).notification.placement = PLACEMENT;
    gpui_base::Root::register_plugin(cx, |_, cx| ToastsAboveDialogs {
        state: ToastStackState::default(),
        focus: cx.focus_handle().tab_stop(true),
    });
}
