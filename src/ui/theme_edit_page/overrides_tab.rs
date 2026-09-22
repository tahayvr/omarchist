use crate::system::themes::overrides::{self, OverrideSpec, btop, chromium, shell_section};
use crate::system::themes::theme_management::update_theme;
use crate::types::themes::{ColorsConfig, EditingTheme};
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::focus::FocusableSwitch;
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, error_message, focus_section, form_section, help_text,
    tab_container,
};
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    separator::Separator,
    v_flex,
};

// Per-app overrides for files Omarchy would otherwise generate from
// colors.toml. Each section has a switch: off means no file on disk and
// Omarchy's template tracks the palette; on writes the file Omarchy would
// have generated, which the user then edits field by field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Override {
    Browser,
    Lock,
    Btop,
}

impl Override {
    const ALL: [Override; 3] = [Override::Browser, Override::Lock, Override::Btop];

    fn index(self) -> usize {
        self as usize
    }

    fn spec(self) -> Option<&'static OverrideSpec> {
        overrides::find(match self {
            Override::Browser => "chromium.theme",
            Override::Lock => "shell.lock.toml",
            Override::Btop => "btop.theme",
        })
    }

    fn fields(self) -> &'static [Field] {
        match self {
            Override::Browser => BROWSER_FIELDS,
            Override::Lock => LOCK_FIELDS,
            Override::Btop => BTOP_FIELDS,
        }
    }

    fn get(self, content: &str, key: &str) -> Option<String> {
        match self {
            Override::Browser => chromium::to_hex(content),
            Override::Lock => shell_section::get(content, "lock", key),
            Override::Btop => btop::get(content, key),
        }
    }

    fn set(self, content: &str, key: &str, hex: &str) -> Option<String> {
        match self {
            Override::Browser => chromium::from_hex(hex),
            Override::Lock => Some(shell_section::set(content, "lock", key, hex)),
            Override::Btop => Some(btop::set(content, key, hex)),
        }
    }
}

// One color field of an override file: the key it edits and where it sits in
// the form.
struct Field {
    id: &'static str,
    label: &'static str,
    group: &'static str,
    key: &'static str,
}

const fn field(
    id: &'static str,
    label: &'static str,
    group: &'static str,
    key: &'static str,
) -> Field {
    Field {
        id,
        label,
        group,
        key,
    }
}

const BROWSER_FIELDS: &[Field] = &[field("browser-theme", "Theme Color", "Chromium", "")];

const LOCK_FIELDS: &[Field] = &[
    field("lock-text", "Text", "Text", "text"),
    field("lock-placeholder", "Placeholder", "Text", "placeholder"),
    field("lock-text-error", "Text Error", "Text", "text-error"),
    field("lock-border", "Border", "Border", "border"),
    field(
        "lock-border-active",
        "Border Active",
        "Border",
        "border-active",
    ),
    field(
        "lock-border-error",
        "Border Error",
        "Border",
        "border-error",
    ),
];

