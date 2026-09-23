use std::collections::HashMap;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize, Sizable,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    radio::Radio,
    v_flex,
};

use crate::system::themes::overrides::shell_section::{
    self, KeyGroup, KeyKind, SectionKey, is_hex_color,
};
use crate::system::themes::overrides::{self, OverrideSpec};
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::focus::FocusableSwitch;
use crate::ui::theme_edit_page::override_editors::ContentChanged;
use crate::ui::theme_edit_page::shared::{color_picker_with_clipboard, error_message, help_text};

/// Shell values that refer to the Hyprland border instead of naming a color.
const REFERENCES: &[(&str, &str)] = &[
    ("hyprland.active-border", "Window border"),
    (
        "hyprland.active-border-foreground",
        "Window border, else text color",
    ),
];

enum Field {
    Color(Entity<ColorPickerState>),
    Reference {
        value: String,
        picker: Entity<ColorPickerState>,
    },
    Input(Entity<InputState>),
    Bool(bool),
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
                Field::Input(input)
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

    fn render_field(&self, key: &SectionKey, cx: &mut Context<Self>) -> Option<AnyElement> {
        let label = humanize(&key.key);
        let id = format!("shell-{}-{}", self.section, key.key);
        let element = match self.fields.get(&key.key)? {
            Field::Color(picker) => {
                color_picker_with_clipboard(id, label, picker).into_any_element()
            }
            Field::Reference { value, picker } => {
                let custom = !value.starts_with("hyprland.");
                let name = key.key.clone();
                let options = REFERENCES
                    .iter()
                    .enumerate()
                    .map(|(ix, (reference, text))| {
                        let name = name.clone();
                        Radio::new(SharedString::from(format!("{id}-ref-{ix}")))
                            .label(*text)
                            .checked(value == reference)
                            .on_click(cx.listener(move |this, _: &bool, _, cx| {
                                if let Some(Field::Reference { value, .. }) =
                                    this.fields.get_mut(&name)
                                {
                                    *value = reference.to_string();
                                }
                                this.set(
                                    &name,
                                    Some(toml::Value::String(reference.to_string())),
                                    cx,
                                );
                            }))
                    });
                let custom_radio = Radio::new(SharedString::from(format!("{id}-ref-custom")))
                    .label("Custom color")
                    .checked(custom)
                    .on_click(cx.listener({
                        let name = name.clone();
                        let picker = picker.clone();
                        move |this, _: &bool, _, cx| {
                            let hex = picker
                                .read(cx)
                                .value()
                                .map(|c| c.to_hex())
                                .unwrap_or_else(|| "#ffffff".into());
                            let hex = hex[..hex.len().min(7)].to_lowercase();
                            if let Some(Field::Reference { value, .. }) = this.fields.get_mut(&name)
                            {
                                *value = hex.clone();
                            }
                            this.set(&name, Some(toml::Value::String(hex)), cx);
                        }
                    }));
                v_flex()
                    .gap_2()
                    .min_w(px(240.))
                    .child(Label::new(label).text_sm())
                    .child(v_flex().gap_1().children(options).child(custom_radio))
                    .when(custom, |col| {
                        col.child(gpui_component::color_picker::ColorPicker::new(picker))
                    })
                    .into_any_element()
            }
            Field::Input(input) => v_flex()
                .gap_2()
                .w(px(180.))
                .child(Label::new(label).text_sm())
                .child(Input::new(input).small())
                .into_any_element(),
            Field::Bool(checked) => {
                let name = key.key.clone();
                FocusableSwitch::new(SharedString::from(id))
                    .label(label)
                    .checked(*checked)
                    .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                        if let Some(Field::Bool(value)) = this.fields.get_mut(&name) {
                            *value = *checked;
                        }
                        this.set(&name, Some(toml::Value::Boolean(*checked)), cx);
                    }))
                    .into_any_element()
            }
        };
        Some(element)
    }

    fn render_group(&self, help: &str, keys: &[SectionKey], cx: &mut Context<Self>) -> Div {
        let muted = cx.theme().muted_foreground;
        let fields: Vec<AnyElement> = keys
            .iter()
            .filter_map(|key| self.render_field(key, cx))
            .collect();
        v_flex()
            .gap_3()
            .when(!help.is_empty(), |group| {
                group.child(help_text(help.to_string(), muted))
            })
            .when(!fields.is_empty(), |group| {
                group.child(
                    h_flex()
                        .gap_x_10()
                        .gap_y_4()
                        .flex_wrap()
                        .items_start()
                        .children(fields),
                )
            })
    }
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let Some(groups) = self.groups.clone() else {
            return v_flex()
                .children(self.error.clone().map(|e| error_message(e, cx)))
                .when(self.error.is_none(), |this| {
                    this.child(help_text("Reading the section's settings…", muted))
                });
        };

        let extra = self.extra.clone();
        v_flex()
            .gap_6()
            .children(
                groups
                    .iter()
                    .map(|group| self.render_group(&group.help, &group.keys, cx)),
            )
            .when(!extra.is_empty(), |form| {
                form.child(self.render_group(
                    "Other keys this file sets, which Omarchy's template does not have.",
                    &extra,
                    cx,
                ))
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
}
