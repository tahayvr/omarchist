use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
    input::{Editor, EditorState, Input, InputEvent, InputState},
    label::Label,
    radio::Radio,
    v_flex,
};

use crate::system::themes::overrides::{OverrideSpec, btop, chromium, shell_section};
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::theme_edit_page::shared::color_picker_with_clipboard;

/// Emitted with the file's full new content after every edit.
pub struct ContentChanged(pub String);

/// The editor an override pane shows while the override is on.
pub enum OverrideEditor {
    Source(Entity<SourceEditor>),
    Colors(Entity<ColorFieldsForm>),
    Icons(Entity<IconsForm>),
    Vscode(Entity<VscodeForm>),
}

impl OverrideEditor {
    pub fn new(
        spec: &'static OverrideSpec,
        content: &str,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        match spec.file {
            "btop.theme" => Self::Colors(
                cx.new(|cx| ColorFieldsForm::new(Codec::Btop, BTOP_FIELDS, content, window, cx)),
            ),
            "chromium.theme" => Self::Colors(cx.new(|cx| {
                ColorFieldsForm::new(Codec::Chromium, CHROMIUM_FIELDS, content, window, cx)
            })),
            "keyboard.rgb" => Self::Colors(cx.new(|cx| {
                ColorFieldsForm::new(Codec::Plain, KEYBOARD_FIELDS, content, window, cx)
            })),
            "icons.theme" => Self::Icons(cx.new(|_| IconsForm::new(content))),
            "vscode.json" => Self::Vscode(cx.new(|cx| VscodeForm::new(content, window, cx))),
            _ => Self::Source(cx.new(|cx| SourceEditor::new(spec, content, window, cx))),
        }
    }

    /// Calls `f` with each edit's new content.
    pub fn subscribe<T: 'static>(
        &self,
        cx: &mut Context<T>,
        f: impl Fn(&mut T, &ContentChanged, &mut Context<T>) + 'static,
    ) -> Subscription {
        let f = Rc::new(f);
        macro_rules! forward {
            ($editor:expr) => {{
                let f = f.clone();
                cx.subscribe($editor, move |this, _, event: &ContentChanged, cx| {
                    f(this, event, cx)
                })
            }};
        }
        match self {
            Self::Source(editor) => forward!(editor),
            Self::Colors(editor) => forward!(editor),
            Self::Icons(editor) => forward!(editor),
            Self::Vscode(editor) => forward!(editor),
        }
    }

    pub fn element(&self) -> AnyElement {
        match self {
            Self::Source(editor) => editor.clone().into_any_element(),
            Self::Colors(editor) => editor.clone().into_any_element(),
            Self::Icons(editor) => editor.clone().into_any_element(),
            Self::Vscode(editor) => editor.clone().into_any_element(),
        }
    }
}

// MARK: Source

/// The file as text, highlighted when the highlighter knows its language.
pub struct SourceEditor {
    input: Entity<EditorState>,
    _subscription: Subscription,
}

impl EventEmitter<ContentChanged> for SourceEditor {}

impl SourceEditor {
    fn new(
        spec: &'static OverrideSpec,
        content: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            let state = EditorState::new(window, cx).line_number(true);
            match spec.format.language() {
                Some(language) => state.language(language),
                None => state,
            }
            .default_value(content.to_string())
        });
        let subscription = cx.subscribe(&input, |_, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                cx.emit(ContentChanged(input.read(cx).value().to_string()));
            }
        });
        Self {
            input,
            _subscription: subscription,
        }
    }
}

impl Render for SourceEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().h(px(420.)).child(
            Editor::new(&self.input)
                .bg(cx.theme().background)
                .border_1()
                .border_color(cx.theme().border)
                .h_full()
                .appearance(false),
        )
    }
}

