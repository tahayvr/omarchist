//! The dialog that adds or edits one automation of a flow: what to wait
//! for, its details, and whether to ask before running.
use std::collections::HashSet;

use crate::ui::notify;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, IndexPath, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState, NumberInput},
    select::{SearchableVec, Select, SelectEvent, SelectState},
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::flows::automations::{Automation, Day, Event, bluetooth_in, wifi_in};
use crate::ui::flows_page::app_picker::{AppPicker, AppPickerEvent};
use crate::ui::flows_page::step_builder::{VARIABLES_CONTEXT, step_vars};
use crate::ui::focus::{self, FocusableSwitch};
use crate::ui::keybinds_page::action_builder::LabeledItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationDialogEvent {
    /// The automation, and the place of the one it replaces.
    Save(Option<usize>, Automation),
    Cancel,
}

/// What an automation can wait for, as the dialog lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Time,
    Every,
    AppOpened,
    AppClosed,
    Workspace,
    Fullscreen,
    MonitorConnected,
    MonitorDisconnected,
    BluetoothConnected,
    UsbConnected,
    WifiJoined,
    ChargerConnected,
    ChargerDisconnected,
    BatteryBelow,
    Woke,
    Locked,
    Unlocked,
}

impl EventKind {
    pub const ALL: [EventKind; 17] = [
        EventKind::Time,
        EventKind::Every,
        EventKind::AppOpened,
        EventKind::AppClosed,
        EventKind::Workspace,
        EventKind::Fullscreen,
        EventKind::MonitorConnected,
        EventKind::MonitorDisconnected,
        EventKind::BluetoothConnected,
        EventKind::UsbConnected,
        EventKind::WifiJoined,
        EventKind::ChargerConnected,
        EventKind::ChargerDisconnected,
        EventKind::BatteryBelow,
        EventKind::Woke,
        EventKind::Locked,
        EventKind::Unlocked,
    ];

    pub fn id(self) -> &'static str {
        match self {
            EventKind::Time => "time",
            EventKind::Every => "every",
            EventKind::AppOpened => "app_opened",
            EventKind::AppClosed => "app_closed",
            EventKind::Workspace => "workspace",
            EventKind::Fullscreen => "fullscreen",
            EventKind::MonitorConnected => "monitor_connected",
            EventKind::MonitorDisconnected => "monitor_disconnected",
            EventKind::BluetoothConnected => "bluetooth_connected",
            EventKind::UsbConnected => "usb_connected",
            EventKind::WifiJoined => "wifi_joined",
            EventKind::ChargerConnected => "charger_connected",
            EventKind::ChargerDisconnected => "charger_disconnected",
            EventKind::BatteryBelow => "battery_below",
            EventKind::Woke => "woke",
            EventKind::Locked => "locked",
            EventKind::Unlocked => "unlocked",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EventKind::Time => "At a time of day",
            EventKind::Every => "Every so often",
            EventKind::AppOpened => "An app opens",
            EventKind::AppClosed => "An app closes",
            EventKind::Workspace => "I switch to a workspace",
            EventKind::Fullscreen => "A window goes full screen",
            EventKind::MonitorConnected => "A display is connected",
            EventKind::MonitorDisconnected => "A display is disconnected",
            EventKind::BluetoothConnected => "A Bluetooth device connects",
            EventKind::UsbConnected => "A USB device is plugged in",
            EventKind::WifiJoined => "I join a Wi-Fi network",
            EventKind::ChargerConnected => "The charger is plugged in",
            EventKind::ChargerDisconnected => "The charger is unplugged",
            EventKind::BatteryBelow => "The battery runs low",
            EventKind::Woke => "The computer wakes up",
            EventKind::Locked => "The screen is locked",
            EventKind::Unlocked => "The screen is unlocked",
        }
    }

    pub fn group(self) -> &'static str {
        match self {
            EventKind::Time | EventKind::Every => "Time",
            EventKind::AppOpened
            | EventKind::AppClosed
            | EventKind::Workspace
            | EventKind::Fullscreen => "Apps and windows",
            EventKind::MonitorConnected
            | EventKind::MonitorDisconnected
            | EventKind::BluetoothConnected
            | EventKind::UsbConnected
            | EventKind::WifiJoined => "Devices and network",
            EventKind::ChargerConnected
            | EventKind::ChargerDisconnected
            | EventKind::BatteryBelow => "Power",
            EventKind::Woke | EventKind::Locked | EventKind::Unlocked => "Session",
        }
    }

    /// The Lucide icon the automation wears in lists.
    pub fn icon(self) -> &'static str {
        match self {
            EventKind::Time | EventKind::Every => "icons/clock.svg",
            EventKind::AppOpened | EventKind::AppClosed | EventKind::Fullscreen => {
                "icons/app-window.svg"
            }
            EventKind::Workspace => "icons/layout-grid.svg",
            EventKind::MonitorConnected | EventKind::MonitorDisconnected => "icons/monitor.svg",
            EventKind::BluetoothConnected => "icons/bluetooth.svg",
            EventKind::UsbConnected => "icons/usb.svg",
            EventKind::WifiJoined => "icons/wifi.svg",
            EventKind::ChargerConnected | EventKind::ChargerDisconnected => "icons/plug.svg",
            EventKind::BatteryBelow => "icons/battery-low.svg",
            EventKind::Woke => "icons/sun.svg",
            EventKind::Locked | EventKind::Unlocked => "icons/lock.svg",
        }
    }

    pub fn of(event: &Event) -> Self {
        match event {
            Event::Time { .. } => EventKind::Time,
            Event::Every { .. } => EventKind::Every,
            Event::AppOpened { .. } => EventKind::AppOpened,
            Event::AppClosed { .. } => EventKind::AppClosed,
            Event::Workspace { .. } => EventKind::Workspace,
            Event::Fullscreen => EventKind::Fullscreen,
            Event::MonitorConnected => EventKind::MonitorConnected,
            Event::MonitorDisconnected => EventKind::MonitorDisconnected,
            Event::BluetoothConnected { .. } => EventKind::BluetoothConnected,
            Event::UsbConnected { .. } => EventKind::UsbConnected,
            Event::WifiJoined { .. } => EventKind::WifiJoined,
            Event::ChargerConnected => EventKind::ChargerConnected,
            Event::ChargerDisconnected => EventKind::ChargerDisconnected,
            Event::BatteryBelow { .. } => EventKind::BatteryBelow,
            Event::Woke => EventKind::Woke,
            Event::Locked => EventKind::Locked,
            Event::Unlocked => EventKind::Unlocked,
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.id() == id)
    }

    /// Whether the event is about one named thing (a network, a device).
    fn named(self) -> Option<(&'static str, &'static str)> {
        match self {
            EventKind::WifiJoined => Some(("Network", "Any network")),
            EventKind::BluetoothConnected => Some(("Device", "Any device")),
            EventKind::UsbConnected => Some(("Device", "Any device")),
            _ => None,
        }
    }
}