const BTOP_FIELDS: &[Field] = &[
    field("btop-main-bg", "Background", "Main Colors", "main_bg"),
    field("btop-main-fg", "Foreground", "Main Colors", "main_fg"),
    field("btop-title", "Title", "Main Colors", "title"),
    field("btop-hi-fg", "Highlight", "Main Colors", "hi_fg"),
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

pub struct OverridesTab {
    theme_name: String,
    theme_data: EditingTheme,
    // The theme's own file per override, by `Override::index`; `None` when
    // Omarchy generates it.
    contents: [Option<String>; 3],
    // Pickers parallel to each override's field table; empty when it is off.
    pickers: [Vec<Entity<ColorPickerState>>; 3],
    // Free-text because Hyprland border specs can be gradients
    // ("rgba(..ee) rgba(..ee) 45deg"), which a color picker cannot express.
    active_border_input: Entity<InputState>,
    inactive_border_input: Entity<InputState>,
    error_message: Option<String>,
    scroll: ScrollHandle,
}

impl OverridesTab {
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        scroll: &ScrollHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let active_border_input = Self::border_input(
            window,
            cx,
            theme_data.colors.hyprland_active_border.as_deref(),
            "Default: accent color",
            |c, v| c.hyprland_active_border = v,
        );
        let inactive_border_input = Self::border_input(
            window,
            cx,
            theme_data.colors.hyprland_inactive_border.as_deref(),
            "Default: rgba(595959aa)",
            |c, v| c.hyprland_inactive_border = v,
        );
        let contents = Override::ALL.map(|kind| {
            kind.spec()
                .and_then(|spec| overrides::read(&theme_name, spec).ok().flatten())
        });

        let mut tab = Self {
            theme_name,
            theme_data,
            contents,
            pickers: Default::default(),
            active_border_input,
            inactive_border_input,
            error_message: None,
            scroll: scroll.clone(),
        };
        for kind in Override::ALL {
            tab.rebuild_pickers(kind, window, cx);
        }
        tab
    }

    fn border_input(
        window: &mut Window,
        cx: &mut Context<Self>,
        value: Option<&str>,
        placeholder: &str,
        setter: fn(&mut ColorsConfig, Option<String>),
    ) -> Entity<InputState> {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder.to_string())
                .default_value(value.unwrap_or_default().to_string())
        });

        cx.subscribe_in(
            &input,
            window,
            move |this, input, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    let raw = input.read(cx).value().to_string();
                    let trimmed = raw.trim();
                    let value = (!trimmed.is_empty()).then(|| trimmed.to_string());
                    setter(&mut this.theme_data.colors, value);
                    this.save_borders(cx);
                }
            },
        )
        .detach();

        input
    }

    fn is_enabled(&self, kind: Override) -> bool {
        self.contents[kind.index()].is_some()
    }

    fn rebuild_pickers(&mut self, kind: Override, window: &mut Window, cx: &mut Context<Self>) {
        let pickers = match &self.contents[kind.index()] {
            Some(content) => kind
                .fields()
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let hex = kind.get(content, field.key).unwrap_or_default();
                    Self::picker(kind, index, &hex, window, cx)
                })
                .collect(),
            None => Vec::new(),
        };
        self.pickers[kind.index()] = pickers;
    }

    fn picker(
        kind: Override,
        index: usize,
        hex: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<ColorPickerState> {
        let color = hex_to_hsla(hex).unwrap_or(gpui::rgb(0x33A1FF).into());
        let picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(color));

        cx.subscribe_in(
            &picker,
            window,
            move |this, _picker, event: &ColorPickerEvent, _window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    this.set_field(kind, index, &color.to_hex(), cx);
                }
            },
        )
        .detach();

        picker
    }

    fn set_field(&mut self, kind: Override, index: usize, hex: &str, cx: &mut Context<Self>) {
        let (Some(spec), Some(content)) = (kind.spec(), &self.contents[kind.index()]) else {
            return;
        };
        let Some(updated) = kind.set(content, kind.fields()[index].key, hex) else {
            return;
        };
        match overrides::write(&self.theme_name, spec, &updated) {
            Ok(()) => {
                self.contents[kind.index()] = Some(updated);
                self.error_message = None;
            }
            Err(e) => self.error_message = Some(e.to_string()),
        }
        cx.notify();
    }

    // Turning an override on writes exactly what Omarchy would generate from
    // the current palette; turning it off deletes the file.
    fn set_enabled(
        &mut self,
        kind: Override,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(spec) = kind.spec() else {
            return;
        };
        let theme = &self.theme_name;
        let result = if enabled {
            overrides::generated(theme, spec)
                .and_then(|content| overrides::write(theme, spec, &content).map(|()| content))
                .map(Some)
        } else {
            overrides::remove(theme, spec).map(|()| None)
        };
        match result {
            Ok(content) => {
                self.contents[kind.index()] = content;
                self.error_message = None;
            }
            Err(e) => self.error_message = Some(e.to_string()),
        }
        self.rebuild_pickers(kind, window, cx);
        cx.notify();
    }

    fn save_borders(&mut self, cx: &mut Context<Self>) {
        let (active_border, inactive_border) = (
            self.theme_data.colors.hyprland_active_border.clone(),
            self.theme_data.colors.hyprland_inactive_border.clone(),
        );
        let result = update_theme(&self.theme_name, |theme| {
            theme.colors.hyprland_active_border = active_border;
            theme.colors.hyprland_inactive_border = inactive_border;
        });
        self.error_message = result.err().map(|e| e.to_string());
        cx.notify();
    }

    fn render_section(
        &self,
        kind: Override,
        title: &'static str,
        description: &'static str,
        cx: &mut Context<Self>,
    ) -> Div {
        let enabled = self.is_enabled(kind);
        let fields = kind.fields();
        let pickers = &self.pickers[kind.index()];
        let switch_id: SharedString = format!("override-{}", title.to_lowercase()).into();

        let header = h_flex()
            .gap_4()
            .items_start()
            .justify_between()
            .flex_wrap()
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(title),
                    )
                    .child(help_text(description, cx.theme().muted_foreground)),
            )
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        Label::new(if enabled {
                            "Custom colors"
                        } else {
                            "Generated by Omarchy"
                        })
                        .text_sm()
                        .text_color(cx.theme().muted_foreground),
                    )
                    .child(FocusableSwitch::new(switch_id).checked(enabled).on_change(
                        cx.listener(move |this, checked, window, cx| {
                            this.set_enabled(kind, *checked, window, cx);
                        }),
                    )),
            );

        let mut section = form_section().gap_4().child(header);

        if enabled {
            // Group pickers by their `group` label, preserving table order.
            let mut groups: Vec<(&'static str, Vec<AnyElement>)> = Vec::new();
            for (field, picker) in fields.iter().zip(pickers) {
                let element =
                    color_picker_with_clipboard(field.id, field.label, picker).into_any_element();
                match groups.iter_mut().find(|(name, _)| *name == field.group) {
                    Some((_, items)) => items.push(element),
                    None => groups.push((field.group, vec![element])),
                }
            }

            let single_group = groups.len() == 1;
            section = section.children(groups.into_iter().map(|(name, items)| {
                let mut group = v_flex().gap_3();
                if !single_group {
                    group = group.child(Label::new(name).text_sm().font_weight(FontWeight::MEDIUM));
                }
                group.child(h_flex().gap_24().flex_wrap().children(items))
            }));
        }

        section
    }
}

