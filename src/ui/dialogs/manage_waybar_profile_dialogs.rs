use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, WindowExt,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    v_flex,
};

use crate::system::waybar::{
    delete_waybar_profile, duplicate_waybar_profile, rename_waybar_profile,
};
use crate::ui::menu::app_menu::WaybarProfileManaged;

// After a rename/duplicate/delete, the new "active" profile name is stored here
// for delete, it's the profile to switch to; for rename/duplicate, it's the new name.

// The outcome of a profile management action.
#[derive(Debug, Clone, PartialEq)]
pub enum ProfileManagementResult {
    Renamed { new_name: String },
    Duplicated { new_name: String },
    Deleted { switch_to: String },
}

pub fn open_rename_waybar_profile_dialog(
    current_profile: String,
    window: &mut Window,
    cx: &mut App,
) {
    let name_input = cx.new(|cx| {
        InputState::new(window, cx)
            .default_value(current_profile.clone())
            .placeholder("New profile name")
    });
    let error: Entity<Option<String>> = cx.new(|_| None);
    let focus = name_input.focus_handle(cx);

    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Rename Profile")
            .w(px(420.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(true)
            .child(RenameProfileForm {
                current_profile: current_profile.clone(),
                name_input: name_input.clone(),
                error: error.clone(),
            })
    });

    focus.focus(window);
}

#[derive(IntoElement)]
struct RenameProfileForm {
    current_profile: String,
    name_input: Entity<InputState>,
    error: Entity<Option<String>>,
}

impl RenderOnce for RenameProfileForm {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let value = self.name_input.read(cx).value().trim().to_string();
        let is_empty = value.is_empty();
        let unchanged = value == self.current_profile;
        let error_text = self.error.read(cx).clone();

        let name_input = self.name_input.clone();
        let error_entity = self.error.clone();
        let old_name = self.current_profile.clone();

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
                            .child("New profile name"),
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
                        Button::new("cancel-rename")
                            .label("Cancel")
                            .ghost()
                            .on_click(|_, window, cx| {
                                window.close_dialog(cx);
                            }),
                    )
                    .child(
                        Button::new("confirm-rename")
                            .label("Rename")
                            .primary()
                            .disabled(is_empty || unchanged)
                            .on_click(move |_, window: &mut Window, cx| {
                                let new_name = name_input.read(cx).value().trim().to_string();

                                error_entity.update(cx, |err, cx| {
                                    *err = None;
                                    cx.notify();
                                });

                                match rename_waybar_profile(&old_name, &new_name) {
                                    Ok(renamed) => {
                                        window.close_dialog(cx);
                                        window.push_notification(
                                            format!("Renamed profile to \"{}\"", renamed),
                                            cx,
                                        );
                                        let action = WaybarProfileManaged(
                                            ProfileManagementResult::Renamed { new_name: renamed },
                                        );
                                        cx.dispatch_action(&action);
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

pub fn open_duplicate_waybar_profile_dialog(
    source_profile: String,
    window: &mut Window,
    cx: &mut App,
) {
    let suggested = format!("{}-copy", source_profile);
    let name_input = cx.new(|cx| {
        InputState::new(window, cx)
            .default_value(suggested)
            .placeholder("New profile name")
    });
    let error: Entity<Option<String>> = cx.new(|_| None);
    let focus = name_input.focus_handle(cx);

    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Duplicate Profile")
            .w(px(420.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(true)
            .child(DuplicateProfileForm {
                source_profile: source_profile.clone(),
                name_input: name_input.clone(),
                error: error.clone(),
            })
    });

    focus.focus(window);
}

#[derive(IntoElement)]
struct DuplicateProfileForm {
    source_profile: String,
    name_input: Entity<InputState>,
    error: Entity<Option<String>>,
}

impl RenderOnce for DuplicateProfileForm {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let value = self.name_input.read(cx).value().trim().to_string();
        let is_empty = value.is_empty();
        let error_text = self.error.read(cx).clone();

        let name_input = self.name_input.clone();
        let error_entity = self.error.clone();
        let source = self.source_profile.clone();

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
                            .child(format!("New name for copy of \"{}\"", self.source_profile)),
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
                        Button::new("cancel-duplicate")
                            .label("Cancel")
                            .ghost()
                            .on_click(|_, window, cx| {
                                window.close_dialog(cx);
                            }),
                    )
                    .child(
                        Button::new("confirm-duplicate")
                            .label("Duplicate")
                            .primary()
                            .disabled(is_empty)
                            .on_click(move |_, window: &mut Window, cx| {
                                let new_name = name_input.read(cx).value().trim().to_string();

                                error_entity.update(cx, |err, cx| {
                                    *err = None;
                                    cx.notify();
                                });

                                match duplicate_waybar_profile(&source, &new_name) {
                                    Ok(created) => {
                                        window.close_dialog(cx);
                                        window.push_notification(
                                            format!("Duplicated profile as \"{}\"", created),
                                            cx,
                                        );
                                        let action = WaybarProfileManaged(
                                            ProfileManagementResult::Duplicated {
                                                new_name: created,
                                            },
                                        );
                                        cx.dispatch_action(&action);
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

pub fn open_delete_waybar_profile_dialog(profile_name: String, window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Delete Profile")
            .w(px(420.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(true)
            .child(DeleteProfileConfirm {
                profile_name: profile_name.clone(),
            })
    });
}

#[derive(IntoElement)]
struct DeleteProfileConfirm {
    profile_name: String,
}

impl RenderOnce for DeleteProfileConfirm {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let name = self.profile_name.clone();

        v_flex()
            .p_4()
            .gap_4()
            .w_full()
            .child(div().text_sm().text_color(theme.foreground).child(format!(
                "Delete profile \"{}\"? This cannot be undone.",
                self.profile_name
            )))
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("cancel-delete")
                            .label("Cancel")
                            .ghost()
                            .on_click(|_, window, cx| {
                                window.close_dialog(cx);
                            }),
                    )
                    .child(
                        Button::new("confirm-delete")
                            .label("Delete")
                            .danger()
                            .on_click(
                                move |_, window: &mut Window, cx| match delete_waybar_profile(&name)
                                {
                                    Ok(switch_to) => {
                                        if let Some(next) = switch_to {
                                            window.close_dialog(cx);
                                            window.push_notification(
                                                format!("Deleted profile \"{}\"", name),
                                                cx,
                                            );
                                            let action = WaybarProfileManaged(
                                                ProfileManagementResult::Deleted {
                                                    switch_to: next,
                                                },
                                            );
                                            cx.dispatch_action(&action);
                                        }
                                    }
                                    Err(e) => {
                                        window.push_notification(
                                            format!("Could not delete: {}", e),
                                            cx,
                                        );
                                        window.close_dialog(cx);
                                    }
                                },
                            ),
                    ),
            )
    }
}