/// What is on the machine now that a named event could be about, so the
/// name can be picked rather than typed.
fn known(kind: EventKind) -> Vec<String> {
    let output = |program: &str, args: &[&str]| {
        std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .unwrap_or_default()
    };
    let mut names: Vec<String> = match kind {
        EventKind::WifiJoined => wifi_in(&output("iw", &["dev"])).into_iter().collect(),
        EventKind::BluetoothConnected => bluetooth_in(&output("bluetoothctl", &["devices"]))
            .into_iter()
            .collect(),
        EventKind::UsbConnected => std::fs::read_dir("/sys/bus/usb/devices")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| std::fs::read_to_string(entry.path().join("product")).ok())
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty() && !name.contains("Host Controller"))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect(),
        _ => Vec::new(),
    };
    names.sort();
    names.truncate(8);
    names
}

pub struct AutomationDialog {
    /// The place of the automation being edited; `None` adds one.
    index: Option<usize>,
    kind: EventKind,
    kind_select: Entity<SelectState<SearchableVec<LabeledItem>>>,
    time: Entity<InputState>,
    days: Vec<Day>,
    days_focus: FocusHandle,
    /// The day the keyboard is on in the row.
    day_ix: usize,
    minutes: Entity<InputState>,
    app: Entity<AppPicker>,
    workspace: Entity<InputState>,
    percent: Entity<InputState>,
    name: Entity<InputState>,
    /// Names to offer for a named event, found in the background.
    known: Vec<String>,
    ask: bool,
    enabled: bool,
    body_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<AutomationDialogEvent> for AutomationDialog {}

impl AutomationDialog {
    pub fn new(
        index: Option<usize>,
        initial: Option<&Automation>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let kind = initial.map_or(EventKind::Time, |a| EventKind::of(&a.event));
        let (mut time, mut days, mut minutes) = ("09:00".to_string(), Vec::new(), "30".to_string());
        let (mut class, mut workspace, mut percent, mut name) = (
            String::new(),
            String::new(),
            "20".to_string(),
            String::new(),
        );
        match initial.map(|a| &a.event) {
            Some(Event::Time { at, days: d }) => (time, days) = (at.clone(), d.clone()),
            Some(Event::Every { minutes: m }) => minutes = m.to_string(),
            Some(Event::AppOpened { class: c } | Event::AppClosed { class: c }) => {
                class = c.clone()
            }
            Some(Event::Workspace { name: n }) => workspace = n.clone(),
            Some(Event::BatteryBelow { percent: p }) => percent = p.to_string(),
            Some(Event::WifiJoined { name: n }) => name = n.clone(),
            Some(Event::BluetoothConnected { device } | Event::UsbConnected { device }) => {
                name = device.clone()
            }
            _ => {}
        }

        let items: Vec<LabeledItem> = EventKind::ALL
            .iter()
            .map(|kind| LabeledItem {
                id: kind.id().to_string(),
                label: kind.label().into(),
                group: kind.group().into(),
            })
            .collect();
        let selected = EventKind::ALL.iter().position(|k| *k == kind);
        let kind_select = cx.new(|cx| {
            SelectState::new(
                SearchableVec::new(items),
                selected.map(|ix| IndexPath::default().row(ix)),
                window,
                cx,
            )
            .searchable(true)
        });
        let line = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str, value: &str| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder.to_string())
                    .default_value(value.to_string())
            })
        };
        let time = line(window, cx, "09:00", &time);
        let workspace = line(window, cx, "Its name or number", &workspace);
        let name = line(window, cx, "", &name);
        let minutes = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(minutes)
                .step(5.)
                .min(1.)
                .max(1440.)
        });
        let percent = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(percent)
                .step(5.)
                .min(1.)
                .max(99.)
        });
        let app = cx.new(|cx| AppPicker::new(&class, window, cx));

        let mut subscriptions = vec![
            cx.subscribe_in(
                &kind_select,
                window,
                |this, _, event: &SelectEvent<SearchableVec<LabeledItem>>, window, cx| {
                    if let SelectEvent::Confirm(Some(id)) = event
                        && let Some(kind) = EventKind::from_id(id)
                    {
                        this.set_kind(kind, window, cx);
                    }
                },
            ),
            cx.subscribe_in(&app, window, |_, _, _: &AppPickerEvent, _, cx| {
                cx.notify();
            }),
        ];
        for input in [&time, &workspace, &name, &minutes, &percent] {
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                |_, _, event: &InputEvent, _, cx| {
                    if matches!(event, InputEvent::Change) {
                        cx.notify();
                    }
                },
            ));
        }

        let mut this = Self {
            index,
            kind,
            kind_select,
            time,
            days,
            days_focus: focus::tab_stop(cx),
            day_ix: 0,
            minutes,
            app,
            workspace,
            percent,
            name,
            known: Vec::new(),
            ask: initial.is_some_and(|a| a.ask),
            enabled: initial.is_none_or(|a| a.enabled),
            body_focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        this.name_placeholder(window, cx);
        this.load_known(window, cx);
        this
    }

    fn set_kind(&mut self, kind: EventKind, window: &mut Window, cx: &mut Context<Self>) {
        if self.kind == kind {
            return;
        }
        self.kind = kind;
        self.known.clear();
        self.name
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.name_placeholder(window, cx);
        self.load_known(window, cx);
        cx.notify();
    }

    /// An empty name means any network or device; the field says so.
    fn name_placeholder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((_, placeholder)) = self.kind.named() {
            self.name.update(cx, |input, cx| {
                input.set_placeholder(placeholder, window, cx)
            });
        }
    }

    /// Looks up the networks or devices to offer for the current kind.
    fn load_known(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let kind = self.kind;
        if kind.named().is_none() {
            return;
        }
        cx.spawn_in(window, async move |this, cx| {
            let names = cx.background_spawn(async move { known(kind) }).await;
            this.update(cx, |this, cx| {
                if this.kind == kind {
                    this.known = names;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    fn toggle_day(&mut self, day: Day, cx: &mut Context<Self>) {
        match self.days.iter().position(|d| *d == day) {
            Some(ix) => {
                self.days.remove(ix);
            }
            None => self.days.push(day),
        }
        // Kept in week order, which is how the file lists them.
        self.days
            .sort_by_key(|day| Day::ALL.iter().position(|d| d == day));
        cx.notify();
    }

    /// The automation the dialog describes, or what is missing.
    pub fn automation(&self, cx: &App) -> Result<Automation, String> {
        let read = |input: &Entity<InputState>| input.read(cx).value().trim().to_string();
        let number = |input: &Entity<InputState>| read(input).parse::<u32>().ok();
        let event = match self.kind {
            EventKind::Time => Event::Time {
                at: read(&self.time),
                days: self.days.clone(),
            },
            EventKind::Every => Event::Every {
                minutes: number(&self.minutes).unwrap_or(0),
            },
            EventKind::AppOpened => Event::AppOpened {
                class: self.app.read(cx).class().to_string(),
            },
            EventKind::AppClosed => Event::AppClosed {
                class: self.app.read(cx).class().to_string(),
            },
            EventKind::Workspace => Event::Workspace {
                name: read(&self.workspace),
            },
            EventKind::Fullscreen => Event::Fullscreen,
            EventKind::MonitorConnected => Event::MonitorConnected,
            EventKind::MonitorDisconnected => Event::MonitorDisconnected,
            EventKind::BluetoothConnected => Event::BluetoothConnected {
                device: read(&self.name),
            },
            EventKind::UsbConnected => Event::UsbConnected {
                device: read(&self.name),
            },
            EventKind::WifiJoined => Event::WifiJoined {
                name: read(&self.name),
            },
            EventKind::ChargerConnected => Event::ChargerConnected,
            EventKind::ChargerDisconnected => Event::ChargerDisconnected,
            EventKind::BatteryBelow => Event::BatteryBelow {
                percent: number(&self.percent)
                    .and_then(|p| u8::try_from(p).ok())
                    .unwrap_or(0),
            },
            EventKind::Woke => Event::Woke,
            EventKind::Locked => Event::Locked,
            EventKind::Unlocked => Event::Unlocked,
        };
        event.validate().map_err(|message| {
            let mut chars = message.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => message,
            }
        })?;
        Ok(Automation {
            event,
            ask: self.ask,
            enabled: self.enabled,
        })
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.automation(cx) {
            Ok(automation) => {
                cx.emit(AutomationDialogEvent::Save(self.index, automation));
                window.close_dialog(cx);
            }
            Err(error) => notify::error(window, error, cx),
        }
    }

    fn label(text: &'static str) -> Div {
        div().text_sm().child(text)
    }

    fn field(label: &'static str, control: impl IntoElement) -> Div {
        v_flex().gap_1().child(Self::label(label)).child(control)
    }

    /// The days of the week as a row of toggles: one tab stop, the arrows
    /// move along it, Enter or Space toggles.
    fn render_days(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let focused = self.days_focus.is_focused(window);
        let ring = focus::focus_border(focused, theme.transparent, cx);
        h_flex().child(
            h_flex()
                .id("automation-days")
                .test_support()
                .key_context(VARIABLES_CONTEXT)
                .track_focus(&self.days_focus)
                .on_action(cx.listener(|this, _: &step_vars::Prev, _, cx| {
                    this.day_ix = (this.day_ix + 6) % 7;
                    cx.notify();
                }))
                .on_action(cx.listener(|this, _: &step_vars::Next, _, cx| {
                    this.day_ix = (this.day_ix + 1) % 7;
                    cx.notify();
                }))
                .on_action(cx.listener(|this, _: &step_vars::Insert, _, cx| {
                    this.toggle_day(Day::ALL[this.day_ix], cx);
                }))
                .rounded(theme.radius)
                .border_1()
                .border_color(ring)
                .p_0p5()
                .gap_1()
                .flex_wrap()
                .children(Day::ALL.into_iter().enumerate().map(|(ix, day)| {
                    let on = self.days.contains(&day);
                    let current = focused && ix == self.day_ix;
                    let button = Button::new(("automation-day", ix))
                        .label(day.label())
                        .small()
                        .tab_stop(false)
                        .cursor_pointer();
                    let button = if on {
                        button.primary()
                    } else if current {
                        button.outline()
                    } else {
                        button.ghost()
                    };
                    button.on_click(cx.listener(move |this, _, _, cx| {
                        this.day_ix = ix;
                        this.toggle_day(day, cx);
                    }))
                })),
        )
    }

    fn render_fields(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let theme = cx.theme();
        let unit = |text: &'static str| {
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(text)
        };
        Some(match self.kind {
            EventKind::Time => v_flex()
                .gap_3()
                .child(Self::field(
                    "Time",
                    div()
                        .w_24()
                        .child(Input::new(&self.time).id("automation-time").small()),
                ))
                .child(Self::field("Days", self.render_days(window, cx)))
                .into_any_element(),
            EventKind::Every => Self::field(
                "Every",
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .id("automation-minutes")
                            .test_support()
                            .w_32()
                            .child(NumberInput::new(&self.minutes).small()),
                    )
                    .child(unit("minutes")),
            )
            .into_any_element(),
            EventKind::AppOpened | EventKind::AppClosed => {
                Self::field("App", self.app.clone()).into_any_element()
            }
            EventKind::Workspace => Self::field(
                "Workspace",
                Input::new(&self.workspace)
                    .id("automation-workspace")
                    .small(),
            )
            .into_any_element(),
            EventKind::BatteryBelow => Self::field(
                "Below",
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .id("automation-percent")
                            .test_support()
                            .w_32()
                            .child(NumberInput::new(&self.percent).small()),
                    )
                    .child(unit("%")),
            )
            .into_any_element(),
            kind => {
                let (label, _) = kind.named()?;
                v_flex()
                    .gap_2()
                    .child(Self::field(
                        label,
                        Input::new(&self.name).id("automation-name").small(),
                    ))
                    .when(!self.known.is_empty(), |this| {
                        this.child(h_flex().gap_1().flex_wrap().children(
                            self.known.iter().enumerate().map(|(ix, name)| {
                                let value = name.clone();
                                Button::new(("automation-known", ix))
                                    .label(name.clone())
                                    .xsmall()
                                    .outline()
                                    .tab_stop(false)
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        let value = value.clone();
                                        this.name.update(cx, |input, cx| {
                                            input.set_value(value, window, cx)
                                        });
                                        cx.notify();
                                    }))
                            }),
                        ))
                    })
                    .into_any_element()
            }
        })
    }
}

