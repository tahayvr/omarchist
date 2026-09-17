use crate::system::themes::theme_management::{
    default_browser_config, default_btop_config, default_lock_config, update_theme,
};
use crate::types::themes::{BrowserConfig, BtopConfig, EditingTheme, LockScreenConfig};
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
    divider::Divider,
    h_flex,
    label::Label,
    v_flex,
};

// Per-app overrides for files Omarchy would otherwise generate from
// colors.toml (`btop.theme`, `chromium.theme`, `shell.lock.toml`). Each
// section has a switch: off means no file on disk and Omarchy's template
// tracks the palette; on writes a file seeded from the current palette that
// the user can then edit field by field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Override {
    Browser,
    Lock,
    Btop,
}

// One color field of an override config: how to read it, how to write it,
// and where it sits in the form.
struct Field<C: 'static> {
    id: &'static str,
    label: &'static str,
    group: &'static str,
    get: fn(&C) -> String,
    set: fn(&mut C, String),
}

const BROWSER_FIELDS: &[Field<BrowserConfig>] = &[Field {
    id: "browser-theme",
    label: "Theme Color",
    group: "Chromium",
    get: |c| c.theme_color.clone(),
    set: |c, v| c.theme_color = v,
}];

const LOCK_FIELDS: &[Field<LockScreenConfig>] = &[
    Field {
        id: "lock-text",
        label: "Text",
        group: "Text",
        get: |c| c.text.clone(),
        set: |c, v| c.text = v,
    },
    Field {
        id: "lock-placeholder",
        label: "Placeholder",
        group: "Text",
        get: |c| c.placeholder.clone(),
        set: |c, v| c.placeholder = v,
    },
    Field {
        id: "lock-text-error",
        label: "Text Error",
        group: "Text",
        get: |c| c.text_error.clone(),
        set: |c, v| c.text_error = v,
    },
    Field {
        id: "lock-border",
        label: "Border",
        group: "Border",
        get: |c| c.border.clone(),
        set: |c, v| c.border = v,
    },
    Field {
        id: "lock-border-active",
        label: "Border Active",
        group: "Border",
        get: |c| c.border_active.clone(),
        set: |c, v| c.border_active = v,
    },
    Field {
        id: "lock-border-error",
        label: "Border Error",
        group: "Border",
        get: |c| c.border_error.clone(),
        set: |c, v| c.border_error = v,
    },
];

macro_rules! btop_field {
    ($id:literal, $label:literal, $group:literal, $field:ident) => {
        Field {
            id: $id,
            label: $label,
            group: $group,
            get: |c| c.$field.clone(),
            set: |c, v| c.$field = v,
        }
    };
}

