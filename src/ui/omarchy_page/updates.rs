//! The one place Omarchy's version and update state live, shared by the
//! title-bar badge and the Omarchy page.
use std::time::{Duration, Instant};

use gpui::*;

use crate::system::config::config_setup::settings;
use crate::system::notify;
use crate::system::omarchy::updates::{
    UpdateCheck, check_for_updates, installed_version, update_in_progress,
};

/// How long a disabled background check sleeps before looking at the
/// setting again.
const DISABLED_RECHECK: Duration = Duration::from_secs(60 * 60);

/// A check older than this is re-run when the Omarchy page opens.
const STALE_AFTER: Duration = Duration::from_secs(5 * 60);
/// How long to wait for a launched `omarchy-update` to take its lock before
/// assuming the terminal never started or the user declined.
const LAUNCH_GRACE: Duration = Duration::from_secs(30);
const LOCK_POLL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    Checking,
    UpToDate,
    Available(Vec<String>),
    Failed(String),
    /// `omarchy-update` was launched from here and has not finished.
    Updating,
}

pub struct OmarchyUpdates {
    version: Option<String>,
    state: UpdateState,
    checking: bool,
    last_checked: Option<Instant>,
    /// An update was found by an earlier check, so the next one is not news.
    known_available: bool,
}

impl OmarchyUpdates {
    /// Does nothing until `refresh` or `start_periodic` is called, so a
    /// window can be built without any background work.
    pub fn new() -> Self {
        Self {
            version: None,
            state: UpdateState::Checking,
            checking: false,
            last_checked: None,
            known_available: false,
        }
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn state(&self) -> &UpdateState {
        &self.state
    }

    pub fn available(&self) -> bool {
        matches!(self.state, UpdateState::Available(_))
    }

    /// Re-reads the version and asks Omarchy whether an update is pending.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.checking {
            return;
        }
        self.checking = true;
        self.state = UpdateState::Checking;
        cx.notify();
        let was_available = self.known_available;
        cx.spawn(async move |this, cx| {
            let (version, check) = cx
                .background_spawn(async { (installed_version(), check_for_updates()) })
                .await;
            this.update(cx, |this, cx| {
                this.version = version;
                this.state = match check {
                    Ok(UpdateCheck::UpToDate) => UpdateState::UpToDate,
                    Ok(UpdateCheck::Available(lines)) => UpdateState::Available(lines),
                    Err(e) => UpdateState::Failed(e.to_string()),
                };
                this.checking = false;
                this.last_checked = Some(Instant::now());
                if let UpdateState::Available(lines) = &this.state {
                    // Say so once per update, and only when asked to.
                    if !was_available && settings().notify_updates {
                        notify::send(
                            "Omarchy update available",
                            &lines.join("\n"),
                            notify::Urgency::Low,
                        );
                    }
                    this.known_available = true;
                } else if this.state == UpdateState::UpToDate {
                    this.known_available = false;
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn refresh_if_stale(&mut self, cx: &mut Context<Self>) {
        let stale = self
            .last_checked
            .is_none_or(|checked| checked.elapsed() > STALE_AFTER);
        if stale && self.state != UpdateState::Updating {
            self.refresh(cx);
        }
    }

    /// Waits for a just-launched `omarchy-update` to finish, then refreshes.
    pub fn watch_running_update(&mut self, cx: &mut Context<Self>) {
        self.state = UpdateState::Updating;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let started = Instant::now();
            let mut running = false;
            while started.elapsed() < LAUNCH_GRACE {
                if cx.background_spawn(async { update_in_progress() }).await {
                    running = true;
                    break;
                }
                smol::Timer::after(LOCK_POLL).await;
            }
            while running {
                smol::Timer::after(LOCK_POLL).await;
                running = cx.background_spawn(async { update_in_progress() }).await;
            }
            this.update(cx, |this, cx| this.refresh(cx)).ok();
        })
        .detach();
    }

    /// Checks now and again on the schedule from the Settings page
    /// (`check_updates`, `update_check_hours`). The wait is taken in short
    /// slices against the time of the last check, and the schedule is
    /// re-read on each, so turning the check on or shortening the interval
    /// takes effect within a minute rather than after the wait already
    /// under way.
    pub fn start_periodic(this: Entity<Self>, cx: &mut App) {
        let this = this.downgrade();
        cx.spawn(async move |cx| {
            let slice = Duration::from_secs(60).min(DISABLED_RECHECK);
            let mut last_check: Option<std::time::Instant> = None;
            loop {
                let settings = settings();
                let due = settings.check_updates
                    && last_check.is_none_or(|at| {
                        at.elapsed()
                            >= Duration::from_secs(
                                u64::from(settings.update_check_hours.clamp(1, 24 * 7)) * 3600,
                            )
                    });
                if due {
                    if this.update(cx, |this, cx| this.refresh(cx)).is_err() {
                        break;
                    }
                    last_check = Some(std::time::Instant::now());
                }
                smol::Timer::after(slice).await;
            }
        })
        .detach();
    }
}

impl Default for OmarchyUpdates {
    fn default() -> Self {
        Self::new()
    }
}
