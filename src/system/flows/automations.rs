//! Automations: a flow started by something that happens, without anyone
//! asking for it. A flow's `[[triggers.automation]]` entries say what to
//! wait for; `omarchist automations run` (a systemd user service, see
//! `service.rs`) watches for it and starts the flow.
//!
//! What happens on the machine reaches this module as [`Signal`]s, from
//! Hyprland's event socket and from a few cheap polls. An [`Event`] says
//! which signals it answers to. Everything that decides is a pure function
//! of signals and state, so it is tested without a desktop.
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::Flow;
use super::condition::{Power, parse_time, power};
use super::store::{flows_dir, load_flows};

/// A day of the week, as a flow file writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Day {
    pub const ALL: [Day; 7] = [
        Day::Mon,
        Day::Tue,
        Day::Wed,
        Day::Thu,
        Day::Fri,
        Day::Sat,
        Day::Sun,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Day::Mon => "Mon",
            Day::Tue => "Tue",
            Day::Wed => "Wed",
            Day::Thu => "Thu",
            Day::Fri => "Fri",
            Day::Sat => "Sat",
            Day::Sun => "Sun",
        }
    }

    fn from_chrono(day: chrono::Weekday) -> Self {
        Day::ALL[day.num_days_from_monday() as usize]
    }
}

/// What an automation waits for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "on", rename_all = "snake_case")]
pub enum Event {
    /// A time of day (`HH:MM`), on `days`, or every day when there are none.
    Time {
        at: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        days: Vec<Day>,
    },
    /// Every so many minutes, counted from when the service started.
    Every {
        minutes: u32,
    },
    /// The first window of an app opens.
    AppOpened {
        class: String,
    },
    /// The last window of an app closes.
    AppClosed {
        class: String,
    },
    /// The workspace of this name comes to the front.
    Workspace {
        name: String,
    },
    MonitorConnected,
    MonitorDisconnected,
    /// A window goes full screen.
    Fullscreen,
    ChargerConnected,
    ChargerDisconnected,
    /// The battery falls below this charge while running on it.
    BatteryBelow {
        percent: u8,
    },
    /// A Wi-Fi network is joined: this one, or any when the name is empty.
    WifiJoined {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        name: String,
    },
    /// A Bluetooth device connects: this one, or any.
    BluetoothConnected {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        device: String,
    },
    /// A USB device is plugged in: this one, or any.
    UsbConnected {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        device: String,
    },
    /// The computer wakes from sleep.
    Woke,
    Locked,
    Unlocked,
}

/// One of a flow's automations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Automation {
    #[serde(flatten)]
    pub event: Event,
    /// Show a notification to click instead of running at once.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ask: bool,
    #[serde(default = "enabled", skip_serializing_if = "is_true")]
    pub enabled: bool,
}

fn enabled() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

impl Automation {
    pub fn new(event: Event) -> Self {
        Self {
            event,
            ask: false,
            enabled: true,
        }
    }
}

/// Something that happened on the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    /// A new minute of the wall clock began.
    Minute {
        of_day: u32,
        day: Day,
    },
    AppOpened(String),
    AppClosed(String),
    Workspace(String),
    MonitorConnected(String),
    MonitorDisconnected(String),
    Fullscreen,
    ChargerConnected,
    ChargerDisconnected,
    /// The battery's charge went from one level to another, on battery.
    BatteryFell {
        from: u8,
        to: u8,
    },
    WifiJoined(String),
    BluetoothConnected(String),
    UsbConnected(String),
    Woke,
    Locked,
    Unlocked,
}

/// A name the person typed against one the machine reports: the same
/// ignoring case and outer spaces. An empty wanted name matches any.
fn names(wanted: &str, actual: &str) -> bool {
    wanted.trim().is_empty() || wanted.trim().eq_ignore_ascii_case(actual.trim())
}