const BTOP_FIELDS: &[Field<BtopConfig>] = &[
    btop_field!("btop-main-bg", "Background", "Main Colors", main_bg),
    btop_field!("btop-main-fg", "Foreground", "Main Colors", main_fg),
    btop_field!("btop-title", "Title", "Main Colors", title),
    btop_field!("btop-hi-fg", "Highlight", "Main Colors", hi_fg),
    btop_field!(
        "btop-selected-bg",
        "Selected Background",
        "Selection Colors",
        selected_bg
    ),
    btop_field!(
        "btop-selected-fg",
        "Selected Foreground",
        "Selection Colors",
        selected_fg
    ),
    btop_field!("btop-inactive-fg", "Inactive", "Status Colors", inactive_fg),
    btop_field!("btop-proc-misc", "Proc Misc", "Status Colors", proc_misc),
    btop_field!("btop-cpu-box", "CPU Box", "Box Outline Colors", cpu_box),
    btop_field!("btop-mem-box", "Memory Box", "Box Outline Colors", mem_box),
    btop_field!("btop-net-box", "Net Box", "Box Outline Colors", net_box),
    btop_field!("btop-proc-box", "Proc Box", "Box Outline Colors", proc_box),
    btop_field!(
        "btop-div-line",
        "Divider Line",
        "Box Outline Colors",
        div_line
    ),
    btop_field!("btop-temp-start", "Start", "Temperature Graph", temp_start),
    btop_field!("btop-temp-mid", "Mid", "Temperature Graph", temp_mid),
    btop_field!("btop-temp-end", "End", "Temperature Graph", temp_end),
    btop_field!("btop-cpu-start", "Start", "CPU Graph", cpu_start),
    btop_field!("btop-cpu-mid", "Mid", "CPU Graph", cpu_mid),
    btop_field!("btop-cpu-end", "End", "CPU Graph", cpu_end),
    btop_field!("btop-free-start", "Start", "Free Meter", free_start),
    btop_field!("btop-free-mid", "Mid", "Free Meter", free_mid),
    btop_field!("btop-free-end", "End", "Free Meter", free_end),
    btop_field!("btop-cached-start", "Start", "Cached Meter", cached_start),
    btop_field!("btop-cached-mid", "Mid", "Cached Meter", cached_mid),
    btop_field!("btop-cached-end", "End", "Cached Meter", cached_end),
    btop_field!(
        "btop-available-start",
        "Start",
        "Available Meter",
        available_start
    ),
    btop_field!(
        "btop-available-mid",
        "Mid",
        "Available Meter",
        available_mid
    ),
    btop_field!(
        "btop-available-end",
        "End",
        "Available Meter",
        available_end
    ),
    btop_field!("btop-used-start", "Start", "Used Meter", used_start),
    btop_field!("btop-used-mid", "Mid", "Used Meter", used_mid),
    btop_field!("btop-used-end", "End", "Used Meter", used_end),
    btop_field!(
        "btop-download-start",
        "Start",
        "Download Graph",
        download_start
    ),
    btop_field!("btop-download-mid", "Mid", "Download Graph", download_mid),
    btop_field!("btop-download-end", "End", "Download Graph", download_end),
    btop_field!("btop-upload-start", "Start", "Upload Graph", upload_start),
    btop_field!("btop-upload-mid", "Mid", "Upload Graph", upload_mid),
    btop_field!("btop-upload-end", "End", "Upload Graph", upload_end),
];

