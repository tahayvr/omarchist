use crate::system::themes::theme_file_ops::{
    add_background_image, boot_logo, list_background_images, remove_background_image,
    remove_boot_logo, render_boot_preview, set_boot_logo,
};
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::text::selectable;
use crate::ui::theme_edit_page::shared::{
    IMAGE_EXTENSIONS, error_message, focus_section, tab_container,
};
use anyhow;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    separator::Separator,
    v_flex,
};
use std::path::PathBuf;

#[derive(Clone)]
pub struct BackgroundImage {
    pub path: PathBuf,
    pub filename: String,
}

pub struct BackgroundsTab {
    theme_name: String,
    is_system_theme: bool,
    images: Vec<BackgroundImage>,
    error_message: Option<String>,
    is_loading: bool,
    boot_logo: Option<PathBuf>,
    /// Set while a logo is copied or its preview rendered.
    boot_busy: bool,
    boot_error: Option<String>,
    scroll: ScrollHandle,
}

impl BackgroundsTab {
    pub fn new(
        theme_name: String,
        is_system_theme: bool,
        scroll: &ScrollHandle,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut tab = Self {
            theme_name: theme_name.clone(),
            is_system_theme,
            images: Vec::new(),
            error_message: None,
            is_loading: true,
            boot_logo: boot_logo(&theme_name, is_system_theme),
            boot_busy: false,
            boot_error: None,
            scroll: scroll.clone(),
        };

        tab.load_images(cx);

        tab
    }

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

