use std::collections::HashMap;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize, Sizable,
    button::Button,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    menu::{DropdownMenu, PopupMenuItem},
    v_flex,
};

use crate::system::themes::overrides::shell_section::{
    self, KeyGroup, KeyKind, SectionKey, is_hex_color,
};
use crate::system::themes::overrides::{self, OverrideSpec};
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::focus::FocusableSwitch;
use crate::ui::theme_edit_page::override_editors::ContentChanged;
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, error_message, field_grid, field_label, group_title, help_text,
    pane_grid_columns,
};

/// Shell values that refer to the Hyprland border instead of naming a color.
const REFERENCES: &[(&str, &str)] = &[
    ("hyprland.active-border", "Window border"),
    (
        "hyprland.active-border-foreground",
        "Border, else text color",
    ),
];

enum Field {
    Color(Entity<ColorPickerState>),
    Reference {
        value: String,
        picker: Entity<ColorPickerState>,
    },
    Input(Entity<InputState>),
    Number(Entity<InputState>),
    Bool(bool),
}

/// The +/- step and the allowed range of a number key.
fn number_range(kind: KeyKind) -> (f64, f64, f64) {
    match kind {
        KeyKind::Alpha => (0.05, 0.0, 1.0),
        KeyKind::Float => (0.1, 0.0, 100.0),
        _ => (1.0, 0.0, 999.0),
    }
}

fn format_number(kind: KeyKind, value: f64) -> String {
    match kind {
        KeyKind::Integer => format!("{}", value.round() as i64),
        _ => {
            let text = format!("{value:.2}");
            let text = text.trim_end_matches('0');
            if text.ends_with('.') {
                format!("{text}0")
            } else {
                text.to_string()
            }
        }
    }
}

