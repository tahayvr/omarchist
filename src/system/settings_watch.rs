//! Tells the Configuration page when something it shows changed outside
//! it: the bar's Do Not Disturb, a flag Omarchy's menu sets, a Hyprland
//! reload after `hyprland.lua` was edited. Files are watched through
//! inotify (one watch per folder, nothing read until a file in it is
//! written) and Hyprland's event socket is read on a thread of its own,
//! so an open page costs nothing while nothing changes.
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ::notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use smol::channel::Sender;

use crate::system::flows::automations::hyprland_socket;

/// Something the page shows may have changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// This file (or a folder on the way to one) was written, created or
    /// removed.
    File(PathBuf),
    /// Hyprland reloaded its configuration.
    HyprlandReloaded,
}

/// Watches while it lives; dropping it stops both watchers.
pub struct SettingsWatch {
    files: Option<RecommendedWatcher>,
    /// Folders that hold a wanted file and are watched.
    watched: HashSet<PathBuf>,
    /// Folders that hold a wanted file and do not exist yet; their
    /// nearest existing ancestor is watched instead until they appear.
    missing: HashSet<PathBuf>,
    _hyprland: HyprlandEvents,
}

impl SettingsWatch {
    /// Starts watching `files` and Hyprland's reloads, sending what
    /// changed on `tx`.
    pub fn start(files: &HashSet<PathBuf>, tx: Sender<Change>) -> Self {
        let interesting = interesting(files);
        let file_tx = tx.clone();
        let files_watcher =
            ::notify::recommended_watcher(move |event: ::notify::Result<::notify::Event>| {
                let Ok(event) = event else { return };
                if matches!(event.kind, EventKind::Access(_)) {
                    return;
                }
                for path in event.paths {
                    if interesting.contains(&path) {
                        let _ = file_tx.try_send(Change::File(path));
                    }
                }
            })
            .map_err(|e| log::warn!("Could not watch Omarchy's settings: {e}"))
            .ok();
        let mut watch = Self {
            files: files_watcher,
            watched: HashSet::new(),
            missing: files
                .iter()
                .filter_map(|f| f.parent().map(Path::to_path_buf))
                .collect(),
            _hyprland: HyprlandEvents::start(tx),
        };
        watch.rearm();
        watch
    }

    /// Watches every wanted folder that exists now, and the nearest
    /// existing ancestor of each one that does not. Run after a change
    /// that is not a wanted file: a folder on the way may have appeared.
    pub fn rearm(&mut self) {
        let Some(watcher) = self.files.as_mut() else {
            return;
        };
        // A watched folder that was removed lost its watch with it.
        let gone: Vec<PathBuf> = self
            .watched
            .iter()
            .filter(|d| !d.is_dir())
            .cloned()
            .collect();
        for dir in gone {
            self.watched.remove(&dir);
            self.missing.insert(dir);
        }
        let appeared: Vec<PathBuf> = self
            .missing
            .iter()
            .filter(|d| d.is_dir())
            .cloned()
            .collect();
        for dir in appeared {
            if watcher.watch(&dir, RecursiveMode::NonRecursive).is_ok() {
                self.missing.remove(&dir);
                self.watched.insert(dir);
            }
        }
        for dir in &self.missing {
            if let Some(ancestor) = dir.ancestors().skip(1).find(|a| a.is_dir()) {
                // Watching a folder twice is harmless; inotify keeps one.
                let _ = watcher.watch(ancestor, RecursiveMode::NonRecursive);
            }
        }
    }
}

/// The wanted files and every folder on the way to them: a change to
/// anything else is not this page's business.
fn interesting(files: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    files
        .iter()
        .flat_map(|f| f.ancestors().map(Path::to_path_buf))
        .collect()
}

/// A thread reading Hyprland's event socket for `configreloaded`.
struct HyprlandEvents {
    stop: Arc<AtomicBool>,
    stream: Arc<Mutex<Option<UnixStream>>>,
}

