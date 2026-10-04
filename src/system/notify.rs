//! Desktop notifications, for feedback from things that have no window:
//! a flow run from a keybind, an update found in the background.
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy)]
pub enum Urgency {
    Low,
    Normal,
}

/// Sends a notification with `notify-send`, which Omarchy's shell shows.
/// Failures are ignored: there is nobody to report them to.
pub fn send(title: &str, body: &str, urgency: Urgency) {
    let urgency = match urgency {
        Urgency::Low => "low",
        Urgency::Normal => "normal",
    };
    let _ = Command::new("notify-send")
        .args(["-a", "Omarchist", "-u", urgency, "--", title, body])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
