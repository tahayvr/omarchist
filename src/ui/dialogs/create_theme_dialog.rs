use std::cell::RefCell;
use std::path::PathBuf;

use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, WindowExt,
    button::{Button, ButtonVariants},
    divider::Divider,
    h_flex, v_flex,
};

use crate::system::themes::theme_generator::create_theme_from_image;
use crate::system::themes::theme_management::{
    create_theme_from_defaults, generate_unique_theme_name,
};

thread_local! {
    pub static PENDING_THEME_NAVIGATION: RefCell<Option<String>> = const { RefCell::new(None) };
    pub static PENDING_REFRESH_THEMES: RefCell<bool> = const { RefCell::new(false) };
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
                                    .cursor_pointer()
                                    .on_click(|_, window, cx| {
                                        window.close_dialog(cx);
                                        open_image_picker(window, cx);
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
                                    .cursor_pointer()
                                    .on_click(|_, window, cx| {
                                        let theme_name = generate_unique_theme_name();

                                        match create_theme_from_defaults(&theme_name) {
                                            Ok(created_theme_name) => {
                                                PENDING_THEME_NAVIGATION.with(|nav| {
                                                    *nav.borrow_mut() =
                                                        Some(created_theme_name.clone());
                                                });

                                                window.close_dialog(cx);

                                                let msg = format!(
                                                    "Created new theme: {}",
                                                    created_theme_name
                                                );
                                                window.push_notification(msg, cx);

                                                cx.refresh_windows();
                                            }
                                            Err(e) => {
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

fn open_image_picker(window: &mut Window, cx: &mut App) {
    // Open file picker dialog
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
        .set_title("Select an image for theme creation")
        .pick_file()
    {
        // Process the image and create theme
        process_image_and_create_theme(window, cx, path);
    }
}

fn process_image_and_create_theme(window: &mut Window, cx: &mut App, image_path: PathBuf) {
    // Generate theme name from filename
    let theme_name = image_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase().replace(' ', "-"))
        .unwrap_or_else(generate_unique_theme_name);

    // Show a brief notification that we're processing
    window.push_notification("Creating theme from image...", cx);

    // Run theme creation
    match create_theme_from_image(&image_path, &theme_name, None) {
        Ok(created_name) => {
            // Store theme name for navigation
            PENDING_THEME_NAVIGATION.with(|nav| {
                *nav.borrow_mut() = Some(created_name.clone());
            });

            // Show success notification
            window.push_notification(format!("Created theme: {}", created_name), cx);

            // Trigger navigation
            cx.refresh_windows();
        }
        Err(e) => {
            window.push_notification(format!("Failed to create theme: {}", e), cx);
        }
    }
}

/// Get the pending theme navigation and clear it
pub fn take_pending_navigation() -> Option<String> {
    PENDING_THEME_NAVIGATION.with(|nav| nav.borrow_mut().take())
}

/// Check if themes list needs refresh and clear the flag
pub fn take_pending_refresh() -> bool {
    PENDING_REFRESH_THEMES.with(|refresh| {
        let value = *refresh.borrow();
        *refresh.borrow_mut() = false;
        value
    })
}