impl Event {
    /// Whether the event answers to `signal`. When it does, the text is
    /// what the flow gets as its input: the app, the network, the device.
    /// `Every` answers to no signal; the service counts its minutes.
    pub fn answers(&self, signal: &Signal) -> Option<String> {
        match (self, signal) {
            (Event::Time { at, days }, Signal::Minute { of_day, day }) => {
                let due = parse_time(at) == Some(*of_day);
                (due && (days.is_empty() || days.contains(day))).then(String::new)
            }
            (Event::AppOpened { class }, Signal::AppOpened(opened)) => {
                (!class.trim().is_empty() && names(class, opened)).then(|| opened.clone())
            }
            (Event::AppClosed { class }, Signal::AppClosed(closed)) => {
                (!class.trim().is_empty() && names(class, closed)).then(|| closed.clone())
            }
            (Event::Workspace { name }, Signal::Workspace(now)) => {
                (!name.trim().is_empty() && names(name, now)).then(|| now.clone())
            }
            (Event::MonitorConnected, Signal::MonitorConnected(name))
            | (Event::MonitorDisconnected, Signal::MonitorDisconnected(name)) => Some(name.clone()),
            (Event::Fullscreen, Signal::Fullscreen)
            | (Event::ChargerConnected, Signal::ChargerConnected)
            | (Event::ChargerDisconnected, Signal::ChargerDisconnected)
            | (Event::Woke, Signal::Woke)
            | (Event::Locked, Signal::Locked)
            | (Event::Unlocked, Signal::Unlocked) => Some(String::new()),
            (Event::BatteryBelow { percent }, Signal::BatteryFell { from, to }) => {
                (to < percent && percent <= from).then(|| to.to_string())
            }
            (Event::WifiJoined { name }, Signal::WifiJoined(joined)) => {
                names(name, joined).then(|| joined.clone())
            }
            (Event::BluetoothConnected { device }, Signal::BluetoothConnected(connected))
            | (Event::UsbConnected { device }, Signal::UsbConnected(connected)) => {
                names(device, connected).then(|| connected.clone())
            }
            _ => None,
        }
    }

    /// What is missing before the event can be waited for.
    pub fn validate(&self) -> std::result::Result<(), String> {
        match self {
            Event::Time { at, .. } if parse_time(at).is_none() => {
                Err("the time is written as 09:00".to_string())
            }
            Event::Every { minutes } if *minutes == 0 || *minutes > 24 * 60 => {
                Err("repeat every 1 to 1440 minutes".to_string())
            }
            Event::AppOpened { class } | Event::AppClosed { class } if class.trim().is_empty() => {
                Err("pick the app".to_string())
            }
            Event::Workspace { name } if name.trim().is_empty() => {
                Err("name the workspace".to_string())
            }
            Event::BatteryBelow { percent } if *percent == 0 || *percent > 99 => {
                Err("the battery level goes from 1 to 99".to_string())
            }
            _ => Ok(()),
        }
    }

    /// The event as the start of a sentence: "At 09:00 on weekdays".
    pub fn describe(&self) -> String {
        let named = |name: &str, any: &str| {
            if name.trim().is_empty() {
                any.to_string()
            } else {
                name.trim().to_string()
            }
        };
        match self {
            Event::Time { at, days } => format!("At {}{}", at.trim(), days_phrase(days)),
            Event::Every { minutes: 1 } => "Every minute".to_string(),
            Event::Every { minutes } if minutes % 60 == 0 => {
                let hours = minutes / 60;
                if hours == 1 {
                    "Every hour".to_string()
                } else {
                    format!("Every {hours} hours")
                }
            }
            Event::Every { minutes } => format!("Every {minutes} minutes"),
            Event::AppOpened { class } => format!("When {} opens", class.trim()),
            Event::AppClosed { class } => format!("When {} closes", class.trim()),
            Event::Workspace { name } => format!("When I switch to workspace {}", name.trim()),
            Event::MonitorConnected => "When a display is connected".to_string(),
            Event::MonitorDisconnected => "When a display is disconnected".to_string(),
            Event::Fullscreen => "When a window goes full screen".to_string(),
            Event::ChargerConnected => "When the charger is plugged in".to_string(),
            Event::ChargerDisconnected => "When the charger is unplugged".to_string(),
            Event::BatteryBelow { percent } => format!("When the battery falls below {percent}%"),
            Event::WifiJoined { name } => {
                format!("When I join {}", named(name, "a Wi-Fi network"))
            }
            Event::BluetoothConnected { device } => {
                format!("When {} connects", named(device, "a Bluetooth device"))
            }
            Event::UsbConnected { device } => {
                format!("When {} is plugged in", named(device, "a USB device"))
            }
            Event::Woke => "When the computer wakes up".to_string(),
            Event::Locked => "When the screen is locked".to_string(),
            Event::Unlocked => "When the screen is unlocked".to_string(),
        }
    }
}

/// " on weekdays", " on Mon, Wed", or nothing for every day.
fn days_phrase(days: &[Day]) -> String {
    let set: HashSet<Day> = days.iter().copied().collect();
    let weekdays: HashSet<Day> = Day::ALL[..5].iter().copied().collect();
    let weekend: HashSet<Day> = Day::ALL[5..].iter().copied().collect();
    if set.is_empty() || set.len() == 7 {
        String::new()
    } else if set == weekdays {
        " on weekdays".to_string()
    } else if set == weekend {
        " at the weekend".to_string()
    } else {
        let listed: Vec<&str> = Day::ALL
            .iter()
            .filter(|day| set.contains(day))
            .map(|day| day.label())
            .collect();
        format!(" on {}", listed.join(", "))
    }
}

