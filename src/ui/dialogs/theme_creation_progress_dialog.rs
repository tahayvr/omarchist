use std::path::PathBuf;

use crate::ui::app_events::{AppEvent, emit};
use crate::ui::notify;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, WindowExt, spinner::Spinner, v_flex};

use crate::system::themes::theme_generator::create_theme_from_image;
use crate::system::themes::utils::dir_to_title;
use crate::ui::app_view::ActivePage;
use crate::ui::text::selectable;

pub struct ThemeCreationProgressDialog {
    theme_name: String,
    image_path: PathBuf,
    status_message: String,
}

impl ThemeCreationProgressDialog {
    pub fn new(theme_name: String, image_path: PathBuf) -> Self {
        Self {
            theme_name,
            image_path,
            status_message: "Analyzing image…".to_string(),
        }
    }

    /// Runs the creation once, off the UI thread. The opener closes the
    /// dialog either way: on success the editor opens, on failure the
    /// error is reported (the generator removes the half-made folder).
    fn start_creation(&mut self, cx: &mut Context<Self>) {
        let theme_name = self.theme_name.clone();
        let image_path = self.image_path.clone();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { create_theme_from_image(&image_path, &theme_name) })
                .await;

            this.update(cx, |this, cx| match result {
                Ok(created_name) => {
                    this.status_message = format!("Created '{}'", dir_to_title(&created_name));
                    cx.emit(DialogEvent::Created(created_name));
                }
                Err(e) => cx.emit(DialogEvent::Failed(e.to_string())),
            })
            .ok();
        })
        .detach();
    }
}

/// The creation ended; the opener closes the dialog and opens the editor
/// or reports why there is none.
pub enum DialogEvent {
    Created(String),
    Failed(String),
}

impl EventEmitter<DialogEvent> for ThemeCreationProgressDialog {}

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
                Spinner::new()
                    .icon(Icon::new(IconName::Loader))
                    .with_size(px(48.0))
                    .color(theme.primary),
            )
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.foreground)
                    .child("Creating Theme from Image"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(selectable("status", self.status_message.clone())),
            )
    }
}

pub fn open_theme_creation_progress_dialog(
    theme_name: String,
    image_path: PathBuf,
    window: &mut Window,
    cx: &mut App,
) {
    // Created once, here: the dialog builder below runs on every frame, so
    // anything created inside it would start over each frame.
    let view = cx.new(|cx| {
        let mut dialog = ThemeCreationProgressDialog::new(theme_name, image_path);
        dialog.start_creation(cx);
        dialog
    });
    // Close this dialog (it is the active one: it cannot be dismissed while
    // running) before opening the editor, so the editor gets focus and no
    // later close can hit another dialog.
    window
        .subscribe(&view, cx, |_, event: &DialogEvent, window, cx| {
            window.close_dialog(cx);
            match event {
                DialogEvent::Created(name) => {
                    emit(cx, AppEvent::RefreshThemes);
                    emit(cx, AppEvent::Navigate(ActivePage::ThemeEdit(name.clone())));
                }
                DialogEvent::Failed(error) => notify::error(window, error.clone(), cx),
            }
        })
        .detach();

    window.open_dialog(cx, move |dialog_builder, _, _| {
        dialog_builder
            .overlay(true)
            .keyboard(false)
            .close_button(false)
            .overlay_closable(false)
            .child(view.clone())
    });
}