impl HyprlandEvents {
    fn start(tx: Sender<Change>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stream: Arc<Mutex<Option<UnixStream>>> = Arc::new(Mutex::new(None));
        let (thread_stop, thread_stream) = (stop.clone(), stream.clone());
        let spawned = std::thread::Builder::new()
            .name("settings-hyprland-events".into())
            .spawn(move || {
                while !thread_stop.load(Ordering::Relaxed) {
                    if let Some(socket) =
                        hyprland_socket().and_then(|path| UnixStream::connect(path).ok())
                    {
                        if let Ok(clone) = socket.try_clone()
                            && let Ok(mut held) = thread_stream.lock()
                        {
                            *held = Some(clone);
                        }
                        // Dropped while connecting: the shutdown missed this stream.
                        if thread_stop.load(Ordering::Relaxed) {
                            return;
                        }
                        for line in BufReader::new(socket).lines() {
                            let Ok(line) = line else { break };
                            if is_reload(&line) && tx.try_send(Change::HyprlandReloaded).is_err() {
                                return;
                            }
                        }
                    }
                    // The compositor restarted, or is not there: try again.
                    std::thread::sleep(Duration::from_secs(5));
                }
            });
        if let Err(e) = spawned {
            log::warn!("Could not read Hyprland's events: {e}");
        }
        Self { stop, stream }
    }
}

impl Drop for HyprlandEvents {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // Ends the blocking read, and with it the thread.
        if let Ok(held) = self.stream.lock()
            && let Some(stream) = held.as_ref()
        {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

fn is_reload(line: &str) -> bool {
    line.split_once(">>").map_or(line, |(name, _)| name) == "configreloaded"
}

#[cfg(test)]
mod tests {
    use super::{Change, SettingsWatch, interesting, is_reload};
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn only_a_reload_counts() {
        assert!(is_reload("configreloaded>>"));
        assert!(is_reload("configreloaded"));
        assert!(!is_reload("activewindow>>kitty,~"));
        assert!(!is_reload("configreloadedish>>"));
    }

    #[test]
    fn a_file_and_the_folders_on_the_way_are_interesting() {
        let files = HashSet::from([PathBuf::from("/a/b/c.json")]);
        let set = interesting(&files);
        assert!(set.contains(&PathBuf::from("/a/b/c.json")));
        assert!(set.contains(&PathBuf::from("/a/b")));
        assert!(!set.contains(&PathBuf::from("/a/b/d.json")));
    }

    /// A flag written, removed, and written in a folder that did not exist
    /// when watching began, as Omarchy's toggles do.
    #[test]
    fn files_are_seen_even_in_folders_made_later() {
        let dir = std::env::temp_dir().join(format!("omarchist-watch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let flag = dir.join("toggles").join("hypr").join("flag");
        let (tx, rx) = smol::channel::unbounded();
        let mut watch = SettingsWatch::start(&HashSet::from([flag.clone()]), tx);
        // The next file change, skipping the live compositor's reloads.
        let next = || {
            smol::block_on(async {
                smol::future::or(
                    async {
                        loop {
                            match rx.recv().await {
                                Ok(Change::File(path)) => return Some(path),
                                Ok(Change::HyprlandReloaded) => continue,
                                Err(_) => return None,
                            }
                        }
                    },
                    async {
                        smol::Timer::after(Duration::from_secs(5)).await;
                        None
                    },
                )
                .await
            })
        };

        std::fs::create_dir_all(flag.parent().unwrap()).unwrap();
        // The page re-arms on any change that is not a wanted file.
        assert_eq!(next(), Some(dir.join("toggles")));
        watch.rearm();
        while rx.try_recv().is_ok() {}

        std::fs::write(&flag, "").unwrap();
        assert_eq!(next(), Some(flag.clone()));
        while rx.try_recv().is_ok() {}
        std::fs::remove_file(&flag).unwrap();
        assert_eq!(next(), Some(flag.clone()));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