// MARK: Watching

/// The windows that are open, by address, so a closing window (which
/// Hyprland names only by address) can be told to be an app's last.
#[derive(Debug, Default)]
pub struct Windows {
    classes: HashMap<String, String>,
}

impl Windows {
    /// Starts from the windows already open (`hyprctl -j clients`).
    pub fn seed(&mut self, clients: &serde_json::Value) {
        for client in clients.as_array().into_iter().flatten() {
            let address = client.get("address").and_then(|a| a.as_str());
            let class = client.get("class").and_then(|c| c.as_str());
            if let (Some(address), Some(class)) = (address, class) {
                self.classes.insert(
                    address.trim_start_matches("0x").to_string(),
                    class.to_string(),
                );
            }
        }
    }

    fn count(&self, class: &str) -> usize {
        self.classes
            .values()
            .filter(|c| c.as_str() == class)
            .count()
    }

    /// Takes one line of Hyprland's event socket and says what happened.
    pub fn apply(&mut self, line: &str) -> Vec<Signal> {
        let Some((event, data)) = line.trim_end().split_once(">>") else {
            return Vec::new();
        };
        match event {
            "openwindow" => {
                // ADDRESS,WORKSPACE,CLASS,TITLE; a title may hold commas.
                let mut parts = data.splitn(4, ',');
                let (Some(address), _, Some(class)) = (parts.next(), parts.next(), parts.next())
                else {
                    return Vec::new();
                };
                let first = self.count(class) == 0;
                self.classes.insert(address.to_string(), class.to_string());
                if first && !class.is_empty() {
                    vec![Signal::AppOpened(class.to_string())]
                } else {
                    Vec::new()
                }
            }
            "closewindow" => match self.classes.remove(data) {
                Some(class) if !class.is_empty() && self.count(&class) == 0 => {
                    vec![Signal::AppClosed(class)]
                }
                _ => Vec::new(),
            },
            "workspace" => vec![Signal::Workspace(data.to_string())],
            "monitoradded" => vec![Signal::MonitorConnected(data.to_string())],
            "monitorremoved" => vec![Signal::MonitorDisconnected(data.to_string())],
            "fullscreen" if data == "1" => vec![Signal::Fullscreen],
            _ => Vec::new(),
        }
    }
}

/// Turns successive readings of the machine's power into signals. The
/// first reading only sets the baseline.
#[derive(Debug, Default)]
pub struct PowerWatch {
    last: Option<Power>,
}

impl PowerWatch {
    pub fn update(&mut self, now: Power) -> Vec<Signal> {
        let mut signals = Vec::new();
        if let Some(before) = self.last
            && before.battery
            && now.battery
        {
            if now.plugged_in && !before.plugged_in {
                signals.push(Signal::ChargerConnected);
            }
            if !now.plugged_in && before.plugged_in {
                signals.push(Signal::ChargerDisconnected);
            }
            if let (Some(from), Some(to)) = (before.percent, now.percent)
                && to < from
                && !now.plugged_in
            {
                signals.push(Signal::BatteryFell { from, to });
            }
        }
        self.last = Some(now);
        signals
    }
}

/// Reports what is new in a set that is read again and again (connected
/// devices). The first reading only sets the baseline.
#[derive(Debug, Default)]
pub struct NewOnes {
    seen: Option<HashSet<String>>,
}

impl NewOnes {
    pub fn update(&mut self, now: HashSet<String>) -> Vec<String> {
        let mut new: Vec<String> = match &self.seen {
            Some(seen) => now.difference(seen).cloned().collect(),
            None => Vec::new(),
        };
        new.sort();
        self.seen = Some(now);
        new
    }
}

/// What the service polls for, and only when a flow waits for it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Needs {
    pub hyprland: bool,
    pub power: bool,
    pub wifi: bool,
    pub bluetooth: bool,
    pub usb: bool,
    pub sleep: bool,
    pub lock: bool,
}

