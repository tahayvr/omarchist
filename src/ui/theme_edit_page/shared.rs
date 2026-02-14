//! Shared components and utilities for theme edit tabs
//!
//! This module provides reusable UI components and patterns that all tabs
//! in the theme edit page can use, following gpui-component conventions.

use gpui::*;
use gpui_component::{
    h_flex,
    input::{Input, InputState},
    label::Label,
    switch::Switch,
    v_flex, ActiveTheme,
};

/// A reusable form field with label and input
pub struct FormField {
    label: String,
    input: Entity<InputState>,
}

impl FormField {
    /// Create a new form field with label, initial value, and placeholder
    pub fn new(
        label: &str,
        initial_value: impl Into<String>,
        placeholder: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(initial_value.into())
                .placeholder(placeholder.into())
        });

        Self {
            label: label.to_string(),
            input,
        }
    }

    /// Get the input entity for event subscription
    pub fn input(&self) -> &Entity<InputState> {
        &self.input
    }

    /// Get current value from input
    pub fn value(&self, cx: &App) -> String {
        self.input.read(cx).value().to_string()
    }
}

impl RenderOnce for FormField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(
                Label::new(&self.label)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground),
            )
            .child(Input::new(&self.input).cleanable(true))
    }
}

/// A reusable toggle field with label and switch
pub struct ToggleField {
    id: String,
    label: String,
    is_checked: bool,
    on_change: Option<Box<dyn Fn(bool, &mut Window, &mut App)>>,
}

impl ToggleField {
    /// Create a new toggle field
    pub fn new(id: &str, label: &str, is_checked: bool) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            is_checked,
            on_change: None,
        }
    }

    /// Set the change handler
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool, &mut Window, &mut App) + 'static,
    {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ToggleField {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let is_checked = self.is_checked;
        let on_change = self.on_change;
        let id: gpui::SharedString = self.id.into();

        h_flex()
            .gap_4()
            .items_center()
            .child(Label::new(&self.label))
            .child(
                Switch::new(id)
                    .checked(is_checked)
                    .on_click(move |checked, window, cx| {
                        if let Some(ref handler) = on_change {
                            handler(*checked, window, cx);
                        }
                    }),
            )
    }
}

/// Helper function to create a form section with consistent styling
pub fn form_section() -> Div {
    v_flex().gap_2()
}

/// Helper function to create a help text element
pub fn help_text(text: impl Into<SharedString>) -> Div {
    div()
        .text_sm()
        .text_color(gpui::rgb(0x888888))
        .child(text.into())
}

/// Trait for handling input changes with auto-save coordination
///
/// Tabs implementing this trait can use standardized input event handling
pub trait TabInputHandler: Sized {
    /// Called when an input field value changes
    ///
    /// # Arguments
    /// * `field` - The field that changed (use a custom enum per tab)
    /// * `value` - The new value
    /// * `window` - The window context
    /// * `cx` - The GPUI context
    fn on_input_change(
        &mut self,
        field_id: &str,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    );

    /// Trigger save operation for the tab
    fn trigger_save(&mut self, window: &mut Window, cx: &mut Context<Self>);
}

/// Standard tab container styling
pub fn tab_container() -> Div {
    v_flex().gap_6().pt_4().pb_4()
}

/// Error message display component
pub fn error_message(text: impl Into<SharedString>) -> Div {
    div()
        .p_2()
        .bg(gpui::rgb(0xffcccc))
        .border_1()
        .border_color(gpui::rgb(0xff0000))
        .child(
            div()
                .text_sm()
                .text_color(gpui::rgb(0xff0000))
                .child(text.into()),
        )
}
