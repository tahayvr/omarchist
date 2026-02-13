//! General tab for theme editing
//!
//! Provides UI for editing basic theme metadata:
//! - Theme name
//! - Author
//! - Light/Dark theme toggle (manages light.mode file)

use crate::system::theme_management::{rename_theme, save_theme_data};
use crate::types::themes::EditingTheme;
use crate::ui::theme_edit_page::shared::{error_message, form_section, help_text, tab_container};
use gpui::*;
use gpui_component::{
    button::Button,
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    switch::Switch,
    ActiveTheme, Disableable, Sizable,
};

/// General tab content for editing theme metadata
pub struct GeneralTab {
    theme_data: EditingTheme,
    original_theme_name: String, // Used for saving - folder name doesn't change on rename
    name_input: Entity<InputState>,
    author_input: Entity<InputState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl GeneralTab {
    /// Create a new GeneralTab instance
    pub fn new(theme_data: EditingTheme, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Store the original theme name for saving
        let original_theme_name = theme_data.name.clone();

        // Extract author value before moving theme_data
        let author_value = theme_data.author.clone().unwrap_or_default();

        // Create input states with current values
        let name_input = cx.new(|cx| InputState::new(window, cx).default_value(&theme_data.name));

        let author_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter author name...")
                .default_value(&author_value)
        });

        let tab = Self {
            theme_data,
            original_theme_name,
            name_input,
            author_input,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to name input changes
        cx.subscribe_in(
            &tab.name_input,
            window,
            |this, _input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event {
                    let new_name = this.name_input.read(cx).value().to_string();
                    if new_name != this.theme_data.name {
                        this.theme_data.name = new_name;
                        this.save(window, cx);
                    }
                }
            },
        )
        .detach();

        // Subscribe to author input changes
        cx.subscribe_in(
            &tab.author_input,
            window,
            |this, _input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event {
                    let author = this.author_input.read(cx).value().to_string();
                    this.theme_data.author = if author.is_empty() {
                        None
                    } else {
                        Some(author)
                    };
                    this.save(window, cx);
                }
            },
        )
        .detach();

        tab
    }

    /// Get the current theme data
    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    /// Save the theme data
    fn save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        // Don't save if theme name is empty
        if self.original_theme_name.is_empty() {
            self.error_message = Some("Theme name cannot be empty".to_string());
            cx.notify();
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        // Save theme data using the ORIGINAL theme name (folder name)
        // The new name is stored in theme_data.name but we save to the original folder
        match save_theme_data(&self.original_theme_name, &self.theme_data) {
            Ok(()) => {
                self.is_saving = false;
            }
            Err(e) => {
                self.is_saving = false;
                self.error_message = Some(e);
            }
        }

        cx.notify();
    }

    /// Handle light mode toggle
    fn on_light_mode_toggle(&mut self, checked: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.theme_data.is_light_theme = checked;
        self.save(window, cx);
    }

    /// Rename the theme folder
    fn rename_theme(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let new_name = self.theme_data.name.clone();
        let old_name = self.original_theme_name.clone();

        // Don't rename if names are the same or new name is empty
        if new_name == old_name || new_name.is_empty() {
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        match rename_theme(&old_name, &new_name) {
            Ok(()) => {
                self.is_saving = false;
                // Update the original theme name to the new name
                self.original_theme_name = new_name.clone();
                // Also update the header display
                // TODO: Notify parent that theme name changed
            }
            Err(e) => {
                self.is_saving = false;
                self.error_message = Some(format!("Failed to rename theme: {}", e));
            }
        }

        cx.notify();
    }
}

impl Render for GeneralTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_light = self.theme_data.is_light_theme;

        // Check if theme name has changed for rename button
        let current_name = self.name_input.read(cx).value().to_string();
        let can_rename = current_name != self.original_theme_name && !current_name.is_empty();

        tab_container()
            .child(
                // Theme Name Section with Rename button
                form_section()
                    .child(
                        Label::new("Theme Name")
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Input::new(&self.name_input).cleanable(true))
                            .child(
                                Button::new("rename-btn")
                                    .label("Rename")
                                    .small()
                                    .disabled(!can_rename)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.rename_theme(window, cx);
                                    })),
                            ),
                    ),
            )
            .child(
                // Author Section
                form_section()
                    .child(
                        Label::new("Author")
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(Input::new(&self.author_input).cleanable(true)),
            )
            .child(
                // Light Mode Toggle Section
                h_flex()
                    .gap_4()
                    .items_center()
                    .child(Label::new("Light Theme"))
                    .child(
                        Switch::new("light-theme-toggle")
                            .checked(is_light)
                            .on_click(cx.listener(|this, checked, window, cx| {
                                this.on_light_mode_toggle(*checked, window, cx);
                            })),
                    ),
            )
            .child(
                // Help Text
                help_text("Themes are dark mode by default."),
            )
            .children(
                self.error_message
                    .as_ref()
                    .map(|msg| error_message(msg.clone())),
            )
    }
}