impl Needs {
    pub fn of(flows: &[Flow]) -> Self {
        let mut needs = Needs::default();
        for automation in flows
            .iter()
            .flat_map(|flow| &flow.triggers.automations)
            .filter(|automation| automation.enabled)
        {
            match automation.event {
                Event::AppOpened { .. }
                | Event::AppClosed { .. }
                | Event::Workspace { .. }
                | Event::MonitorConnected
                | Event::MonitorDisconnected
                | Event::Fullscreen => needs.hyprland = true,
                Event::ChargerConnected
                | Event::ChargerDisconnected
                | Event::BatteryBelow { .. } => needs.power = true,
                Event::WifiJoined { .. } => needs.wifi = true,
                Event::BluetoothConnected { .. } => needs.bluetooth = true,
                Event::UsbConnected { .. } => needs.usb = true,
                Event::Woke => needs.sleep = true,
                Event::Locked | Event::Unlocked => needs.lock = true,
                Event::Time { .. } | Event::Every { .. } => {}
            }
        }
        needs
    }
}

fn output_of(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
}

/// The Wi-Fi network the machine is on, from `iw dev`'s `ssid` line.
pub fn wifi_in(iw_dev: &str) -> Option<String> {
    iw_dev
        .lines()
        .find_map(|line| line.trim().strip_prefix("ssid "))
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

/// The names of the connected Bluetooth devices, from
/// `bluetoothctl devices Connected` (`Device <address> <name>`).
pub fn bluetooth_in(listing: &str) -> HashSet<String> {
    listing
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Device "))
        .filter_map(|rest| rest.split_once(' '))
        .map(|(_, name)| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

/// The product names of the USB devices plugged in, from sysfs.
fn usb_devices() -> HashSet<String> {
    std::fs::read_dir("/sys/bus/usb/devices")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| std::fs::read_to_string(entry.path().join("product")).ok())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

/// How often the machine has gone to sleep and come back.
fn sleep_count() -> Option<u64> {
    std::fs::read_to_string("/sys/power/suspend_stats/success")
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Whether the session is locked, read the way Omarchy does: an active
/// lock is one of the reasons a monitor reports for not going solitary.
pub fn locked_in(monitors: &serde_json::Value) -> Option<bool> {
    let monitors = monitors.as_array()?;
    let blockers = |monitor: &serde_json::Value| -> Vec<String> {
        monitor
            .get("solitaryBlockedBy")
            .and_then(|b| b.as_array())
            .into_iter()
            .flatten()
            .filter_map(|b| b.as_str().map(str::to_string))
            .collect()
    };
    if monitors
        .iter()
        .any(|m| blockers(m).iter().any(|b| b == "LOCK"))
    {
        return Some(true);
    }
    // A monitor with no workspace yet stops at that reason and never gets
    // to the lock, so it says nothing either way.
    monitors
        .iter()
        .any(|m| !blockers(m).iter().any(|b| b == "WORKSPACE"))
        .then_some(false)
}

fn hyprland_socket() -> Option<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")?;
    let instance = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;
    Some(
        PathBuf::from(runtime)
            .join("hypr")
            .join(instance)
            .join(".socket2.sock"),
    )
}

/// Reads Hyprland's event socket for as long as the service runs,
/// reconnecting when the compositor restarts.
fn watch_hyprland(signals: mpsc::Sender<Signal>) {
    loop {
        if let Some(stream) = hyprland_socket().and_then(|path| UnixStream::connect(path).ok()) {
            let mut windows = Windows::default();
            let clients = output_of("hyprctl", &["-j", "clients"]);
            if let Ok(clients) = serde_json::from_str(&clients) {
                windows.seed(&clients);
            }
            for line in BufReader::new(stream).lines() {
                let Ok(line) = line else { break };
                for signal in windows.apply(&line) {
                    if signals.send(signal).is_err() {
                        return;
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}

// MARK: The service

/// The least time between two runs of one automation, so a burst of
/// events (windows opening at login) starts a flow once.
const MIN_GAP: Duration = Duration::from_secs(5);

/// When each automation last ran, and what is still running.
#[derive(Default)]
pub struct Throttle {
    last: HashMap<(String, usize), Instant>,
    running: HashMap<String, Child>,
}

impl Throttle {
    /// Whether automation `index` of the flow `id` may start a run now:
    /// its last one was long enough ago and the flow is not still going.
    pub fn allows(&mut self, id: &str, index: usize, now: Instant) -> bool {
        if let Some(child) = self.running.get_mut(id) {
            match child.try_wait() {
                Ok(None) => return false,
                _ => {
                    self.running.remove(id);
                }
            }
        }
        !self
            .last
            .get(&(id.to_string(), index))
            .is_some_and(|last| now.duration_since(*last) < MIN_GAP)
    }

    pub fn started(&mut self, id: &str, index: usize, now: Instant, child: Option<Child>) {
        self.last.insert((id.to_string(), index), now);
        if let Some(child) = child {
            self.running.insert(id.to_string(), child);
        }
    }

    /// When `Every` automation `index` of `id` is next due; an automation
    /// first seen at `now` is due one interval later.
    pub fn every_due(&mut self, id: &str, index: usize, minutes: u32, now: Instant) -> bool {
        let key = (id.to_string(), index);
        match self.last.get(&key) {
            Some(last) => now.duration_since(*last) >= Duration::from_secs(minutes as u64 * 60),
            None => {
                self.last.insert(key, now);
                false
            }
        }
    }
}

/// The flows that have automations, read again when their folder changes.
struct Watched {
    flows: Vec<Flow>,
    stamp: Option<SystemTime>,
}

impl Watched {
    /// The newest change time among the flows folder and its files.
    fn stamp() -> Option<SystemTime> {
        let dir = flows_dir().ok()?;
        let mut newest = std::fs::metadata(&dir).ok()?.modified().ok()?;
        for entry in std::fs::read_dir(&dir).ok()?.flatten() {
            if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
                newest = newest.max(modified);
            }
        }
        Some(newest)
    }

    fn reload_if_changed(&mut self) -> bool {
        let stamp = Self::stamp();
        if stamp == self.stamp && stamp.is_some() {
            return false;
        }
        self.stamp = stamp;
        self.flows = load_flows()
            .unwrap_or_default()
            .into_iter()
            .filter(|flow| flow.triggers.automations.iter().any(|a| a.enabled))
            .collect();
        true
    }
}

/// Starts the flow for an automation: at once, or after a click on a
/// notification when it asks first.
fn start(flow: &Flow, automation: &Automation, input: &str) -> Option<Child> {
    // Never `current_exe`: after a package upgrade that is
    // `/usr/bin/omarchist (deleted)` for as long as the service runs.
    let binary = crate::system::binary::omarchist_binary();
    let label = automation.event.describe();
    let mut run: Vec<String> = vec![
        binary,
        "flow".into(),
        "run".into(),
        flow.id.clone(),
        "--trigger".into(),
        label.clone(),
    ];
    if !input.trim().is_empty() {
        run.push("--".into());
        run.push(input.to_string());
    }
    if automation.ask {
        let status = Command::new("omarchy-notification-send")
            .args(["--app-name", "Omarchist", "-u", "normal"])
            .arg(format!("Run {}?", flow.name.trim()))
            .arg(format!("{label}. Click to run."))
            .arg("--exec")
            .args(&run)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if !status.is_ok_and(|s| s.success()) {
            eprintln!("Could not ask before running '{}'", flow.id);
        }
        return None;
    }
    println!("{label}: running '{}'", flow.id);
    Command::new(&run[0])
        .args(&run[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| eprintln!("Could not run '{}': {e}", flow.id))
        .ok()
}

/// Runs every automation `signal` is for.
fn dispatch(signal: &Signal, flows: &[Flow], throttle: &mut Throttle) {
    let now = Instant::now();
    for flow in flows {
        for (index, automation) in flow.triggers.automations.iter().enumerate() {
            if !automation.enabled {
                continue;
            }
            let Some(input) = automation.event.answers(signal) else {
                continue;
            };
            if throttle.allows(&flow.id, index, now) {
                let child = start(flow, automation, &input);
                throttle.started(&flow.id, index, now, child);
            }
        }
    }
}

/// The service: watches for what the flows' automations wait for and
/// starts them, until it is stopped.
pub fn run() -> Result<()> {
    if hyprland_socket().is_none() {
        return Err(Error::Invalid(
            "Automations need a running Hyprland session".to_string(),
        ));
    }
    let (sender, signals) = mpsc::channel::<Signal>();
    {
        let sender = sender.clone();
        std::thread::spawn(move || watch_hyprland(sender));
    }
    let mut watched = Watched {
        flows: Vec::new(),
        stamp: None,
    };
    let mut throttle = Throttle::default();
    let mut needs = Needs::default();
    let mut power_watch = PowerWatch::default();
    let (mut wifi, mut bluetooth, mut usb) =
        (NewOnes::default(), NewOnes::default(), NewOnes::default());
    let mut sleeps = sleep_count();
    let mut locked: Option<bool> = None;
    let mut minute: Option<u32> = None;
    let mut tick: u64 = 0;
    println!("Omarchist automations are running");

    loop {
        // A second at most between polls; Hyprland's events come at once.
        match signals.recv_timeout(Duration::from_secs(1)) {
            Ok(signal) => {
                if needs.hyprland {
                    dispatch(&signal, &watched.flows, &mut throttle);
                }
                continue;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {}
        }
        tick += 1;
        if tick % 3 == 1 && watched.reload_if_changed() {
            needs = Needs::of(&watched.flows);
            println!(
                "Watching {} flow{} with automations",
                watched.flows.len(),
                if watched.flows.len() == 1 { "" } else { "s" }
            );
        }
        let mut happened: Vec<Signal> = Vec::new();

        let now = chrono::Local::now();
        use chrono::{Datelike, Timelike};
        let of_day = now.hour() * 60 + now.minute();
        if minute != Some(of_day) {
            // The first reading sets the clock; a flow set for this very
            // minute still runs, since the service may start within it.
            minute = Some(of_day);
            happened.push(Signal::Minute {
                of_day,
                day: Day::from_chrono(now.weekday()),
            });
        }

        if needs.power {
            happened.extend(power_watch.update(power()));
        }
        if needs.sleep {
            let count = sleep_count();
            if let (Some(before), Some(after)) = (sleeps, count)
                && after > before
            {
                happened.push(Signal::Woke);
            }
            sleeps = count;
        }
        if needs.lock && tick.is_multiple_of(2) {
            let monitors = output_of("hyprctl", &["-j", "monitors"]);
            let now_locked = serde_json::from_str(&monitors)
                .ok()
                .and_then(|monitors| locked_in(&monitors));
            if let (Some(before), Some(after)) = (locked, now_locked)
                && before != after
            {
                happened.push(if after {
                    Signal::Locked
                } else {
                    Signal::Unlocked
                });
            }
            if now_locked.is_some() {
                locked = now_locked;
            }
        }
        if tick.is_multiple_of(5) {
            if needs.wifi {
                let network: HashSet<String> =
                    wifi_in(&output_of("iw", &["dev"])).into_iter().collect();
                happened.extend(wifi.update(network).into_iter().map(Signal::WifiJoined));
            }
            if needs.bluetooth {
                let devices = bluetooth_in(&output_of("bluetoothctl", &["devices", "Connected"]));
                happened.extend(
                    bluetooth
                        .update(devices)
                        .into_iter()
                        .map(Signal::BluetoothConnected),
                );
            }
        }
        if needs.usb && tick.is_multiple_of(2) {
            happened.extend(
                usb.update(usb_devices())
                    .into_iter()
                    .map(Signal::UsbConnected),
            );
        }

        for signal in &happened {
            dispatch(signal, &watched.flows, &mut throttle);
        }

        // Automations on an interval are counted here, not signalled.
        let now = Instant::now();
        for flow in &watched.flows {
            for (index, automation) in flow.triggers.automations.iter().enumerate() {
                if let Event::Every { minutes } = automation.event
                    && automation.enabled
                    && throttle.every_due(&flow.id, index, minutes, now)
                    && throttle.allows(&flow.id, index, now)
                {
                    let child = start(flow, automation, "");
                    throttle.started(&flow.id, index, now, child);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minute(time: &str, day: Day) -> Signal {
        Signal::Minute {
            of_day: parse_time(time).unwrap(),
            day,
        }
    }

    #[test]
    fn a_time_answers_to_its_minute_on_its_days() {
        let weekdays = Event::Time {
            at: "09:00".into(),
            days: Day::ALL[..5].to_vec(),
        };
        assert_eq!(
            weekdays.answers(&minute("09:00", Day::Tue)),
            Some(String::new())
        );
        assert_eq!(weekdays.answers(&minute("09:00", Day::Sat)), None);
        assert_eq!(weekdays.answers(&minute("09:01", Day::Tue)), None);
        let daily = Event::Time {
            at: "23:59".into(),
            days: Vec::new(),
        };
        assert!(daily.answers(&minute("23:59", Day::Sun)).is_some());
        assert_eq!(daily.answers(&Signal::Woke), None);
    }

    #[test]
    fn named_events_match_ignoring_case_and_hand_the_name_on() {
        let opened = Event::AppOpened {
            class: "Firefox".into(),
        };
        assert_eq!(
            opened.answers(&Signal::AppOpened("firefox".into())),
            Some("firefox".into())
        );
        assert_eq!(opened.answers(&Signal::AppOpened("chromium".into())), None);
        assert_eq!(opened.answers(&Signal::AppClosed("firefox".into())), None);

        let any_wifi = Event::WifiJoined {
            name: String::new(),
        };
        assert_eq!(
            any_wifi.answers(&Signal::WifiJoined("Home".into())),
            Some("Home".into())
        );
        let home = Event::WifiJoined {
            name: " home ".into(),
        };
        assert!(home.answers(&Signal::WifiJoined("Home".into())).is_some());
        assert!(home.answers(&Signal::WifiJoined("Office".into())).is_none());

        let headphones = Event::BluetoothConnected {
            device: "WH-1000XM5".into(),
        };
        assert!(
            headphones
                .answers(&Signal::BluetoothConnected("WH-1000XM5".into()))
                .is_some()
        );
        // A USB device of the same name is not a Bluetooth one.
        assert!(
            headphones
                .answers(&Signal::UsbConnected("WH-1000XM5".into()))
                .is_none()
        );
    }

    #[test]
    fn the_battery_level_answers_once_when_crossed() {
        let low = Event::BatteryBelow { percent: 20 };
        assert_eq!(
            low.answers(&Signal::BatteryFell { from: 20, to: 19 }),
            Some("19".into())
        );
        assert_eq!(low.answers(&Signal::BatteryFell { from: 19, to: 18 }), None);
        assert_eq!(low.answers(&Signal::BatteryFell { from: 25, to: 21 }), None);
        assert!(
            low.answers(&Signal::BatteryFell { from: 30, to: 10 })
                .is_some()
        );
    }

    #[test]
    fn events_read_as_sentences() {
        let at = |days: &[Day]| {
            Event::Time {
                at: "09:00".into(),
                days: days.to_vec(),
            }
            .describe()
        };
        assert_eq!(at(&[]), "At 09:00");
        assert_eq!(at(&Day::ALL[..5]), "At 09:00 on weekdays");
        assert_eq!(at(&Day::ALL[5..]), "At 09:00 at the weekend");
        assert_eq!(at(&[Day::Wed, Day::Mon]), "At 09:00 on Mon, Wed");
        assert_eq!(Event::Every { minutes: 60 }.describe(), "Every hour");
        assert_eq!(Event::Every { minutes: 90 }.describe(), "Every 90 minutes");
        assert_eq!(
            Event::WifiJoined {
                name: String::new()
            }
            .describe(),
            "When I join a Wi-Fi network"
        );
        assert_eq!(
            Event::BatteryBelow { percent: 15 }.describe(),
            "When the battery falls below 15%"
        );
    }

    #[test]
    fn events_are_checked() {
        let bad = [
            Event::Time {
                at: "9am".into(),
                days: Vec::new(),
            },
            Event::Every { minutes: 0 },
            Event::AppOpened { class: " ".into() },
            Event::Workspace {
                name: String::new(),
            },
            Event::BatteryBelow { percent: 100 },
        ];
        for event in bad {
            assert!(event.validate().is_err(), "{event:?}");
        }
        assert!(Event::Woke.validate().is_ok());
        assert!(
            Event::UsbConnected {
                device: String::new()
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn automations_are_tables_of_their_flow() {
        let mut flow = Flow::new("morning".into(), "Morning".into());
        flow.triggers.automations = vec![
            Automation::new(Event::Time {
                at: "09:00".into(),
                days: vec![Day::Mon, Day::Fri],
            }),
            Automation {
                ask: true,
                enabled: false,
                ..Automation::new(Event::ChargerDisconnected)
            },
        ];
        let text = flow.to_toml().unwrap();
        assert!(text.contains("format = 2\n"), "{text}");
        assert!(
            text.contains("[[triggers.automation]]\non = \"time\"\nat = \"09:00\"\ndays = [\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "[[triggers.automation]]\non = \"charger_disconnected\"\nask = true\nenabled = false\n"
            ),
            "{text}"
        );
        assert_eq!(crate::system::flows::parse_flow(&text).unwrap(), {
            let mut expected = flow.clone();
            expected.format = crate::system::flows::FORMAT;
            expected
        });
    }

    #[test]
    fn windows_tell_an_apps_first_and_last() {
        let mut windows = Windows::default();
        windows.seed(&serde_json::json!([
            { "address": "0xaaa", "class": "kitty" },
        ]));
        // A second terminal is not the app opening.
        assert!(windows.apply("openwindow>>bbb,1,kitty,~").is_empty());
        assert_eq!(
            windows.apply("openwindow>>ccc,2,firefox,Title, with commas"),
            vec![Signal::AppOpened("firefox".into())]
        );
        assert!(windows.apply("closewindow>>aaa").is_empty());
        assert_eq!(
            windows.apply("closewindow>>bbb"),
            vec![Signal::AppClosed("kitty".into())]
        );
        assert!(windows.apply("closewindow>>zzz").is_empty(), "never seen");
        assert_eq!(
            windows.apply("workspace>>3\n"),
            vec![Signal::Workspace("3".into())]
        );
        assert_eq!(
            windows.apply("monitoradded>>DP-1"),
            vec![Signal::MonitorConnected("DP-1".into())]
        );
        assert_eq!(windows.apply("fullscreen>>1"), vec![Signal::Fullscreen]);
        assert!(windows.apply("fullscreen>>0").is_empty());
        assert!(windows.apply("activewindow>>kitty,~").is_empty());
        assert!(windows.apply("garbage").is_empty());
    }

    #[test]
    fn power_readings_become_signals_after_the_first() {
        let reading = |plugged_in: bool, percent: u8| Power {
            battery: true,
            plugged_in,
            percent: Some(percent),
        };
        let mut watch = PowerWatch::default();
        assert!(watch.update(reading(true, 80)).is_empty(), "the baseline");
        assert_eq!(
            watch.update(reading(false, 80)),
            vec![Signal::ChargerDisconnected]
        );
        assert_eq!(
            watch.update(reading(false, 79)),
            vec![Signal::BatteryFell { from: 80, to: 79 }]
        );
        assert!(watch.update(reading(false, 79)).is_empty());
        assert_eq!(
            watch.update(reading(true, 79)),
            vec![Signal::ChargerConnected]
        );
        // Charging, a dip is not the battery running down.
        assert!(watch.update(reading(true, 78)).is_empty());
        // A desktop never signals.
        let mut desktop = PowerWatch::default();
        desktop.update(Power::default());
        assert!(desktop.update(Power::default()).is_empty());
    }

    #[test]
    fn new_devices_are_reported_once() {
        let set =
            |names: &[&str]| -> HashSet<String> { names.iter().map(|n| n.to_string()).collect() };
        let mut devices = NewOnes::default();
        assert!(devices.update(set(&["Mouse"])).is_empty(), "the baseline");
        assert_eq!(
            devices.update(set(&["Mouse", "Headphones"])),
            vec!["Headphones"]
        );
        assert!(devices.update(set(&["Mouse", "Headphones"])).is_empty());
        assert!(devices.update(set(&["Mouse"])).is_empty());
        assert_eq!(
            devices.update(set(&["Mouse", "Headphones"])),
            vec!["Headphones"]
        );
    }

    #[test]
    fn machine_listings_are_parsed() {
        let iw =
            "phy#0\n\tInterface wlan0\n\t\tifindex 3\n\t\tssid Home Network\n\t\ttype managed\n";
        assert_eq!(wifi_in(iw).as_deref(), Some("Home Network"));
        assert_eq!(
            wifi_in("phy#0\n\tInterface wlan0\n\t\ttype managed\n"),
            None
        );
        assert_eq!(
            bluetooth_in(
                "Device AA:BB:CC:DD:EE:FF MX Master 3S\nDevice 11:22:33:44:55:66 WH-1000XM5\n"
            ),
            ["MX Master 3S", "WH-1000XM5"]
                .into_iter()
                .map(String::from)
                .collect()
        );
        assert_eq!(
            locked_in(&serde_json::json!([{ "solitaryBlockedBy": ["LOCK", "WINDOWED"] }])),
            Some(true)
        );
        assert_eq!(
            locked_in(&serde_json::json!([{ "solitaryBlockedBy": ["WINDOWED"] }])),
            Some(false)
        );
        assert_eq!(
            locked_in(&serde_json::json!([{ "solitaryBlockedBy": ["WORKSPACE"] }])),
            None
        );
    }

    #[test]
    fn a_burst_of_events_runs_an_automation_once() {
        let mut throttle = Throttle::default();
        let start = Instant::now();
        assert!(throttle.allows("flow", 0, start));
        throttle.started("flow", 0, start, None);
        assert!(!throttle.allows("flow", 0, start + Duration::from_secs(1)));
        assert!(
            throttle.allows("flow", 1, start + Duration::from_secs(1)),
            "another automation"
        );
        assert!(throttle.allows("flow", 0, start + MIN_GAP));

        // An interval is counted from when it is first seen.
        assert!(!throttle.every_due("tick", 0, 2, start));
        assert!(!throttle.every_due("tick", 0, 2, start + Duration::from_secs(119)));
        assert!(throttle.every_due("tick", 0, 2, start + Duration::from_secs(120)));
    }

    #[test]
    fn the_service_polls_only_what_a_flow_waits_for() {
        let mut flow = Flow::new("f".into(), "F".into());
        flow.triggers.automations = vec![
            Automation::new(Event::WifiJoined {
                name: String::new(),
            }),
            Automation {
                enabled: false,
                ..Automation::new(Event::Locked)
            },
        ];
        let needs = Needs::of(&[flow]);
        assert!(needs.wifi);
        assert!(!needs.lock, "a switched-off automation needs nothing");
        assert!(!needs.hyprland && !needs.power && !needs.usb);
    }
}
