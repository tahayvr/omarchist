//! Ready-made actions: steps with a form instead of a command. Each one
//! is a row of [`ACTIONS`]: its fields, and what runs with their values.
//!
//! Values reach a shell action as environment variables (`$ARG_<KEY>`)
//! that its fixed script quotes, and a Hyprland action as escaped string
//! literals, so nothing a field or a variable holds is ever run as code.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::vars;

/// What an action's field holds in a flow file: text, a whole number, or
/// a switch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    Flag(bool),
    Number(i64),
    Text(String),
}

impl Arg {
    pub fn text(&self) -> String {
        match self {
            Arg::Flag(value) => value.to_string(),
            Arg::Number(value) => value.to_string(),
            Arg::Text(value) => value.clone(),
        }
    }
}

pub type Args = BTreeMap<String, Arg>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionGroup {
    Text,
    Apps,
    Desktop,
    Capture,
    Web,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// A line of text; takes variables.
    Text,
    /// A whole number from `min` to `max`.
    Number {
        min: i64,
        max: i64,
        step: i64,
        unit: &'static str,
    },
    /// One of a few named values: `(value, label)`.
    Choice(&'static [(&'static str, &'static str)]),
    /// An installed app; the value is the class of its windows.
    App,
    /// An installed Omarchy theme, by name.
    Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
    /// The value a new step starts with, and the one used when a file
    /// leaves the field out.
    pub default: &'static str,
    /// Whether the field may be left empty.
    pub optional: bool,
    pub placeholder: &'static str,
}

#[derive(Clone, Copy)]
pub enum Run {
    /// A shell script that reads its fields from `$ARG_<KEY>`.
    Shell(&'static str),
    /// A Hyprland dispatcher call; `{key}` marks where a field goes,
    /// always inside a string literal.
    Lua(&'static str),
    /// Rust, for work that needs no other program.
    Native(fn(&BTreeMap<&'static str, String>) -> String),
}

#[derive(Clone, Copy)]
pub struct ActionDef {
    /// What a flow file calls it; never changes.
    pub id: &'static str,
    pub label: &'static str,
    pub group: ActionGroup,
    pub icon: &'static str,
    /// Extra words the step search matches.
    pub keywords: &'static str,
    pub fields: &'static [Field],
    /// The name a new step saves the action's output under; empty when
    /// the action produces none.
    pub saves_as: &'static str,
    /// The step's title in the list; `{key}` stands for a field's value.
    pub title: &'static str,
    /// Programs the action needs, shown as missing when they are.
    pub requires: &'static [&'static str],
    pub run: Run,
}

impl std::fmt::Debug for ActionDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionDef").field("id", &self.id).finish()
    }
}

impl PartialEq for ActionDef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ActionDef {}

const fn text(key: &'static str, label: &'static str, placeholder: &'static str) -> Field {
    Field {
        key,
        label,
        kind: FieldKind::Text,
        default: "",
        optional: false,
        placeholder,
    }
}

const fn optional(field: Field) -> Field {
    Field {
        optional: true,
        ..field
    }
}

const fn choice(
    key: &'static str,
    label: &'static str,
    choices: &'static [(&'static str, &'static str)],
) -> Field {
    Field {
        key,
        label,
        kind: FieldKind::Choice(choices),
        default: choices[0].0,
        optional: false,
        placeholder: "",
    }
}

const fn percent(key: &'static str, label: &'static str, min: i64, default: &'static str) -> Field {
    Field {
        key,
        label,
        kind: FieldKind::Number {
            min,
            max: 100,
            step: 5,
            unit: "%",
        },
        default,
        optional: false,
        placeholder: "",
    }
}

const ON_OFF: &[(&str, &str)] = &[("on", "On"), ("off", "Off")];
const THE_TEXT: Field = text("text", "Text", "Text, usually a variable");

/// Every ready-made action, in the order the step list offers them.
pub const ACTIONS: &[ActionDef] = &[
    // Text and clipboard
    ActionDef {
        id: "text",
        label: "Text",
        group: ActionGroup::Text,
        icon: "icons/type.svg",
        keywords: "compose combine build string join variable",
        fields: &[text("text", "Text", "Text, with variables")],
        saves_as: "text",
        title: "Text {text}",
        requires: &[],
        run: Run::Native(|args| args["text"].clone()),
    },
    ActionDef {
        id: "clipboard.set",
        label: "Copy to the clipboard",
        group: ActionGroup::Text,
        icon: "icons/clipboard-copy.svg",
        keywords: "set put paste",
        fields: &[THE_TEXT],
        saves_as: "",
        title: "Copy {text} to the clipboard",
        requires: &["wl-copy"],
        run: Run::Shell(r#"printf '%s' "$ARG_TEXT" | wl-copy"#),
    },
    ActionDef {
        id: "clipboard.get",
        label: "Get the clipboard",
        group: ActionGroup::Text,
        icon: "icons/clipboard-paste.svg",
        keywords: "read paste copied now",
        fields: &[],
        saves_as: "copied",
        title: "Get the clipboard",
        requires: &["wl-paste"],
        run: Run::Shell("wl-paste --no-newline 2>/dev/null || true"),
    },
    ActionDef {
        id: "text.case",
        label: "Change case",
        group: ActionGroup::Text,
        icon: "icons/case-sensitive.svg",
        keywords: "upper lower title capital letters",
        fields: &[
            THE_TEXT,
            choice(
                "to",
                "To",
                &[
                    ("upper", "UPPERCASE"),
                    ("lower", "lowercase"),
                    ("title", "Title Case"),
                ],
            ),
        ],
        saves_as: "text",
        title: "Change {text} to {to}",
        requires: &[],
        run: Run::Native(|args| change_case(&args["text"], &args["to"])),
    },
    ActionDef {
        id: "text.replace",
        label: "Replace text",
        group: ActionGroup::Text,
        icon: "icons/replace.svg",
        keywords: "find substitute swap remove",
        fields: &[
            THE_TEXT,
            text("find", "Find", ""),
            optional(text("with", "Replace with", "Nothing")),
        ],
        saves_as: "text",
        title: "Replace {find} with {with} in {text}",
        requires: &[],
        run: Run::Native(|args| args["text"].replace(args["find"].as_str(), &args["with"])),
    },
    ActionDef {
        id: "text.trim",
        label: "Trim text",
        group: ActionGroup::Text,
        icon: "icons/scissors.svg",
        keywords: "strip spaces whitespace clean",
        fields: &[THE_TEXT],
        saves_as: "text",
        title: "Trim {text}",
        requires: &[],
        run: Run::Native(|args| args["text"].trim().to_string()),
    },
    ActionDef {
        id: "text.split",
        label: "Split into lines",
        group: ActionGroup::Text,
        icon: "icons/list.svg",
        keywords: "separate comma list items each",
        fields: &[
            THE_TEXT,
            Field {
                default: ",",
                ..text("by", "At every", "")
            },
        ],
        saves_as: "lines",
        title: "Split {text} at every {by}",
        requires: &[],
        run: Run::Native(|args| split_lines(&args["text"], &args["by"])),
    },
    // Apps
    ActionDef {
        id: "open.link",
        label: "Open a link",
        group: ActionGroup::Apps,
        icon: "icons/link.svg",
        keywords: "url website browser web page",
        fields: &[text("url", "Link", "https://example.com")],
        saves_as: "",
        title: "Open {url}",
        requires: &["xdg-open"],
        run: Run::Shell(r#"xdg-open "$ARG_URL""#),
    },
    ActionDef {
        id: "open.path",
        label: "Open a file or folder",
        group: ActionGroup::Apps,
        icon: "icons/folder-open.svg",
        keywords: "document directory path show",
        fields: &[text("path", "File or folder", "~/Documents")],
        saves_as: "",
        title: "Open {path}",
        requires: &["xdg-open"],
        run: Run::Shell(
            r#"path="$ARG_PATH"
case "$path" in "~/"*) path="$HOME/${path#"~/"}" ;; "~") path="$HOME" ;; esac
xdg-open "$path""#,
        ),
    },
    ActionDef {
        id: "app.focus",
        label: "Focus an app",
        group: ActionGroup::Apps,
        icon: "icons/scan-eye.svg",
        keywords: "switch to window bring front",
        fields: &[Field {
            kind: FieldKind::App,
            ..text("app", "App", "")
        }],
        saves_as: "",
        title: "Focus {app}",
        requires: &[],
        run: Run::Lua(r#"hl.dsp.focus({ window = "class:{app}" })"#),
    },
    ActionDef {
        id: "app.close",
        label: "Close an app",
        group: ActionGroup::Apps,
        icon: "icons/circle-x.svg",
        keywords: "quit window exit kill",
        fields: &[Field {
            kind: FieldKind::App,
            ..text("app", "App", "")
        }],
        saves_as: "",
        title: "Close {app}",
        requires: &[],
        run: Run::Lua(r#"hl.dsp.window.close({ window = "class:{app}" })"#),
    },
    // Desktop
    ActionDef {
        id: "volume.set",
        label: "Set the volume",
        group: ActionGroup::Desktop,
        icon: "icons/volume-2.svg",
        keywords: "sound audio loud quiet level",
        fields: &[percent("level", "Volume", 0, "30")],
        saves_as: "",
        title: "Set the volume to {level}%",
        requires: &["pactl"],
        run: Run::Shell(r#"pactl set-sink-volume "$(omarchy-audio-output-sink)" "${ARG_LEVEL}%""#),
    },
    ActionDef {
        id: "volume.mute",
        label: "Mute the sound",
        group: ActionGroup::Desktop,
        icon: "icons/volume-x.svg",
        keywords: "audio silence unmute speakers",
        fields: &[choice(
            "state",
            "Sound",
            &[("on", "Mute"), ("off", "Unmute")],
        )],
        saves_as: "",
        title: "{state} the sound",
        requires: &["pactl"],
        run: Run::Shell(
            r#"[ "$ARG_STATE" = on ] && mute=1 || mute=0
pactl set-sink-mute "$(omarchy-audio-output-sink)" "$mute""#,
        ),
    },
    ActionDef {
        id: "mic.mute",
        label: "Mute the microphone",
        group: ActionGroup::Desktop,
        icon: "icons/mic-off.svg",
        keywords: "audio input unmute meeting",
        fields: &[choice(
            "state",
            "Microphone",
            &[("on", "Mute"), ("off", "Unmute")],
        )],
        saves_as: "",
        title: "{state} the microphone",
        requires: &["wpctl"],
        run: Run::Shell(
            r#"[ "$ARG_STATE" = on ] && mute=1 || mute=0
wpctl set-mute @DEFAULT_AUDIO_SOURCE@ "$mute""#,
        ),
    },
    ActionDef {
        id: "brightness.set",
        label: "Set the brightness",
        group: ActionGroup::Desktop,
        icon: "icons/sun.svg",
        keywords: "screen display dim bright backlight",
        fields: &[percent("level", "Brightness", 1, "50")],
        saves_as: "",
        title: "Set the brightness to {level}%",
        requires: &["omarchy-brightness-display"],
        run: Run::Shell(r#"omarchy-brightness-display "${ARG_LEVEL}%""#),
    },
    ActionDef {
        id: "nightlight.set",
        label: "Night light",
        group: ActionGroup::Desktop,
        icon: "icons/moon.svg",
        keywords: "warm screen temperature evening blue light",
        fields: &[choice("state", "Night light", ON_OFF)],
        saves_as: "",
        title: "Turn the night light {state}",
        requires: &["omarchy-toggle-nightlight"],
        run: Run::Shell(
            r#"omarchy-toggle-nightlight --status | grep -q '"enabled":true' && now=on || now=off
[ "$now" = "$ARG_STATE" ] || omarchy-toggle-nightlight"#,
        ),
    },
    ActionDef {
        id: "dnd.set",
        label: "Do not disturb",
        group: ActionGroup::Desktop,
        icon: "icons/bell-off.svg",
        keywords: "notifications silence quiet focus",
        fields: &[choice("state", "Do not disturb", ON_OFF)],
        saves_as: "",
        title: "Turn do not disturb {state}",
        requires: &["omarchy-shell"],
        run: Run::Shell(
            r#"omarchy-shell notifications setDnd "$ARG_STATE" && omarchy-shell -q omarchy.indicators refresh"#,
        ),
    },
    ActionDef {
        id: "awake.set",
        label: "Stay awake",
        group: ActionGroup::Desktop,
        icon: "icons/coffee.svg",
        keywords: "idle sleep lock caffeine stay",
        fields: &[choice("state", "Stay awake", ON_OFF)],
        saves_as: "",
        title: "Turn stay awake {state}",
        requires: &["omarchy-toggle-idle"],
        run: Run::Shell(
            r#"[ "$ARG_STATE" = on ] && omarchy-toggle-idle stay-awake || omarchy-toggle-idle allow-idle"#,
        ),
    },
    ActionDef {
        id: "bar.set",
        label: "Show or hide the bar",
        group: ActionGroup::Desktop,
        icon: "icons/panel-top.svg",
        keywords: "top panel status waybar",
        fields: &[choice("state", "Bar", &[("on", "Show"), ("off", "Hide")])],
        saves_as: "",
        title: "{state} the bar",
        requires: &["omarchy-toggle-bar"],
        run: Run::Shell(r#"omarchy-toggle-bar "$ARG_STATE""#),
    },
    ActionDef {
        id: "power.profile",
        label: "Power profile",
        group: ActionGroup::Desktop,
        icon: "icons/gauge.svg",
        keywords: "battery saver performance balanced energy",
        fields: &[choice(
            "profile",
            "Profile",
            &[
                ("power-saver", "Power saver"),
                ("balanced", "Balanced"),
                ("performance", "Performance"),
            ],
        )],
        saves_as: "",
        title: "Switch to the {profile} power profile",
        requires: &["omarchy-powerprofiles-set"],
        run: Run::Shell(r#"omarchy-powerprofiles-set autodetect "$ARG_PROFILE""#),
    },
    ActionDef {
        id: "wallpaper.next",
        label: "Next wallpaper",
        group: ActionGroup::Desktop,
        icon: "icons/image.svg",
        keywords: "background picture cycle",
        fields: &[],
        saves_as: "",
        title: "Next wallpaper",
        requires: &["omarchy-theme-bg-next"],
        run: Run::Shell("omarchy-theme-bg-next"),
    },
    ActionDef {
        id: "theme.set",
        label: "Switch theme",
        group: ActionGroup::Desktop,
        icon: "icons/palette.svg",
        keywords: "colors look appearance style",
        fields: &[Field {
            kind: FieldKind::Theme,
            ..text("theme", "Theme", "")
        }],
        saves_as: "",
        title: "Switch to the {theme} theme",
        requires: &["omarchy-theme-set"],
        run: Run::Shell(r#"omarchy-theme-set "$ARG_THEME""#),
    },
    // Capture
    ActionDef {
        id: "capture.screenshot",
        label: "Take a screenshot",
        group: ActionGroup::Capture,
        icon: "icons/camera.svg",
        keywords: "screen picture image grab snip",
        fields: &[
            choice(
                "of",
                "Of",
                &[
                    ("region", "A region"),
                    ("windows", "A window"),
                    ("fullscreen", "The whole screen"),
                ],
            ),
            choice(
                "then",
                "Then",
                &[
                    ("copy", "Copy it"),
                    ("save", "Save it"),
                    ("slurp", "Edit it"),
                ],
            ),
        ],
        saves_as: "",
        title: "Take a screenshot of {of}",
        requires: &["omarchy-capture-screenshot"],
        run: Run::Shell(r#"omarchy-capture-screenshot "$ARG_OF" "$ARG_THEN""#),
    },
    ActionDef {
        id: "capture.record",
        label: "Screen recording",
        group: ActionGroup::Capture,
        icon: "icons/video.svg",
        keywords: "record video screencast capture",
        fields: &[choice(
            "state",
            "Recording",
            &[("on", "Start"), ("off", "Stop")],
        )],
        saves_as: "",
        title: "{state} a screen recording",
        requires: &["omarchy-capture-screenrecording"],
        run: Run::Shell(
            r#"if [ "$ARG_STATE" = on ]; then
  pgrep -f '^gpu-screen-recorder' >/dev/null || omarchy-capture-screenrecording
else
  omarchy-capture-screenrecording --stop-recording
fi"#,
        ),
    },
    ActionDef {
        id: "capture.text",
        label: "Text from the screen",
        group: ActionGroup::Capture,
        icon: "icons/scan-text.svg",
        keywords: "ocr read recognize extract",
        fields: &[],
        saves_as: "screen text",
        title: "Text from the screen",
        requires: &["slurp", "grim", "tesseract"],
        run: Run::Shell(
            r#"hyprpicker -r -z >/dev/null 2>&1 &
freeze=$!
trap 'kill $freeze 2>/dev/null' EXIT
sleep .1
region=$(slurp 2>/dev/null)
[ -n "$region" ] || exit 0
grim -g "$region" - | tesseract stdin stdout --oem 1 --psm 6 -l "${OMARCHY_OCR_LANGS:-eng}" --dpi 300 -c preserve_interword_spaces=1 2>/dev/null"#,
        ),
    },
    ActionDef {
        id: "capture.color",
        label: "Pick a color",
        group: ActionGroup::Capture,
        icon: "icons/pipette.svg",
        keywords: "colour hex eyedropper picker",
        fields: &[],
        saves_as: "color",
        title: "Pick a color",
        requires: &["hyprpicker"],
        run: Run::Shell("hyprpicker -r -l 2>/dev/null"),
    },
    // Web
    ActionDef {
        id: "web.search",
        label: "Search the web",
        group: ActionGroup::Web,
        icon: "icons/search.svg",
        keywords: "duckduckgo google look up find",
        fields: &[text("query", "Search for", "Text, usually a variable")],
        saves_as: "",
        title: "Search the web for {query}",
        requires: &["xdg-open", "jq"],
        run: Run::Shell(
            r#"xdg-open "https://duckduckgo.com/?q=$(printf '%s' "$ARG_QUERY" | jq -sRr @uri)""#,
        ),
    },
    ActionDef {
        id: "web.get",
        label: "Get a link's contents",
        group: ActionGroup::Web,
        icon: "icons/cloud-download.svg",
        keywords: "download fetch url http curl api request",
        fields: &[text("url", "Link", "https://example.com/data.json")],
        saves_as: "page",
        title: "Get the contents of {url}",
        requires: &["curl"],
        run: Run::Shell(r#"curl -fsSL --max-time 20 -- "$ARG_URL""#),
    },
    ActionDef {
        id: "json.get",
        label: "Get a value from JSON",
        group: ActionGroup::Web,
        icon: "icons/braces.svg",
        keywords: "parse field key jq api data",
        fields: &[
            text("json", "JSON", "Text, usually a variable"),
            text("path", "Value", ".name"),
        ],
        saves_as: "value",
        title: "Get {path} from {json}",
        requires: &["jq"],
        // The path goes through a file: jq before 1.8 has no `--`, and a
        // path starting with `-` would otherwise be read as an option.
        run: Run::Shell(
            r#"f=$(mktemp) && printf '%s' "$ARG_PATH" > "$f" && printf '%s' "$ARG_JSON" | jq -r -f "$f"; s=$?; rm -f "$f"; exit $s"#,
        ),
    },
];

pub fn find(id: &str) -> Option<&'static ActionDef> {
    ACTIONS.iter().find(|action| action.id == id)
}

fn change_case(text: &str, to: &str) -> String {
    match to {
        "upper" => text.to_uppercase(),
        "lower" => text.to_lowercase(),
        _ => {
            let mut out = String::with_capacity(text.len());
            let mut start = true;
            for c in text.chars() {
                if start && c.is_alphanumeric() {
                    out.extend(c.to_uppercase());
                    start = false;
                } else {
                    out.extend(c.to_lowercase());
                    if !c.is_alphanumeric() && c != '\'' {
                        start = true;
                    }
                }
            }
            out
        }
    }
}

/// The parts of `text` between each `by`, trimmed, one per line; empty
/// parts are dropped. `\n` and `\t` in `by` mean a line break and a tab.
fn split_lines(text: &str, by: &str) -> String {
    let by = by.replace("\\n", "\n").replace("\\t", "\t");
    if by.is_empty() {
        return text.to_string();
    }
    text.split(by.as_str())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

impl ActionDef {
    /// Whether the action produces output a later step can use.
    pub fn has_output(&self) -> bool {
        !self.saves_as.is_empty()
    }

    pub fn field(&self, key: &str) -> Option<&'static Field> {
        self.fields.iter().find(|field| field.key == key)
    }

    /// The value of `field` as a flow file gives it, or its default.
    pub fn raw(&self, field: &Field, args: &Args) -> String {
        args.get(field.key)
            .map(Arg::text)
            .unwrap_or_else(|| field.default.to_string())
    }

    /// Every `{{name}}` the fields use.
    pub fn references(&self, args: &Args) -> Vec<String> {
        self.fields
            .iter()
            .filter(|field| field.kind == FieldKind::Text)
            .flat_map(|field| vars::names_in(&self.raw(field, args)))
            .collect()
    }

    /// What is wrong with the fields, if anything: one that is missing,
    /// out of range, not one of its choices, or not a field at all.
    pub fn validate(&self, args: &Args) -> std::result::Result<(), String> {
        if let Some(unknown) = args.keys().find(|key| self.field(key).is_none()) {
            return Err(format!("'{}' has no field '{unknown}'", self.label));
        }
        for field in self.fields {
            let value = self.raw(field, args);
            if value.trim().is_empty() {
                if field.optional {
                    continue;
                }
                return Err(format!("{} is missing", field.label.to_lowercase()));
            }
            match field.kind {
                FieldKind::Number { min, max, .. } => match value.trim().parse::<i64>() {
                    Ok(n) if (min..=max).contains(&n) => {}
                    _ => {
                        return Err(format!(
                            "{} goes from {min} to {max}",
                            field.label.to_lowercase()
                        ));
                    }
                },
                FieldKind::Choice(choices) => {
                    if !choices.iter().any(|(v, _)| *v == value) {
                        return Err(format!(
                            "'{value}' is not a choice for {}",
                            field.label.to_lowercase()
                        ));
                    }
                }
                FieldKind::Text | FieldKind::App | FieldKind::Theme => {}
            }
        }
        Ok(())
    }

    /// The fields as a flow file holds them, from what a form collected:
    /// numbers as numbers, the rest as text, optional empty ones left out.
    pub fn args(&self, values: &BTreeMap<&'static str, String>) -> Args {
        let mut args = Args::new();
        for field in self.fields {
            let value = values.get(field.key).cloned().unwrap_or_default();
            if value.is_empty() && field.optional {
                continue;
            }
            let arg = match field.kind {
                FieldKind::Number { .. } => match value.trim().parse::<i64>() {
                    Ok(n) => Arg::Number(n),
                    Err(_) => Arg::Text(value),
                },
                _ => Arg::Text(value),
            };
            args.insert(field.key.to_string(), arg);
        }
        args
    }

    /// How a field's value reads in a sentence: a choice by its label in
    /// lower case, anything else as it is.
    pub fn display(&self, field: &Field, args: &Args) -> String {
        let value = self.raw(field, args);
        match field.kind {
            FieldKind::Choice(choices) => choices
                .iter()
                .find(|(v, _)| *v == value)
                .map(|(_, label)| label.to_string())
                .unwrap_or(value),
            _ => value,
        }
    }

    /// The step's title with its fields filled in. `show` turns a field's
    /// value into what the list shows (an app's name for its class).
    pub fn title_with(&self, args: &Args, show: &dyn Fn(&Field, String) -> String) -> String {
        let mut title = self.title.to_string();
        for field in self.fields {
            let marker = format!("{{{}}}", field.key);
            if !title.contains(&marker) {
                continue;
            }
            let mut value = show(field, self.display(field, args));
            // A choice reads as part of the sentence unless it starts it.
            if matches!(field.kind, FieldKind::Choice(_)) && !title.starts_with(&marker) {
                value = lower_first(&value);
            }
            if value.trim().is_empty() {
                value = "nothing".to_string();
            }
            title = title.replace(&marker, &value);
        }
        title
    }

    pub fn title(&self, args: &Args) -> String {
        self.title_with(args, &|_, value| value)
    }
}

/// "Power saver" as "power saver"; "UPPERCASE" stays, being a sample of
/// itself.
fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    match (chars.next(), chars.clone().next()) {
        (Some(first), Some(second)) if second.is_lowercase() || second == ' ' => {
            first.to_lowercase().chain(chars).collect()
        }
        _ => text.to_string(),
    }
}

/// The Hyprland call of a `Run::Lua` action with its fields in place,
/// each escaped for the string literal it sits in.
pub fn lua_call(template: &str, values: &BTreeMap<&'static str, String>) -> Result<String> {
    let mut expr = template.to_string();
    for (key, value) in values {
        let marker = format!("{{{key}}}");
        let escaped: String = value
            .chars()
            .filter(|c| !c.is_control())
            .flat_map(|c| match c {
                '\\' => vec!['\\', '\\'],
                '"' => vec!['\\', '"'],
                c => vec![c],
            })
            .collect();
        expr = expr.replace(&marker, &escaped);
    }
    if crate::system::keybinds::overrides::is_dsp_call(&expr) {
        Ok(expr)
    } else {
        Err(Error::Invalid(
            "The action is not a plain Hyprland call once its fields are filled in".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(pairs: &[(&str, Arg)]) -> Args {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn ids_are_unique_and_fields_are_sound() {
        let mut ids: Vec<&str> = ACTIONS.iter().map(|a| a.id).collect();
        let total = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), total, "an action id is used twice");
        for action in ACTIONS {
            let mut keys: Vec<&str> = action.fields.iter().map(|f| f.key).collect();
            let count = keys.len();
            keys.sort();
            keys.dedup();
            assert_eq!(
                keys.len(),
                count,
                "{}: a field key is used twice",
                action.id
            );
            for field in action.fields {
                // Every field the title or the script names exists.
                if let FieldKind::Choice(choices) = field.kind {
                    assert!(
                        choices.iter().any(|(v, _)| *v == field.default),
                        "{}: default of {} is not a choice",
                        action.id,
                        field.key
                    );
                }
            }
            let mut rest = action.title;
            while let Some(open) = rest.find('{') {
                let close = rest[open..].find('}').expect("a closed marker") + open;
                let key = &rest[open + 1..close];
                assert!(
                    action.field(key).is_some(),
                    "{}: the title names an unknown field {key}",
                    action.id
                );
                rest = &rest[close + 1..];
            }
            match action.run {
                Run::Shell(script) => {
                    for field in action.fields {
                        let var = format!("ARG_{}", field.key.to_uppercase());
                        assert!(
                            script.contains(&var),
                            "{}: the script never reads ${var}",
                            action.id
                        );
                    }
                }
                Run::Lua(template) => {
                    let values: BTreeMap<&'static str, String> = action
                        .fields
                        .iter()
                        .map(|f| (f.key, "x".to_string()))
                        .collect();
                    assert!(lua_call(template, &values).is_ok(), "{}", action.id);
                }
                Run::Native(_) => {}
            }
        }
    }

    #[test]
    fn every_shell_action_is_valid_shell() {
        for action in ACTIONS {
            let Run::Shell(script) = action.run else {
                continue;
            };
            let status = std::process::Command::new("sh")
                .args(["-n", "-c", script])
                .status()
                .unwrap();
            assert!(status.success(), "{}: the script does not parse", action.id);
        }
    }

    #[test]
    fn fields_are_checked() {
        let volume = find("volume.set").unwrap();
        assert!(volume.validate(&Args::new()).is_ok(), "the default is fine");
        assert!(
            volume
                .validate(&args(&[("level", Arg::Number(40))]))
                .is_ok()
        );
        assert!(
            volume
                .validate(&args(&[("level", Arg::Number(140))]))
                .is_err()
        );
        assert!(
            volume
                .validate(&args(&[("level", Arg::Text("loud".into()))]))
                .is_err()
        );
        assert!(volume.validate(&args(&[("lvl", Arg::Number(4))])).is_err());

        let copy = find("clipboard.set").unwrap();
        assert!(copy.validate(&Args::new()).is_err(), "text is required");
        let replace = find("text.replace").unwrap();
        assert!(
            replace
                .validate(&args(&[
                    ("text", Arg::Text("a".into())),
                    ("find", Arg::Text("b".into()))
                ]))
                .is_ok(),
            "'with' may be empty"
        );
        let profile = find("power.profile").unwrap();
        assert!(
            profile
                .validate(&args(&[("profile", Arg::Text("turbo".into()))]))
                .is_err()
        );
    }

    #[test]
    fn titles_read_as_sentences() {
        let title = |id: &str, pairs: &[(&str, Arg)]| find(id).unwrap().title(&args(pairs));
        assert_eq!(
            title("volume.set", &[("level", Arg::Number(40))]),
            "Set the volume to 40%"
        );
        assert_eq!(title("volume.set", &[]), "Set the volume to 30%");
        assert_eq!(
            title("volume.mute", &[("state", Arg::Text("off".into()))]),
            "Unmute the sound"
        );
        assert_eq!(
            title("nightlight.set", &[("state", Arg::Text("on".into()))]),
            "Turn the night light on"
        );
        assert_eq!(
            title(
                "power.profile",
                &[("profile", Arg::Text("power-saver".into()))]
            ),
            "Switch to the power saver power profile"
        );
        assert_eq!(
            title("text.case", &[("text", Arg::Text("{{name}}".into()))]),
            "Change {{name}} to UPPERCASE"
        );
        assert_eq!(
            title(
                "text.replace",
                &[
                    ("text", Arg::Text("{{name}}".into())),
                    ("find", Arg::Text("-".into()))
                ]
            ),
            "Replace - with nothing in {{name}}"
        );
    }

    #[test]
    fn text_actions_work_without_a_shell() {
        assert_eq!(
            change_case("hello wORLD, it's me", "title"),
            "Hello World, It's Me"
        );
        assert_eq!(change_case("Hello", "upper"), "HELLO");
        assert_eq!(change_case("Hello", "lower"), "hello");
        assert_eq!(split_lines("a, b,,c ", ","), "a\nb\nc");
        assert_eq!(split_lines("a b", " "), "a\nb");
        assert_eq!(
            split_lines("a\\tb", "\\t"),
            "a\\tb",
            "the text itself is literal"
        );
        assert_eq!(split_lines("a\tb", "\\t"), "a\nb");
    }

    #[test]
    fn a_form_becomes_typed_fields() {
        let volume = find("volume.set").unwrap();
        let values: BTreeMap<&'static str, String> = [("level", "45".to_string())].into();
        assert_eq!(volume.args(&values), args(&[("level", Arg::Number(45))]));
        let replace = find("text.replace").unwrap();
        let values: BTreeMap<&'static str, String> = [
            ("text", "{{a}}".to_string()),
            ("find", "x".to_string()),
            ("with", String::new()),
        ]
        .into();
        assert_eq!(
            replace.args(&values),
            args(&[
                ("find", Arg::Text("x".into())),
                ("text", Arg::Text("{{a}}".into()))
            ])
        );
    }

    #[test]
    fn a_hyprland_action_escapes_its_fields() {
        let values: BTreeMap<&'static str, String> =
            [("app", "a\" }), os.execute(\"x".to_string())].into();
        let expr = lua_call(r#"hl.dsp.focus({ window = "class:{app}" })"#, &values).unwrap();
        assert_eq!(
            expr,
            r#"hl.dsp.focus({ window = "class:a\" }), os.execute(\"x" })"#
        );
    }
}