/// A read-only view of what Omarchy generates.
pub fn preview_editor(
    spec: &'static OverrideSpec,
    content: &str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<EditorState> {
    cx.new(|cx| {
        let state = EditorState::new(window, cx).line_number(true);
        match spec.format.language() {
            Some(language) => state.language(language),
            None => state,
        }
        .default_value(content.to_string())
    })
}

// MARK: Color fields

/// How a form reads and writes one key of the file.
#[derive(Clone, Copy)]
pub enum Codec {
    /// `theme[key]="value"` lines.
    Btop,
    /// A single `R,G,B` value; the key is ignored.
    Chromium,
    /// A single `#rrggbb` value; the key is ignored.
    Plain,
    /// `key = "value"` inside one `shell.toml` section.
    ShellSection(&'static str),
}

impl Codec {
    fn get(self, content: &str, key: &str) -> Option<String> {
        match self {
            Codec::Btop => btop::get(content, key),
            Codec::Chromium => chromium::to_hex(content),
            Codec::Plain => Some(content.trim().to_string()),
            Codec::ShellSection(section) => shell_section::get(content, section, key),
        }
    }

    fn set(self, content: &str, key: &str, hex: &str) -> Option<String> {
        match self {
            Codec::Btop => Some(btop::set(content, key, hex)),
            Codec::Chromium => chromium::from_hex(hex),
            Codec::Plain => Some(format!("{}\n", &hex[..hex.len().min(7)])),
            Codec::ShellSection(section) => Some(shell_section::set(content, section, key, hex)),
        }
    }
}

pub struct ColorField {
    pub id: &'static str,
    pub label: &'static str,
    pub group: &'static str,
    pub key: &'static str,
}

const fn field(
    id: &'static str,
    label: &'static str,
    group: &'static str,
    key: &'static str,
) -> ColorField {
    ColorField {
        id,
        label,
        group,
        key,
    }
}

const CHROMIUM_FIELDS: &[ColorField] = &[field("chromium-theme", "Toolbar Color", "", "")];

const KEYBOARD_FIELDS: &[ColorField] = &[field("keyboard-rgb", "Backlight Color", "", "")];

const BTOP_FIELDS: &[ColorField] = &[
    field("btop-main-bg", "Background", "Main Colors", "main_bg"),
    field("btop-main-fg", "Foreground", "Main Colors", "main_fg"),
    field("btop-title", "Title", "Main Colors", "title"),
    field("btop-hi-fg", "Highlight", "Main Colors", "hi_fg"),
    field("btop-graph-text", "Graph Text", "Main Colors", "graph_text"),
    field(
        "btop-meter-bg",
        "Meter Background",
        "Main Colors",
        "meter_bg",
    ),
    field(
        "btop-selected-bg",
        "Selected Background",
        "Selection Colors",
        "selected_bg",
    ),
    field(
        "btop-selected-fg",
        "Selected Foreground",
        "Selection Colors",
        "selected_fg",
    ),
    field(
        "btop-inactive-fg",
        "Inactive",
        "Status Colors",
        "inactive_fg",
    ),
    field("btop-proc-misc", "Proc Misc", "Status Colors", "proc_misc"),
    field("btop-cpu-box", "CPU Box", "Box Outline Colors", "cpu_box"),
    field(
        "btop-mem-box",
        "Memory Box",
        "Box Outline Colors",
        "mem_box",
    ),
    field("btop-net-box", "Net Box", "Box Outline Colors", "net_box"),
    field(
        "btop-proc-box",
        "Proc Box",
        "Box Outline Colors",
        "proc_box",
    ),
    field(
        "btop-div-line",
        "Divider Line",
        "Box Outline Colors",
        "div_line",
    ),
    field(
        "btop-temp-start",
        "Start",
        "Temperature Graph",
        "temp_start",
    ),
    field("btop-temp-mid", "Mid", "Temperature Graph", "temp_mid"),
    field("btop-temp-end", "End", "Temperature Graph", "temp_end"),
    field("btop-cpu-start", "Start", "CPU Graph", "cpu_start"),
    field("btop-cpu-mid", "Mid", "CPU Graph", "cpu_mid"),
    field("btop-cpu-end", "End", "CPU Graph", "cpu_end"),
    field("btop-free-start", "Start", "Free Meter", "free_start"),
    field("btop-free-mid", "Mid", "Free Meter", "free_mid"),
    field("btop-free-end", "End", "Free Meter", "free_end"),
    field("btop-cached-start", "Start", "Cached Meter", "cached_start"),
    field("btop-cached-mid", "Mid", "Cached Meter", "cached_mid"),
    field("btop-cached-end", "End", "Cached Meter", "cached_end"),
    field(
        "btop-available-start",
        "Start",
        "Available Meter",
        "available_start",
    ),
    field(
        "btop-available-mid",
        "Mid",
        "Available Meter",
        "available_mid",
    ),
    field(
        "btop-available-end",
        "End",
        "Available Meter",
        "available_end",
    ),
    field("btop-used-start", "Start", "Used Meter", "used_start"),
    field("btop-used-mid", "Mid", "Used Meter", "used_mid"),
    field("btop-used-end", "End", "Used Meter", "used_end"),
    field(
        "btop-download-start",
        "Start",
        "Download Graph",
        "download_start",
    ),
    field("btop-download-mid", "Mid", "Download Graph", "download_mid"),
    field("btop-download-end", "End", "Download Graph", "download_end"),
    field("btop-upload-start", "Start", "Upload Graph", "upload_start"),
    field("btop-upload-mid", "Mid", "Upload Graph", "upload_mid"),
    field("btop-upload-end", "End", "Upload Graph", "upload_end"),
];

/// One color picker per field, grouped by the field's `group`.
pub struct ColorFieldsForm {
    codec: Codec,
    fields: &'static [ColorField],
    content: String,
    pickers: Vec<Entity<ColorPickerState>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ContentChanged> for ColorFieldsForm {}

impl ColorFieldsForm {
    pub fn new(
        codec: Codec,
        fields: &'static [ColorField],
        content: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut subscriptions = Vec::with_capacity(fields.len());
        let pickers = fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let hex = codec.get(content, field.key).unwrap_or_default();
                let picker = cx.new(|cx| {
                    let picker = ColorPickerState::new(window, cx);
                    match hex_to_hsla(&hex) {
                        Some(color) => picker.default_value(color),
                        None => picker,
                    }
                });
                subscriptions.push(cx.subscribe(
                    &picker,
                    move |this: &mut Self, _, event: &ColorPickerEvent, cx| {
                        if let ColorPickerEvent::Change(Some(color)) = event {
                            this.set(index, &color.to_hex(), cx);
                        }
                    },
                ));
                picker
            })
            .collect();

        Self {
            codec,
            fields,
            content: content.to_string(),
            pickers,
            _subscriptions: subscriptions,
        }
    }

    fn set(&mut self, index: usize, hex: &str, cx: &mut Context<Self>) {
        let Some(updated) = self.codec.set(&self.content, self.fields[index].key, hex) else {
            return;
        };
        self.content = updated.clone();
        cx.emit(ContentChanged(updated));
    }
}

impl Render for ColorFieldsForm {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut groups: Vec<(&'static str, Vec<AnyElement>)> = Vec::new();
        for (field, picker) in self.fields.iter().zip(&self.pickers) {
            let element =
                color_picker_with_clipboard(field.id, field.label, picker).into_any_element();
            match groups.iter_mut().find(|(name, _)| *name == field.group) {
                Some((_, items)) => items.push(element),
                None => groups.push((field.group, vec![element])),
            }
        }

        v_flex()
            .gap_6()
            .children(groups.into_iter().map(|(name, items)| {
                v_flex()
                    .gap_3()
                    .when(!name.is_empty(), |group| {
                        group.child(Label::new(name).text_sm().font_weight(FontWeight::MEDIUM))
                    })
                    .child(h_flex().gap_x_12().gap_y_4().flex_wrap().children(items))
            }))
    }
}

// MARK: Icons

struct YaruColor {
    value: &'static str,
    label: &'static str,
    color: u32,
}

const YARU_COLORS: &[YaruColor] = &[
    YaruColor {
        value: "Yaru-red",
        label: "Red",
        color: 0xe92020,
    },
    YaruColor {
        value: "Yaru-blue",
        label: "Blue",
        color: 0x208fe9,
    },
    YaruColor {
        value: "Yaru-olive",
        label: "Olive",
        color: 0x636B2F,
    },
    YaruColor {
        value: "Yaru-yellow",
        label: "Yellow",
        color: 0xe9ba20,
    },
    YaruColor {
        value: "Yaru-purple",
        label: "Purple",
        color: 0x5e2750,
    },
    YaruColor {
        value: "Yaru-magenta",
        label: "Magenta",
        color: 0xFF00FF,
    },
    YaruColor {
        value: "Yaru-sage",
        label: "Sage",
        color: 0x123d18,
    },
];

/// The Yaru icon color, one radio per variant.
pub struct IconsForm {
    selected: String,
}

impl EventEmitter<ContentChanged> for IconsForm {}

impl IconsForm {
    fn new(content: &str) -> Self {
        Self {
            selected: content.trim().to_string(),
        }
    }
}

impl Render for IconsForm {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_x_8()
            .gap_y_3()
            .flex_wrap()
            .children(YARU_COLORS.iter().map(|yaru| {
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Radio::new(yaru.value)
                            .label(yaru.label)
                            .checked(self.selected == yaru.value)
                            .on_click(cx.listener(move |this, _: &bool, _, cx| {
                                this.selected = yaru.value.to_string();
                                cx.emit(ContentChanged(format!("{}\n", yaru.value)));
                                cx.notify();
                            })),
                    )
                    .child(div().size_5().bg(rgb(yaru.color)).rounded_sm())
            }))
    }
}

