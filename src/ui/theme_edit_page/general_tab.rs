use crate::system::themes::theme_management::{rename_theme, update_theme};
use crate::types::themes::EditingTheme;
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::focus::FocusableSwitch;
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, error_message, focus_section, form_section, help_text,
    tab_container,
};
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize, Disableable, Sizable,
    button::Button,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
};

pub struct GeneralTab {
    theme_data: EditingTheme,
    original_theme_name: String, // Used for saving - folder name doesn't change on rename
    name_input: Entity<InputState>,
    author_input: Entity<InputState>,
    accent_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
    scroll: ScrollHandle,
}

impl GeneralTab {
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        scroll: &ScrollHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Store the folder name for saving (not the display name from JSON)
        let original_theme_name = theme_name;

        let author_value = theme_data.author.clone().unwrap_or_default();

        let name_input = cx.new(|cx| InputState::new(window, cx).default_value(&theme_data.name));

        let author_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter author name...")
                .default_value(&author_value)
        });

        let accent_color =
            hex_to_hsla(&theme_data.colors.accent).unwrap_or(gpui::rgb(0x33A1FF).into());
        let accent_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(accent_color));

        let tab = Self {
            theme_data,
            original_theme_name,
            name_input,
            author_input,
            accent_picker,
            is_saving: false,
            error_message: None,
            scroll: scroll.clone(),
        };

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

        cx.subscribe_in(
            &tab.accent_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.theme_data.colors.accent = hex;
                    this.save(window, cx);
                }
            },
        )
        .detach();

        tab
    }

    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    fn save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        if self.original_theme_name.is_empty() {
            self.error_message = Some("Theme name cannot be empty".to_string());
            cx.notify();
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        // Save using the ORIGINAL theme name (folder name); the display name
        // lives in theme_data.name. Only this tab's fields are written so a
        // stale snapshot never overwrites another tab's edits.
        let (name, author, accent, is_light) = (
            self.theme_data.name.clone(),
            self.theme_data.author.clone(),
            self.theme_data.colors.accent.clone(),
            self.theme_data.is_light_theme,
        );
        match update_theme(&self.original_theme_name, |theme| {
            theme.name = name;
            theme.author = author;
            theme.colors.accent = accent;
            theme.is_light_theme = is_light;
        }) {
            Ok(()) => {
                self.is_saving = false;
            }
            Err(e) => {
                self.is_saving = false;
                self.error_message = Some(e.to_string());
            }
        }

        cx.notify();
    }

    fn on_light_mode_toggle(&mut self, checked: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.theme_data.is_light_theme = checked;
        self.save(window, cx);
    }

    fn rename_theme(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let new_name = self.theme_data.name.clone();
        let old_name = self.original_theme_name.clone();

        if new_name == old_name || new_name.is_empty() {
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        match rename_theme(&old_name, &new_name) {
            Ok(()) => {
                self.is_saving = false;
                self.original_theme_name = new_name.clone();
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_light = self.theme_data.is_light_theme;
        let _viewport_width = window.viewport_size().width;

        let current_name = self.name_input.read(cx).value().to_string();
        let can_rename = current_name != self.original_theme_name && !current_name.is_empty();

        tab_container()
            .child(focus_section(
                "general-name",
                &self.scroll,
                form_section()
                    .child(
                        Label::new("Theme Name")
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .child(
                                div()
                                    .w_80()
                                    .child(Input::new(&self.name_input).cleanable(true)),
                            )
                            .child(
                                Button::new("rename-btn")
                                    .label("Rename")
                                    .small()
                                    .disabled(!can_rename)
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.rename_theme(window, cx);
                                    })),
                            ),
                    ),
            ))
            .child(focus_section(
                "general-author",
                &self.scroll,
                form_section()
                    .child(
                        Label::new("Author")
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        div()
                            .w_80()
                            .child(Input::new(&self.author_input).cleanable(true)),
                    ),
            ))
            .child(focus_section(
                "general-accent",
                &self.scroll,
                form_section().child(color_picker_with_clipboard(
                    "accent-color",
                    "Accent Color",
                    &self.accent_picker,
                )),
            ))
            .child(focus_section(
                "general-light",
                &self.scroll,
                FocusableSwitch::new("light-theme-toggle")
                    .label("Light Theme")
                    .checked(is_light)
                    .on_change(cx.listener(|this, checked, window, cx| {
                        this.on_light_mode_toggle(*checked, window, cx);
                    })),
            ))
            .child(help_text(
                "Themes are in dark mode by default.",
                cx.theme().muted_foreground,
            ))
            .children(
                self.error_message
                    .as_ref()
                    .map(|msg| error_message(msg.clone(), cx)),
            )
    }
}
