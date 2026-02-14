//! Backgrounds tab for theme editing
//!
//! Provides UI for managing background images in the /backgrounds folder:
//! - Display existing background images in a grid
//! - Open backgrounds folder in Nautilus for adding/removing images
//! - Delete individual images directly from the UI

use crate::system::theme_file_ops::{
    add_background_image, list_background_images, remove_background_image,
};
use crate::ui::theme_edit_page::shared::{error_message, help_text, tab_container};
use gpui::*;
use gpui_component::{
    ActiveTheme, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    v_flex,
};
use std::path::PathBuf;

/// Represents a background image entry
#[derive(Clone)]
pub struct BackgroundImage {
    pub path: PathBuf,
    pub filename: String,
}

/// Backgrounds tab content for managing theme background images
pub struct BackgroundsTab {
    theme_name: String,
    is_system_theme: bool,
    images: Vec<BackgroundImage>,
    error_message: Option<String>,
    is_loading: bool,
}

impl BackgroundsTab {
    /// Create a new BackgroundsTab instance
    pub fn new(
        theme_name: String,
        is_system_theme: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut tab = Self {
            theme_name: theme_name.clone(),
            is_system_theme,
            images: Vec::new(),
            error_message: None,
            is_loading: true,
        };

        // Load background images
        tab.load_images(cx);

        tab
    }

    /// Load the list of background images from the theme folder
    fn load_images(&mut self, cx: &mut Context<Self>) {
        self.is_loading = true;
        self.error_message = None;

        match list_background_images(&self.theme_name, self.is_system_theme) {
            Ok(paths) => {
                self.images = paths
                    .into_iter()
                    .filter_map(|path| {
                        path.file_name().map(|name| BackgroundImage {
                            path: path.clone(),
                            filename: name.to_string_lossy().to_string(),
                        })
                    })
                    .collect();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load backgrounds: {}", e));
            }
        }

        self.is_loading = false;
        cx.notify();
    }

    /// Add images using native file picker dialog
    fn add_images(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.error_message = None;

        // Use rfd to pick multiple image files
        let files = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
            .set_title("Select Background Images")
            .pick_files();

        if let Some(paths) = files {
            let mut added_count = 0;
            let mut errors = Vec::new();

            for path in paths {
                match add_background_image(&self.theme_name, self.is_system_theme, &path) {
                    Ok(_) => added_count += 1,
                    Err(e) => errors.push(format!("{}: {}", path.display(), e)),
                }
            }

            // Reload images to show new ones
            self.load_images(cx);

            // Show error if any files failed
            if !errors.is_empty() {
                self.error_message = Some(format!(
                    "Added {} images. Failed to add: {}",
                    added_count,
                    errors.join("; ")
                ));
                cx.notify();
            }
        }
    }

    /// Delete a background image
    fn delete_image(&mut self, filename: &str, _window: &mut Window, cx: &mut Context<Self>) {
        self.error_message = None;

        match remove_background_image(&self.theme_name, self.is_system_theme, filename) {
            Ok(()) => {
                // Remove from local list
                self.images.retain(|img| img.filename != filename);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to delete image: {}", e));
            }
        }

        cx.notify();
    }

    /// Get the number of images per row based on available width
    fn images_per_row(&self, window: &mut Window) -> usize {
        // Each image card is approximately 170px wide (150px image + padding)
        // Calculate how many fit in the current window width
        let window_width_f32: f32 = window.viewport_size().width.into();
        let window_width = window_width_f32 as usize;
        let card_width = 170;
        let min_cards = 2;
        let max_cards = 6;

        ((window_width / card_width).max(min_cards)).min(max_cards)
    }
}

impl Render for BackgroundsTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let images = self.images.clone();
        let is_loading = self.is_loading;
        let images_per_row = self.images_per_row(window);

        tab_container()
            .child(
                // Header section with title and action button
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        Label::new("Background Images")
                            .text_lg()
                            .font_weight(FontWeight::MEDIUM),
                    )
                    .child(
                        Button::new("add-images-btn")
                            .label("Add Images")
                            .primary()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_images(window, cx);
                            })),
                    ),
            )
            .child(
                help_text("Manage background images for this theme. Click \"Add Images\" to select images to add. You can also delete images directly from the grid below."),
            )
            .child(
                // Image count display
                Label::new(format!("{} image{}",
                    images.len(),
                    if images.len() == 1 { "" } else { "s" }
                ))
                .text_sm()
                .text_color(cx.theme().muted_foreground),
            )
            .child(
                // Image grid or empty state
                if is_loading {
                    v_flex()
                        .p_8()
                        .items_center()
                        .child(Label::new("Loading...").text_color(cx.theme().muted_foreground))
                        .into_any_element()
                } else if images.is_empty() {
                    // Empty state
                    v_flex()
                        .p_8()
                        .gap_4()
                        .items_center()
                        .child(
                            div()
                                .size_16()
                                .bg(cx.theme().muted)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    Label::new("🖼️")
                                        .text_2xl(),
                                ),
                        )
                        .child(
                            Label::new("No background images")
                                .text_color(cx.theme().muted_foreground),
                        )
                        .child(
                            help_text("Click \"Add Images\" to select background images"),
                        )
                        .into_any_element()
                } else {
                    // Image grid
                    let mut grid = v_flex().gap_6();
                    let mut image_index: usize = 0;

                    // Group images into rows
                    for row_images in images.chunks(images_per_row) {
                        let mut row = h_flex().gap_6();

                        for image in row_images {
                            let filename = image.filename.clone();
                            let path = image.path.clone();
                            let current_index = image_index;
                            image_index += 1;

                            row = row.child(
                                // Image card
                                v_flex()
                                    .w(px(150.))
                                    .gap_2()
                                    .child(
                                        // Image container with delete button overlay
                                        div()
                                            .relative()
                                            .w(px(150.))
                                            .h(px(100.))
                                            .overflow_hidden()
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .child(
                                                img(path)
                                                    .w_full()
                                                    .h_full()
                                                    .object_fit(ObjectFit::Cover),
                                            )
                                            .child(
                                                // Delete button overlay (top-right)
                                                div()
                                                    .absolute()
                                                    .top_1()
                                                    .right_1()
                                                    .child(
                                                        Button::new(("delete-bg", current_index))
                                                            .icon(IconName::Close)
                                                            .small()
                                                            .danger()
                                                            .on_click(cx.listener({
                                                                let filename = filename.clone();
                                                                move |this, _, window, cx| {
                                                                    this.delete_image(&filename, window, cx);
                                                                }
                                                            })),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // Filename label (truncated)
                                        div()
                                            .w(px(150.))
                                            .child(
                                                Label::new(&filename)
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .truncate(),
                                            ),
                                    ),
                            );
                        }
                        grid = grid.child(row);
                    }
                    grid.into_any_element()
                },
            )
            .children(
                self.error_message
                    .as_ref()
                    .map(|msg| error_message(msg.clone())),
            )
    }
}
