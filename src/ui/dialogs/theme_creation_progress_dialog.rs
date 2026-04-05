use std::path::PathBuf;

use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, WindowExt, button::Button, v_flex};
use smol;

use crate::system::themes::theme_generator::create_theme_from_image;
use crate::ui::menu::app_menu::{NavigateToThemeEdit, RefreshThemes};

pub struct ThemeCreationProgressDialog {
    theme_name: String,
    image_path: PathBuf,
    status_message: String,
    is_complete: bool,
    has_error: bool,
    error_message: Option<String>,
}

impl ThemeCreationProgressDialog {
    pub fn new(theme_name: String, image_path: PathBuf) -> Self {
        Self {
            theme_name,
            image_path,
            status_message: "Analyzing image...".to_string(),
            is_complete: false,
            has_error: false,
            error_message: None,
        }
    }

    pub fn start_creation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let theme_name = self.theme_name.clone();
        let image_path = self.image_path.clone();
        let window_handle = window.window_handle();

        // Spawn async task to create theme
        cx.spawn(async move |this, cx| {
            // Update status to creating structure
            let _ = window_handle.update(cx, |_view, _window, cx| {
                let _ = this.update(cx, |this, cx| {
                    this.status_message = "Creating theme structure...".to_string();
                    cx.notify();
                });
            });

            // Run theme creation in a blocking thread with periodic status updates
            let result = smol::unblock({
                let theme_name = theme_name.clone();
                let image_path = image_path.clone();
                move || create_theme_from_image(&image_path, &theme_name, None)
            })
            .await;

            // Handle result
            match result {
                Ok(created_name) => {
                    let _ = window_handle.update(cx, |_view, _window, cx| {
                        let nav = NavigateToThemeEdit(created_name.clone());
                        cx.dispatch_action(&nav);
                        cx.dispatch_action(&RefreshThemes);
                        let _ = this.update(cx, |this, cx| {
                            this.is_complete = true;
                            this.status_message = format!("Created theme: {}", created_name);
                            cx.notify();
                        });
                    });

                    // Auto-close after a short delay
                    smol::Timer::after(std::time::Duration::from_secs(1)).await;
                    let _ = window_handle.update(cx, |_view, window, cx| {
                        window.close_dialog(cx);
                    });
                }
                Err(e) => {
                    let _ = window_handle.update(cx, |_view, _window, cx| {
                        let _ = this.update(cx, |this, cx| {
                            this.has_error = true;
                            this.error_message = Some(e.clone());
                            this.status_message = format!("Error: {}", e);
                            cx.notify();
                        });
                    });
                }
            }

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    }

    fn close(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        window.close_dialog(_cx);
    }
}

impl Render for ThemeCreationProgressDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .w(px(400.0))
            .p_6()
            .gap_4()
            .items_center()
            .justify_center()
            .child(
                // Icon
                if self.has_error {
                    Icon::new(IconName::TriangleAlert)
                        .size(px(48.0))
                        .text_color(theme.red)
                } else if self.is_complete {
                    Icon::new(IconName::Check)
                        .size(px(48.0))
                        .text_color(theme.green)
                } else {
                    Icon::new(IconName::Loader)
                        .size(px(48.0))
                        .text_color(theme.primary)
                },
            )
            .child(
                // Title
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.foreground)
                    .child("Creating Theme from Image"),
            )
            .child(
                // Status message
                div()
                    .text_sm()
                    .text_color(if self.has_error {
                        theme.red
                    } else {
                        theme.muted_foreground
                    })
                    .child(self.status_message.clone()),
            )
            .child(
                // Action buttons (only show when error)
                if self.has_error {
                    Button::new("close-btn")
                        .label("Close")
                        .on_click(cx.listener(|this, _event, window, cx| {
                            this.close(window, cx);
                        }))
                        .into_any_element()
                } else {
                    div().into_any_element()
                },
            )
    }
}

pub fn open_theme_creation_progress_dialog(
    theme_name: String,
    image_path: PathBuf,
    window: &mut Window,
    cx: &mut App,
) {
    let theme_name_clone = theme_name.clone();
    let image_path_clone = image_path.clone();

    window.open_dialog(cx, move |dialog_builder, window, cx| {
        let dialog = cx.new(|cx| {
            let mut dialog = ThemeCreationProgressDialog::new(
                theme_name_clone.clone(),
                image_path_clone.clone(),
            );
            dialog.start_creation(window, cx);
            dialog
        });

        dialog_builder
            .overlay(true)
            .keyboard(true)
            .close_button(false)
            .overlay_closable(false)
            .child(dialog)
    });
}