/// A form for one `shell.<section>.toml`, with a field for every key of the
/// section in Omarchy's template, grouped and described by its comments.
pub struct ShellSectionForm {
    section: &'static str,
    content: String,
    /// `None` until the generated section has been read.
    groups: Option<Vec<KeyGroup>>,
    /// Keys the file sets that the template does not have.
    extra: Vec<SectionKey>,
    fields: HashMap<String, Field>,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ContentChanged> for ShellSectionForm {}

impl ShellSectionForm {
    pub fn new(
        theme_name: String,
        spec: &'static OverrideSpec,
        content: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let section = spec.shell_section().unwrap_or_default();
        cx.spawn_in(window, async move |this, cx| {
            let generated = cx
                .background_spawn(async move { overrides::generated(&theme_name, spec) })
                .await;
            this.update_in(cx, |this, window, cx| {
                match generated {
                    Ok(text) => this.build(shell_section::schema(&text), window, cx),
                    Err(e) => this.error = Some(e.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();

        Self {
            section,
            content: content.to_string(),
            groups: None,
            extra: Vec::new(),
            fields: HashMap::new(),
            error: None,
            _subscriptions: Vec::new(),
        }
    }

    fn build(&mut self, groups: Vec<KeyGroup>, window: &mut Window, cx: &mut Context<Self>) {
        let known: Vec<&str> = groups
            .iter()
            .flat_map(|g| g.keys.iter().map(|k| k.key.as_str()))
            .collect();
        self.extra = shell_section::keys(&self.content, self.section)
            .into_iter()
            .filter(|key| !known.contains(&key.as_str()))
            .filter_map(|key| {
                let value = shell_section::get_value(&self.content, self.section, &key)?;
                Some(SectionKey {
                    kind: KeyKind::Text,
                    key,
                    default: value,
                    optional: true,
                })
            })
            .collect();

        let keys: Vec<SectionKey> = groups
            .iter()
            .flat_map(|g| g.keys.iter().cloned())
            .chain(self.extra.iter().cloned())
            .collect();
        for key in &keys {
            let field = self.field(key, window, cx);
            self.fields.insert(key.key.clone(), field);
        }
        self.groups = Some(groups);
    }

    fn current(&self, key: &SectionKey) -> Option<toml::Value> {
        shell_section::get_value(&self.content, self.section, &key.key)
            .or_else(|| (!key.optional).then(|| key.default.clone()))
    }

    fn field(&mut self, key: &SectionKey, window: &mut Window, cx: &mut Context<Self>) -> Field {
        let current = self.current(key);
        let text = |value: &Option<toml::Value>| match value {
            Some(toml::Value::String(s)) => s.clone(),
            Some(value) => value.to_string(),
            None => String::new(),
        };

        match key.kind {
            KeyKind::Color => Field::Color(self.picker(&key.key, &text(&current), window, cx)),
            KeyKind::Reference => {
                let value = text(&current);
                let picker = self.picker(&key.key, &value, window, cx);
                Field::Reference { value, picker }
            }
            KeyKind::Bool => Field::Bool(matches!(current, Some(toml::Value::Boolean(true)))),
            KeyKind::Alpha | KeyKind::Integer | KeyKind::Float | KeyKind::Text => {
                let placeholder = if key.optional {
                    match &key.default {
                        toml::Value::String(s) => s.clone(),
                        value => value.to_string(),
                    }
                } else {
                    String::new()
                };
                let input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder(placeholder)
                        .default_value(text(&current))
                });
                let name = key.key.clone();
                let (kind, optional) = (key.kind, key.optional);
                self._subscriptions.push(cx.subscribe(
                    &input,
                    move |this: &mut Self, input, event: &InputEvent, cx| {
                        if let InputEvent::Change = event {
                            let raw = input.read(cx).value().trim().to_string();
                            this.set_text(&name, kind, optional, &raw, cx);
                        }
                    },
                ));
                if kind == KeyKind::Text {
                    return Field::Input(input);
                }
                let name = key.key.clone();
                let fallback = match &key.default {
                    toml::Value::Integer(n) => *n as f64,
                    toml::Value::Float(n) => *n,
                    _ => 0.0,
                };
                self._subscriptions.push(cx.subscribe_in(
                    &input,
                    window,
                    move |this: &mut Self, input, event: &NumberInputEvent, window, cx| {
                        let NumberInputEvent::Step(action) = event;
                        let (step, min, max) = number_range(kind);
                        let current = input
                            .read(cx)
                            .value()
                            .trim()
                            .parse::<f64>()
                            .unwrap_or(fallback);
                        let next = match action {
                            StepAction::Increment => current + step,
                            StepAction::Decrement => current - step,
                        };
                        let text = format_number(kind, next.clamp(min, max));
                        input.update(cx, |input, cx| input.set_value(text.clone(), window, cx));
                        this.set_text(&name, kind, optional, &text, cx);
                    },
                ));
                Field::Number(input)
            }
        }
    }

    fn picker(
        &mut self,
        key: &str,
        hex: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<ColorPickerState> {
        let picker = cx.new(|cx| {
            let picker = ColorPickerState::new(window, cx);
            match hex_to_hsla(hex).filter(|_| is_hex_color(hex)) {
                Some(color) => picker.default_value(color),
                None => picker,
            }
        });
        let name = key.to_string();
        self._subscriptions.push(cx.subscribe(
            &picker,
            move |this: &mut Self, _, event: &ColorPickerEvent, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    let hex = hex[..hex.len().min(7)].to_lowercase();
                    if let Some(Field::Reference { value, .. }) = this.fields.get_mut(&name) {
                        *value = hex.clone();
                    }
                    this.set(&name, Some(toml::Value::String(hex)), cx);
                }
            },
        ));
        picker
    }

    fn set_text(
        &mut self,
        key: &str,
        kind: KeyKind,
        optional: bool,
        raw: &str,
        cx: &mut Context<Self>,
    ) {
        if raw.is_empty() {
            if optional {
                self.set(key, None, cx);
            }
            return;
        }
        let value = match kind {
            KeyKind::Integer => raw.parse::<i64>().ok().map(toml::Value::Integer),
            KeyKind::Alpha => raw
                .parse::<f64>()
                .ok()
                .filter(|a| (0.0..=1.0).contains(a))
                .map(toml::Value::Float),
            KeyKind::Float => raw.parse::<f64>().ok().map(toml::Value::Float),
            _ => Some(toml::Value::String(raw.to_string())),
        };
        if let Some(value) = value {
            self.set(key, Some(value), cx);
        }
    }

    fn set(&mut self, key: &str, value: Option<toml::Value>, cx: &mut Context<Self>) {
        self.content = match value {
            Some(value) => shell_section::set_value(&self.content, self.section, key, &value),
            None => shell_section::unset(&self.content, self.section, key),
        };
        cx.emit(ContentChanged(self.content.clone()));
        cx.notify();
    }

    fn set_reference(&mut self, key: &str, value: String, cx: &mut Context<Self>) {
        if let Some(Field::Reference { value: current, .. }) = self.fields.get_mut(key) {
            *current = value.clone();
        }
        self.set(key, Some(toml::Value::String(value)), cx);
    }

    fn render_field(&self, key: &SectionKey, cx: &mut Context<Self>) -> Option<AnyElement> {
        let label = humanize(&key.key);
        let id = format!("shell-{}-{}", self.section, key.key);
        let element = match self.fields.get(&key.key)? {
            Field::Color(picker) => {
                color_picker_with_clipboard(id, label, picker).into_any_element()
            }
            Field::Reference { value, picker } => {
                let custom = !value.starts_with("hyprland.");
                let current = value.clone();
                let current_label = REFERENCES
                    .iter()
                    .find(|(reference, _)| *reference == value)
                    .map(|(_, label)| *label)
                    .unwrap_or("Custom color");
                let name = key.key.clone();
                let view = cx.entity();
                let picker = picker.clone();
                let menu_picker = picker.clone();
                v_flex()
                    .gap_2()
                    .items_start()
                    .child(field_label(label, None))
                    .child(
                        Button::new(SharedString::from(format!("{id}-ref")))
                            .label(current_label)
                            .dropdown_caret(true)
                            .outline()
                            .small()
                            .cursor_pointer()
                            .dropdown_menu(move |menu, _, _| {
                                let menu =
                                    REFERENCES.iter().fold(menu, |menu, (reference, text)| {
                                        let view = view.clone();
                                        let name = name.clone();
                                        menu.item(
                                            PopupMenuItem::new(*text)
                                                .checked(current == *reference)
                                                .on_click(move |_, _, cx| {
                                                    view.update(cx, |this, cx| {
                                                        this.set_reference(
                                                            &name,
                                                            reference.to_string(),
                                                            cx,
                                                        )
                                                    });
                                                }),
                                        )
                                    });
                                let view = view.clone();
                                let name = name.clone();
                                let picker = menu_picker.clone();
                                menu.item(
                                    PopupMenuItem::new("Custom color").checked(custom).on_click(
                                        move |_, _, cx| {
                                            let hex = picker
                                                .read(cx)
                                                .value()
                                                .map(|c| c.to_hex())
                                                .unwrap_or_else(|| "#ffffff".into());
                                            let hex = hex[..hex.len().min(7)].to_lowercase();
                                            view.update(cx, |this, cx| {
                                                this.set_reference(&name, hex, cx)
                                            });
                                        },
                                    ),
                                )
                            }),
                    )
                    .when(custom, |col| col.child(ColorPicker::new(&picker)))
                    .into_any_element()
            }
            Field::Input(input) => v_flex()
                .gap_2()
                .child(field_label(label, None))
                .child(Input::new(input).small())
                .into_any_element(),
            Field::Number(input) => v_flex()
                .gap_2()
                .child(field_label(label, None))
                .child(NumberInput::new(input).small())
                .into_any_element(),
            Field::Bool(checked) => {
                let name = key.key.clone();
                v_flex()
                    .gap_2()
                    .child(field_label("", None))
                    .child(
                        FocusableSwitch::new(SharedString::from(id))
                            .label(label)
                            .checked(*checked)
                            .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                                if let Some(Field::Bool(value)) = this.fields.get_mut(&name) {
                                    *value = *checked;
                                }
                                this.set(&name, Some(toml::Value::Boolean(*checked)), cx);
                            })),
                    )
                    .into_any_element()
            }
        };
        Some(element)
    }

