use crate::system::theme_management::{create_theme_from_defaults, generate_unique_theme_name};
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, WindowExt,
    button::{Button, ButtonVariants},
    divider::Divider,
    h_flex, v_flex,
};
use std::cell::RefCell;

thread_local! {
    /// Stores the theme name to navigate to after creation
    pub static PENDING_THEME_NAVIGATION: RefCell<Option<String>> = RefCell::new(None);

    /// Flag to trigger themes list refresh
    pub static PENDING_REFRESH_THEMES: RefCell<bool> = RefCell::new(false);
}

pub fn open_create_theme_dialog(window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, |dialog, _, cx| {
        dialog
            .title("Create New Theme")
            .w(px(640.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(true)
            .child(
                h_flex()
                    .h(px(320.))
                    .child(
                        // Left Column - Create from Image
                        v_flex()
                            .flex_1()
                            .h_full()
                            .p_4()
                            .gap_4()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(Icon::empty())
                                    .path("icons/image.svg")
                                    .size(px(48.))
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Automatically based on an image file"),
                            )
                            .child(
                                Button::new("from-image-btn")
                                    .primary()
                                    .label("Select Image")
                                    .on_click(|_, window, cx| {
                                        // TODO: Implement image-based theme creation
                                        window.close_dialog(cx);
                                        window.push_notification(
                                            "Image-based theme creation coming soon!",
                                            cx,
                                        );
                                    }),
                            ),
                    )
                    .child(Divider::vertical().color(cx.theme().border))
                    .child(
                        // Right Column - Create Manually
                        v_flex()
                            .flex_1()
                            .h_full()
                            .p_4()
                            .gap_4()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(IconName::Palette)
                                    .size(px(48.))
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Manually from scratch"),
                            )
                            .child(
                                Button::new("from-scratch-btn")
                                    .primary()
                                    .label("Create Manually")
                                    .on_click(|_, window, cx| {
                                        // Generate a unique theme name
                                        let theme_name = generate_unique_theme_name();

                                        // Create theme from defaults
                                        match create_theme_from_defaults(&theme_name) {
                                            Ok(created_theme_name) => {
                                                // Store theme name for navigation
                                                PENDING_THEME_NAVIGATION.with(|nav| {
                                                    *nav.borrow_mut() =
                                                        Some(created_theme_name.clone());
                                                });

                                                // Close dialog
                                                window.close_dialog(cx);

                                                // Show success notification
                                                let msg = format!(
                                                    "Created new theme: {}",
                                                    created_theme_name
                                                );
                                                window.push_notification(msg, cx);

                                                // Trigger navigation via refresh
                                                cx.refresh_windows();
                                            }
                                            Err(e) => {
                                                // Close dialog and show error
                                                window.close_dialog(cx);
                                                let msg = format!("Failed to create theme: {}", e);
                                                window.push_notification(msg, cx);
                                            }
                                        }
                                    }),
                            ),
                    ),
            )
    });
}