impl Render for AutomationDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let fields = self.render_fields(window, cx);
        let view = cx.entity();
        focus::dialog_body("automation-dialog", &self.body_focus, move |window, cx| {
            view.update(cx, |this, cx| this.save(window, cx));
        })
        .child(
            v_flex()
                .gap_4()
                .child(Self::field(
                    "When",
                    Select::new(&self.kind_select)
                        .id("automation-kind")
                        .placeholder("Choose what to wait for")
                        .search_placeholder("Search")
                        .menu_max_h(px(360.))
                        .small(),
                ))
                .children(fields)
                .child(
                    div().text_sm().child(
                        FocusableSwitch::new("automation-ask")
                            .label("Ask before running")
                            .checked(self.ask)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.ask = *checked;
                                cx.notify();
                            })),
                    ),
                )
                .child(
                    h_flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("automation-cancel")
                                .outline()
                                .small()
                                .label("Cancel")
                                .cursor_pointer()
                                .on_click(cx.listener(|_, _, window, cx| {
                                    cx.emit(AutomationDialogEvent::Cancel);
                                    window.close_dialog(cx);
                                })),
                        )
                        .child(
                            Button::new("automation-save")
                                .primary()
                                .small()
                                .label(if self.index.is_some() {
                                    "Save automation"
                                } else {
                                    "Add automation"
                                })
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                        ),
                ),
        )
    }
}