    fn render_group(
        &self,
        help: &str,
        keys: &[SectionKey],
        columns: usize,
        cx: &mut Context<Self>,
    ) -> Div {
        let fields: Vec<AnyElement> = keys
            .iter()
            .filter_map(|key| self.render_field(key, cx))
            .collect();
        v_flex()
            .gap_3()
            .children(group_title_of(help).map(|title| group_title(title, cx)))
            .when(!fields.is_empty(), |group| {
                group.child(field_grid(columns, fields))
            })
    }
}

// The name a template comment gives its keys, as in "Normal: idle control
// chrome." Longer prefixes are sentences, not names.
fn group_title_of(help: &str) -> Option<String> {
    let (title, _) = help.split_once(':')?;
    (title.split_whitespace().count() <= 3 && !title.contains('.')).then(|| title.to_string())
}

/// `background-alpha` → `Background alpha`.
fn humanize(key: &str) -> String {
    let text = key.replace(['-', '_'], " ");
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => text,
    }
}

impl Render for ShellSectionForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let Some(groups) = self.groups.clone() else {
            return v_flex()
                .children(self.error.clone().map(|e| error_message(e, cx)))
                .when(self.error.is_none(), |this| {
                    this.child(help_text("Loading…", muted))
                });
        };

        let columns = pane_grid_columns(window);
        let extra = self.extra.clone();
        v_flex()
            .gap_6()
            .children(
                groups
                    .iter()
                    .filter(|group| !group.keys.is_empty())
                    .map(|group| self.render_group(&group.help, &group.keys, columns, cx)),
            )
            .when(!extra.is_empty(), |form| {
                form.child(self.render_group("", &extra, columns, cx))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::humanize;

    #[test]
    fn humanizes_keys() {
        assert_eq!(humanize("background-alpha"), "Background alpha");
        assert_eq!(humanize("base_size"), "Base size");
    }

    #[test]
    fn group_titles_come_from_short_comment_prefixes() {
        assert_eq!(
            super::group_title_of("Normal: idle control chrome.").as_deref(),
            Some("Normal")
        );
        assert_eq!(super::group_title_of("Momentary fills."), None);
        assert_eq!(
            super::group_title_of("Lock screen password input. border/border-active cycle: idle"),
            None
        );
    }
}
