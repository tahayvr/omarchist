//! The one place Omarchy's version and update state live, shared by the
//! title-bar badge and the Omarchy page.
use std::time::{Duration, Instant};

use gpui::*;

use crate::system::omarchy::updates::{
    PERIODIC_CHECK_INTERVAL, UpdateCheck, check_for_updates, installed_version, update_in_progress,
};

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

    /// Checks now and again every `PERIODIC_CHECK_INTERVAL`.
    pub fn start_periodic(this: Entity<Self>, cx: &mut App) {
        let this = this.downgrade();
        cx.spawn(async move |cx| {
            loop {
                if this.update(cx, |this, cx| this.refresh(cx)).is_err() {
                    break;
                }
                smol::Timer::after(PERIODIC_CHECK_INTERVAL).await;
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