    fn add_images(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.error_message = None;

        let theme_name = self.theme_name.clone();
        let is_system_theme = self.is_system_theme;

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    rfd::FileDialog::new()
                        .add_filter("Images", IMAGE_EXTENSIONS)
                        .set_title("Select Background Images")
                        .pick_files()
                })
                .await;

            if let Some(paths) = result {
                this.update(cx, |this, cx| {
                    this.is_loading = true;
                    cx.notify();
                })
                .ok();
                // Copying wallpapers is slow enough to freeze the window.
                let (added_count, errors) = cx
                    .background_spawn(async move {
                        let mut added_count = 0;
                        let mut errors = Vec::new();
                        for path in &paths {
                            match add_background_image(&theme_name, is_system_theme, path) {
                                Ok(_) => added_count += 1,
                                Err(e) => errors.push(format!("{}: {}", path.display(), e)),
                            }
                        }
                        (added_count, errors)
                    })
                    .await;

                let _ = this.update(cx, |this, cx| {
                    this.load_images(cx);

                    if !errors.is_empty() {
                        this.error_message = Some(format!(
                            "Added {} images. Not added: {}",
                            added_count,
                            errors.join("; ")
                        ));
                        cx.notify();
                    }
                });
            }

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    }

    /// Asks first: the file was copied into the theme, so the source may be
    /// gone, and the button sits where a slipped click lands.
    fn confirm_delete_image(&self, filename: String, window: &mut Window, cx: &mut Context<Self>) {
        let tab = cx.entity().downgrade();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Remove this background?",
                message: format!("{filename} is deleted from the theme."),
                confirm_label: "Remove",
                danger: true,
            },
            move |window, cx| {
                tab.update(cx, |tab, cx| tab.delete_image(&filename, window, cx))
                    .ok();
            },
            window,
            cx,
        );
    }

    fn delete_image(&mut self, filename: &str, _window: &mut Window, cx: &mut Context<Self>) {
        self.error_message = None;

        match remove_background_image(&self.theme_name, self.is_system_theme, filename) {
            Ok(()) => {
                self.images.retain(|img| img.filename != filename);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to delete image: {}", e));
            }
        }

        cx.notify();
    }

    /// Picks a PNG, copies it in as `unlock.png`, and renders the preview the
    /// boot screen switcher needs to list the theme.
    fn choose_boot_logo(&mut self, cx: &mut Context<Self>) {
        let theme_name = self.theme_name.clone();
        cx.spawn(async move |this, cx| {
            let picked = cx
                .background_spawn(async move {
                    rfd::FileDialog::new()
                        .add_filter("PNG image", &["png", "PNG"])
                        .set_title("Select a Boot Logo")
                        .pick_file()
                })
                .await;
            let Some(path) = picked else {
                return;
            };
            this.update(cx, |this, cx| {
                this.boot_busy = true;
                cx.notify();
            })
            .ok();
            let result = cx
                .background_spawn(async move {
                    let logo = set_boot_logo(&theme_name, &path)?;
                    render_boot_preview(&theme_name).map(|()| logo)
                })
                .await;
            this.update(cx, |this, cx| {
                this.boot_busy = false;
                this.boot_logo = boot_logo(&this.theme_name, this.is_system_theme);
                this.boot_error = result.err().map(|e| e.to_string());
                // gpui caches decoded images by path; the file changed.
                this.forget_boot_images(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Drops the cached decodes of the logo and its preview so a replaced
    /// file is drawn, not the old picture.
    fn forget_boot_images(&self, cx: &mut App) {
        if let Some(dir) = self.boot_logo.as_ref().and_then(|p| p.parent()) {
            for file in ["unlock.png", "preview-unlock.png"] {
                ImageSource::from(dir.join(file)).remove_asset(cx);
            }
        }
    }

    fn refresh_boot_preview(&mut self, cx: &mut Context<Self>) {
        self.boot_busy = true;
        let theme_name = self.theme_name.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { render_boot_preview(&theme_name) })
                .await;
            this.update(cx, |this, cx| {
                this.boot_busy = false;
                this.boot_error = result.err().map(|e| e.to_string());
                this.forget_boot_images(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn remove_boot_logo(&mut self, cx: &mut Context<Self>) {
        match remove_boot_logo(&self.theme_name) {
            Ok(()) => {
                self.boot_logo = None;
                self.boot_error = None;
            }
            Err(e) => self.boot_error = Some(e.to_string()),
        }
        cx.notify();
    }

    fn render_boot_logo(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let busy = self.boot_busy;
        let editable = !self.is_system_theme;

        let preview = match &self.boot_logo {
            Some(path) => div()
                .w(px(240.))
                .h(px(120.))
                .p_2()
                .overflow_hidden()
                .border_1()
                .border_color(theme.border)
                .bg(theme.muted)
                .child(
                    // `img` otherwise takes the picture's own aspect ratio,
                    // which outweighs the height and overflows the frame.
                    img(path.clone())
                        .w(px(222.))
                        .h(px(102.))
                        .aspect_ratio(222. / 102.)
                        .object_fit(ObjectFit::Contain),
                )
                .into_any_element(),
            None => Label::new("No boot logo")
                .text_sm()
                .text_color(muted)
                .into_any_element(),
        };

        v_flex()
            .gap_3()
            .child(
                Label::new("Boot Logo")
                    .text_lg()
                    .font_weight(FontWeight::MEDIUM),
            )
            .child(preview)
            .children(self.boot_error.clone().map(|e| error_message(e, cx)))
            .when(editable, |section| {
                section.child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .child(
                            Button::new("boot-logo-choose")
                                .label(if self.boot_logo.is_some() {
                                    "Replace Logo"
                                } else {
                                    "Choose Logo"
                                })
                                .small()
                                .outline()
                                .loading(busy)
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| this.choose_boot_logo(cx))),
                        )
                        .when(self.boot_logo.is_some(), |row| {
                            row.child(
                                Button::new("boot-logo-refresh")
                                    .label("Refresh Preview")
                                    .small()
                                    .ghost()
                                    .disabled(busy)
                                    .cursor_pointer()
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.refresh_boot_preview(cx)),
                                    ),
                            )
                            .child(
                                Button::new("boot-logo-remove")
                                    .label("Remove")
                                    .small()
                                    .ghost()
                                    .disabled(busy)
                                    .cursor_pointer()
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.remove_boot_logo(cx)),
                                    ),
                            )
                        }),
                )
            })
    }

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
            .child(focus_section(
                "backgrounds-header",
                &self.scroll,
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
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_images(window, cx);
                            })),
                    ),
            ))
            .child(focus_section(
                "backgrounds-grid",
                &self.scroll,
                if is_loading {
                    v_flex()
                        .p_8()
                        .items_center()
                        .child(Label::new("Loading...").text_color(cx.theme().muted_foreground))
                        .into_any_element()
                } else if images.is_empty() {
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
                                .child(Label::new("🖼️").text_2xl()),
                        )
                        .child(
                            Label::new("No background images")
                                .text_color(cx.theme().muted_foreground),
                        )
                        .into_any_element()
                } else {
                    let mut grid = v_flex().gap_6();
                    let mut image_index: usize = 0;

                    for row_images in images.chunks(images_per_row) {
                        let mut row = h_flex().gap_6();

                        for image in row_images {
                            let filename = image.filename.clone();
                            let path = image.path.clone();
                            let current_index = image_index;
                            image_index += 1;

                            row = row.child(
                                v_flex()
                                    .w(px(150.))
                                    .gap_2()
                                    .child(
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
                                                div().absolute().top_1().right_1().child(
                                                    Button::new(("delete-bg", current_index))
                                                        .icon(IconName::Close)
                                                        .small()
                                                        .danger()
                                                        .cursor_pointer()
                                                        .on_click(cx.listener({
                                                            let filename = filename.clone();
                                                            move |this, _, window, cx| {
                                                                this.confirm_delete_image(
                                                                    filename.clone(),
                                                                    window,
                                                                    cx,
                                                                );
                                                            }
                                                        })),
                                                ),
                                            ),
                                    )
                                    .child(
                                        div().w(px(150.)).child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .truncate()
                                                .child(selectable(
                                                    ("bg-filename", current_index),
                                                    filename.clone(),
                                                )),
                                        ),
                                    ),
                            );
                        }
                        grid = grid.child(row);
                    }
                    grid.into_any_element()
                },
            ))
            .children(
                self.error_message
                    .as_ref()
                    .map(|msg| error_message(msg.clone(), cx)),
            )
            .child(Separator::horizontal())
            .child(focus_section(
                "backgrounds-boot-logo",
                &self.scroll,
                self.render_boot_logo(cx),
            ))
    }
}