impl Render for OverridesTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border_input = |label: &'static str, state: &Entity<InputState>| {
            v_flex()
                .gap_2()
                .flex_1()
                .min_w(px(220.))
                .child(Label::new(label).text_sm())
                .child(Input::new(state).cleanable(true))
        };
        let borders = form_section()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Window Borders"),
            )
            .child(help_text(
                "Hyprland border colors. Omarchy uses the accent color for the active border \
                 and a neutral grey for inactive ones unless you override them. Any Hyprland \
                 color works, including gradients such as rgba(26a269ee) rgba(2ec27eee) 45deg.",
                cx.theme().muted_foreground,
            ))
            .child(
                h_flex()
                    .gap_6()
                    .flex_wrap()
                    .child(border_input("Active Border", &self.active_border_input))
                    .child(border_input("Inactive Border", &self.inactive_border_input)),
            );

        let browser = self.render_section(
            Override::Browser,
            "Browser",
            "Chromium's theme color. Omarchy uses the theme background unless you override it.",
            cx,
        );
        let lock = self.render_section(
            Override::Lock,
            "Lock Screen",
            "Colors for the lock screen input. Omarchy derives them from the palette and \
             accent unless you override them.",
            cx,
        );
        let btop = self.render_section(
            Override::Btop,
            "Btop",
            "Colors for the btop system monitor. Omarchy maps the palette onto btop's \
             graphs and boxes unless you override it.",
            cx,
        );

        tab_container()
            .child(help_text(
                "Omarchy generates these configs from your palette every time the theme is \
                 applied. Fill in an override only when something needs colors that differ \
                 from the generated ones; clearing it or turning it off returns to the palette.",
                cx.theme().muted_foreground,
            ))
            .children(
                self.error_message
                    .as_ref()
                    .map(|msg| error_message(msg.clone(), cx)),
            )
            .child(
                v_flex()
                    .gap_6()
                    .child(focus_section("overrides-borders", &self.scroll, borders))
                    .child(Separator::horizontal())
                    .child(focus_section("overrides-browser", &self.scroll, browser))
                    .child(Separator::horizontal())
                    .child(focus_section("overrides-lock", &self.scroll, lock))
                    .child(Separator::horizontal())
                    .child(focus_section("overrides-btop", &self.scroll, btop)),
            )
    }
}