// MARK: VS Code extension

/// `vscode.json`: a Marketplace extension and the theme name it provides.
pub struct VscodeForm {
    name: Entity<InputState>,
    extension: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ContentChanged> for VscodeForm {}

impl VscodeForm {
    fn new(content: &str, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let value: serde_json::Value = serde_json::from_str(content).unwrap_or_default();
        let field = |key: &str| {
            value
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        let name = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Tokyo Night")
                .default_value(field("name"))
        });
        let extension = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("enkia.tokyo-night")
                .default_value(field("extension"))
        });
        let on_change = |this: &mut Self, _, event: &InputEvent, cx: &mut Context<Self>| {
            if let InputEvent::Change = event {
                let content = serde_json::json!({
                    "name": this.name.read(cx).value().trim(),
                    "extension": this.extension.read(cx).value().trim(),
                });
                let content = serde_json::to_string_pretty(&content).unwrap_or_default();
                cx.emit(ContentChanged(content + "\n"));
            }
        };
        let subscriptions = vec![
            cx.subscribe(&name, on_change),
            cx.subscribe(&extension, on_change),
        ];
        Self {
            name,
            extension,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for VscodeForm {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let input = |label: &'static str, help: &'static str, state: &Entity<InputState>| {
            v_flex()
                .gap_2()
                .flex_1()
                .min_w(px(220.))
                .child(Label::new(label).text_sm())
                .child(Input::new(state))
                .child(div().text_xs().text_color(muted).child(help))
        };
        h_flex()
            .gap_6()
            .flex_wrap()
            .items_start()
            .child(input(
                "Extension",
                "The Marketplace id, shown as publisher.name on the extension's page.",
                &self.extension,
            ))
            .child(input(
                "Theme Name",
                "The color theme's name as VS Code lists it.",
                &self.name,
            ))
    }
}