pub struct OverridesTab {
    theme_name: String,
    theme_data: EditingTheme,
    // Pickers are parallel to the field tables above; empty when the
    // override is off.
    browser_pickers: Vec<Entity<ColorPickerState>>,
    lock_pickers: Vec<Entity<ColorPickerState>>,
    btop_pickers: Vec<Entity<ColorPickerState>>,
    is_saving: bool,
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
        let mut tab = Self {
            theme_name,
            theme_data,
            browser_pickers: Vec::new(),
            lock_pickers: Vec::new(),
            btop_pickers: Vec::new(),
            is_saving: false,
            error_message: None,
            scroll: scroll.clone(),
        };
        tab.rebuild_pickers(Override::Browser, window, cx);
        tab.rebuild_pickers(Override::Lock, window, cx);
        tab.rebuild_pickers(Override::Btop, window, cx);
        tab
    }

    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    fn is_enabled(&self, kind: Override) -> bool {
        match kind {
            Override::Browser => self.theme_data.apps.chromium.is_some(),
            Override::Lock => self.theme_data.apps.lock.is_some(),
            Override::Btop => self.theme_data.apps.btop.is_some(),
        }
    }

    // Current hex values in field-table order, or empty when the override is off.
    fn current_values(&self, kind: Override) -> Vec<String> {
        fn values<C>(fields: &[Field<C>], config: Option<&C>) -> Vec<String> {
            config
                .map(|c| fields.iter().map(|f| (f.get)(c)).collect())
                .unwrap_or_default()
        }
        match kind {
            Override::Browser => values(BROWSER_FIELDS, self.theme_data.apps.chromium.as_ref()),
            Override::Lock => values(LOCK_FIELDS, self.theme_data.apps.lock.as_ref()),
            Override::Btop => values(BTOP_FIELDS, self.theme_data.apps.btop.as_ref()),
        }
    }

    fn rebuild_pickers(&mut self, kind: Override, window: &mut Window, cx: &mut Context<Self>) {
        let pickers: Vec<Entity<ColorPickerState>> = self
            .current_values(kind)
            .into_iter()
            .enumerate()
            .map(|(index, hex)| Self::picker(kind, index, &hex, window, cx))
            .collect();

        match kind {
            Override::Browser => self.browser_pickers = pickers,
            Override::Lock => self.lock_pickers = pickers,
            Override::Btop => self.btop_pickers = pickers,
        }
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
                    this.set_field(kind, index, color.to_hex());
                    this.save(cx);
                }
            },
        )
        .detach();

        picker
    }

    fn set_field(&mut self, kind: Override, index: usize, hex: String) {
        match kind {
            Override::Browser => {
                if let Some(c) = self.theme_data.apps.chromium.as_mut() {
                    (BROWSER_FIELDS[index].set)(c, hex);
                }
            }
            Override::Lock => {
                if let Some(c) = self.theme_data.apps.lock.as_mut() {
                    (LOCK_FIELDS[index].set)(c, hex);
                }
            }
            Override::Btop => {
                if let Some(c) = self.theme_data.apps.btop.as_mut() {
                    (BTOP_FIELDS[index].set)(c, hex);
                }
            }
        }
    }

    // Turning an override on seeds it from the palette exactly as Omarchy's
    // template would; turning it off drops the config so the file is removed
    // on save.
    fn set_enabled(
        &mut self,
        kind: Override,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let colors = &self.theme_data.colors;
        match kind {
            Override::Browser => {
                self.theme_data.apps.chromium = enabled.then(|| default_browser_config(colors));
            }
            Override::Lock => {
                self.theme_data.apps.lock = enabled.then(|| default_lock_config(colors));
            }
            Override::Btop => {
                self.theme_data.apps.btop = enabled.then(|| default_btop_config(colors));
            }
        }
        self.rebuild_pickers(kind, window, cx);
        self.save(cx);
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        if self.theme_name.is_empty() {
            self.error_message = Some("Theme name cannot be empty".to_string());
            cx.notify();
            return;
        }

        self.is_saving = true;
        self.error_message = None;

        let (chromium, lock, btop) = (
            self.theme_data.apps.chromium.clone(),
            self.theme_data.apps.lock.clone(),
            self.theme_data.apps.btop.clone(),
        );
        let result = update_theme(&self.theme_name, |theme| {
            theme.apps.chromium = chromium;
            theme.apps.lock = lock;
            theme.apps.btop = btop;
        });

        if let Err(e) = result {
            self.error_message = Some(e.to_string());
        }

        self.is_saving = false;
        cx.notify();
    }

    fn render_section<C: 'static>(
        &self,
        kind: Override,
        title: &'static str,
        description: &'static str,
        fields: &'static [Field<C>],
        pickers: &[Entity<ColorPickerState>],
        cx: &mut Context<Self>,
    ) -> Div {
        let enabled = self.is_enabled(kind);
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
        let browser = self.render_section(
            Override::Browser,
            "Browser",
            "Chromium's theme color. Omarchy uses the theme background unless you override it.",
            BROWSER_FIELDS,
            &self.browser_pickers,
            cx,
        );
        let lock = self.render_section(
            Override::Lock,
            "Lock Screen",
            "Colors for the lock screen input. Omarchy derives them from the palette and \
             accent unless you override them.",
            LOCK_FIELDS,
            &self.lock_pickers,
            cx,
        );
        let btop = self.render_section(
            Override::Btop,
            "Btop",
            "Colors for the btop system monitor. Omarchy maps the palette onto btop's \
             graphs and boxes unless you override it.",
            BTOP_FIELDS,
            &self.btop_pickers,
            cx,
        );

        tab_container()
            .child(help_text(
                "Omarchy generates these app configs from your palette every time the theme \
                 is applied. Turn an override on only when an app needs colors that differ \
                 from the generated ones; turning it off removes the file so it follows the \
                 palette again.",
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
                    .child(focus_section("overrides-browser", &self.scroll, browser))
                    .child(Divider::horizontal())
                    .child(focus_section("overrides-lock", &self.scroll, lock))
                    .child(Divider::horizontal())
                    .child(focus_section("overrides-btop", &self.scroll, btop)),
            )
    }
}