/// Opens the dialog and returns its entity so the caller can subscribe.
pub fn open_automation_dialog(
    index: Option<usize>,
    initial: Option<&Automation>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<AutomationDialog> {
    let title = if index.is_some() {
        "Edit automation"
    } else {
        "Add automation"
    };
    let dialog = cx.new(|cx| AutomationDialog::new(index, initial, window, cx));
    let view = dialog.clone();
    let body_focus = dialog.read(cx).body_focus.clone();
    window.open_dialog(cx, move |d, window, _| {
        let on_close_view = view.clone();
        d.title(title)
            .w(focus::dialog_width(520., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(false)
            .on_close(move |_, _, cx| {
                on_close_view.update(cx, |_, cx| cx.emit(AutomationDialogEvent::Cancel));
            })
            .child(view.clone())
    });
    focus::focus_first_in(&body_focus, window, cx);
    dialog
}

#[cfg(test)]
mod tests {
    use super::EventKind;
    use crate::system::flows::automations::Event;

    #[test]
    fn every_kind_has_an_embedded_icon_and_round_trips() {
        for kind in EventKind::ALL {
            let embedded =
                gpui::AssetSource::load(&crate::assets::CombinedAssets::new(), kind.icon());
            assert!(
                matches!(embedded, Ok(Some(_))),
                "{} is not embedded",
                kind.icon()
            );
            assert_eq!(EventKind::from_id(kind.id()), Some(kind));
        }
        assert_eq!(EventKind::of(&Event::Woke), EventKind::Woke);
        assert_eq!(
            EventKind::of(&Event::BatteryBelow { percent: 20 }),
            EventKind::BatteryBelow
        );
    }
}
