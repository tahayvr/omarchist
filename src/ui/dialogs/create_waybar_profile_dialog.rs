use std::cell::RefCell;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, WindowExt,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    v_flex,
};

use crate::system::waybar::create_waybar_profile;

thread_local! {
    /// Set to the new profile name after successful creation so the header can
    /// reload its list and switch to it.
    pub static PENDING_PROFILE_NAVIGATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Take the pending profile navigation and clear it.
pub fn take_pending_profile_navigation() -> Option<String> {
    PENDING_PROFILE_NAVIGATION.with(|nav| nav.borrow_mut().take())
}

pub fn open_create_waybar_profile_dialog(window: &mut Window, cx: &mut App) {
    // InputState is created outside the closure so it survives across re-renders
    let name_input =
        cx.new(|cx| InputState::new(window, cx).placeholder("e.g. work, gaming, minimal..."));
    // Shared error string — updated by the Create button, cleared on re-render when empty
    let error: Entity<Option<String>> = cx.new(|_| None);

    let focus = name_input.focus_handle(cx);

    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("New Waybar Profile")
            .w(px(420.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(true)
            .child(CreateProfileForm {
                name_input: name_input.clone(),
                error: error.clone(),
            })
    });

    // Auto-focus the input field
    focus.focus(window);
}

#[derive(IntoElement)]
struct CreateProfileForm {
    name_input: Entity<InputState>,
    error: Entity<Option<String>>,
}

impl RenderOnce for CreateProfileForm {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let value = self.name_input.read(cx).value().trim().to_string();
        let is_empty = value.is_empty();
        let error_text = self.error.read(cx).clone();
        let name_input = self.name_input.clone();
        let error_entity = self.error.clone();

        v_flex()
            .p_4()
            .gap_4()
            .w_full()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("Profile name"),
                    )
                    .child(Input::new(&self.name_input))
                    .when(error_text.is_some(), |this: gpui::Div| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.danger)
                                .child(error_text.unwrap_or_default()),
                        )
                    }),
            )
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("cancel-profile")
                            .label("Cancel")
                            .ghost()
                            .on_click(|_, window, cx| {
                                window.close_dialog(cx);
                            }),
                    )
                    .child(
                        Button::new("create-profile")
                            .label("Create")
                            .primary()
                            .disabled(is_empty)
                            .on_click(move |_, window: &mut Window, cx| {
                                let profile_name = name_input.read(cx).value().trim().to_string();

                                // Clear previous error when attempting again
                                error_entity.update(cx, |err, cx| {
                                    *err = None;
                                    cx.notify();
                                });

                                match create_waybar_profile(&profile_name) {
                                    Ok(created_name) => {
                                        PENDING_PROFILE_NAVIGATION.with(|nav| {
                                            *nav.borrow_mut() = Some(created_name.clone());
                                        });
                                        window.close_dialog(cx);
                                        window.push_notification(
                                            format!("Created profile \"{}\"", created_name),
                                            cx,
                                        );
                                        cx.refresh_windows();
                                    }
                                    Err(e) => {
                                        error_entity.update(cx, |err, cx| {
                                            *err = Some(e);
                                            cx.notify();
                                        });
                                    }
                                }
                            }),
                    ),
            )
    }
}
