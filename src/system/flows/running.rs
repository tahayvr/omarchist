//! The flows that are running now, so the bar widget, the Flows page and
//! `omarchist flow stop` can show them and stop them. Every run leaves a
//! small file in `$XDG_RUNTIME_DIR/omarchist/runs/` for as long as it
//! runs: an `omarchist flow run` process, which a stop reaches as a
//! signal, or a run inside the window (the editor, a card), which a stop
//! reaches over the instance socket and this module's table of cancels.
//! The runtime directory is cleared at logout, and a file whose process is
//! gone is ignored and removed.
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use super::Flow;
use super::runner::Cancel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Running {
    pub id: String,
    pub name: String,
    pub pid: u32,
    /// When the run began, in seconds since the Unix epoch.
    pub started: i64,
    pub trigger: String,
    /// A run inside the Omarchist window rather than a `flow run` process.
    #[serde(default)]
    pub in_app: bool,
}

/// The runs of this process that can be cancelled, by their file name.
static LOCAL: Mutex<Option<HashMap<String, (String, Cancel)>>> = Mutex::new(None);
static SEQ: AtomicU64 = AtomicU64::new(0);

fn local() -> std::sync::MutexGuard<'static, Option<HashMap<String, (String, Cancel)>>> {
    LOCAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn dir() -> Option<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir())
        .or_else(dirs::runtime_dir)?;
    Some(runtime.join("omarchist/runs"))
}

/// Removes a run's file (and its cancel) when the run ends, however it ends.
pub struct Registered(PathBuf);

impl Registered {
    /// The flow id this run is registered under now: the one it started
    /// with, or the one `relabel_local` gave it when the flow was saved.
    pub fn flow_id(&self) -> Option<String> {
        let name = self.0.file_name()?.to_str()?;
        local()
            .as_ref()?
            .get(name)
            .map(|(id, _)| id.clone())
            .filter(|id| !id.is_empty())
    }
}

impl Drop for Registered {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        if let Some(name) = self.0.file_name().and_then(|n| n.to_str())
            && let Some(table) = local().as_mut()
        {
            table.remove(name);
        }
    }
}

/// Marks this `flow run` process as running `flow` until the returned
/// value is dropped. `None` when there is no runtime directory.
pub fn register(flow: &Flow, trigger: &str) -> Option<Registered> {
    write_entry(flow, trigger, None)
}

/// Marks a run inside the window, which `cancel` ends. A stop from
/// anywhere (the bar widget, `flow stop`, a card) reaches it through
/// [`stop_local`], in this process or over the instance socket.
pub fn register_in_app(flow: &Flow, trigger: &str, cancel: Cancel) -> Option<Registered> {
    write_entry(flow, trigger, Some(cancel))
}

fn write_entry(flow: &Flow, trigger: &str, cancel: Option<Cancel>) -> Option<Registered> {
    let dir = dir()?;
    fs::create_dir_all(&dir).ok()?;
    let pid = std::process::id();
    let name = format!("{pid}-{}.json", SEQ.fetch_add(1, Ordering::SeqCst));
    let path = dir.join(&name);
    let running = Running {
        id: flow.id.clone(),
        name: flow.name.clone(),
        pid,
        started: chrono::Local::now().timestamp(),
        trigger: trigger.to_string(),
        in_app: cancel.is_some(),
    };
    write_run(&path, &running)?;
    if let Some(cancel) = cancel {
        local()
            .get_or_insert_with(HashMap::new)
            .insert(name, (flow.id.clone(), cancel));
    }
    Some(Registered(path))
}

/// A run that started before its flow was first saved carries no id.
/// Once the flow has one, the runs of this process that lack an id take
/// it, so a stop by id reaches them.
pub fn relabel_local(flow: &Flow) {
    let Some(dir) = dir() else {
        return;
    };
    let mut table = local();
    for (name, (id, _)) in table.iter_mut().flat_map(|t| t.iter_mut()) {
        if !id.is_empty() {
            continue;
        }
        let path = dir.join(name);
        if let Some(mut run) = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Running>(&text).ok())
        {
            run.id = flow.id.clone();
            run.name = flow.name.clone();
            write_run(&path, &run);
        }
        *id = flow.id.clone();
    }
}

/// Cancels the runs of `id` inside this process; how many there were.
pub fn stop_local(id: &str) -> usize {
    let table = local();
    let cancels: Vec<&Cancel> = table
        .as_ref()
        .into_iter()
        .flat_map(|t| t.values())
        .filter(|(flow, _)| flow == id)
        .map(|(_, cancel)| cancel)
        .collect();
    for cancel in &cancels {
        cancel.cancel();
    }
    cancels.len()
}

/// Writes an entry whole: the bar widget and the Flows page read the
/// folder as it changes, and a half-written file would read as a run that
/// is gone.
fn write_run(path: &Path, run: &Running) -> Option<()> {
    let text = serde_json::to_string(run).ok()?;
    let temp = path.with_extension("tmp");
    fs::write(&temp, text).ok()?;
    fs::rename(&temp, path).ok()
}

/// Whether a run's process is still that run. A pid can be reused after a
/// crash, by anything, and `stop` signals it: a `flow run` must name this
/// flow on its command line, and a run inside the window must be an
/// Omarchist.
fn alive(run: &Running) -> bool {
    let Ok(cmdline) = fs::read(format!("/proc/{}/cmdline", run.pid)) else {
        return false;
    };
    if run.in_app {
        is_omarchist(&cmdline)
    } else {
        is_flow_run_of(&cmdline, &run.id, &run.name)
    }
}

fn is_omarchist(cmdline: &[u8]) -> bool {
    cmdline
        .split(|byte| *byte == 0)
        .next()
        .and_then(|argv0| argv0.rsplit(|b| *b == b'/').next())
        .is_some_and(|name| name == b"omarchist")
}

/// Whether a NUL-separated command line is `… flow run <this flow> …`,
/// by id or by name, as the command line takes either.
fn is_flow_run_of(cmdline: &[u8], id: &str, name: &str) -> bool {
    let args: Vec<&[u8]> = cmdline.split(|byte| *byte == 0).collect();
    args.windows(3).any(|w| {
        w[0] == b"flow"
            && w[1] == b"run"
            && (w[2] == id.as_bytes()
                || w[2].eq_ignore_ascii_case(name.as_bytes())
                || std::str::from_utf8(w[2]).is_ok_and(|given| super::slug(given) == id))
    })
}

/// Every run in progress, oldest first.
pub fn list() -> Vec<Running> {
    let Some(dir) = dir() else {
        return Vec::new();
    };
    let mut running: Vec<Running> = Vec::new();
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        // An entry on its way in is not a run that is gone.
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let parsed = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Running>(&text).ok());
        match parsed {
            Some(run) if alive(&run) => running.push(run),
            // Left behind by a run that was killed.
            _ => {
                let _ = fs::remove_file(&path);
            }
        }
    }
    running.sort_by_key(|run| (run.started, run.pid));
    running
}

/// Asks every run of the flow `id` to stop, and says how many there were.
/// The run ends its current step, kills a command it is waiting for, and
/// records itself as stopped. A `flow run` process gets a signal; a run
/// inside a window is cancelled here when it is this process, and asked
/// over the instance socket otherwise.
pub fn stop(id: &str) -> usize {
    let me = std::process::id();
    let runs: Vec<Running> = list().into_iter().filter(|run| run.id == id).collect();
    let mut asked_window = false;
    for run in &runs {
        if run.in_app {
            if run.pid == me {
                stop_local(id);
            } else if !asked_window {
                asked_window = true;
                crate::system::instance::request_stop(id);
            }
            continue;
        }
        let _ = Command::new("kill")
            .args(["-TERM", &run.pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    runs.len()
}

#[cfg(test)]
mod tests {
    use super::{is_flow_run_of, is_omarchist};

    #[test]
    fn a_window_is_an_omarchist_by_its_program_name() {
        assert!(is_omarchist(b"/usr/bin/omarchist\0--view\0flows\0"));
        assert!(is_omarchist(b"omarchist\0"));
        assert!(!is_omarchist(b"/usr/bin/bash\0omarchist\0"));
        assert!(!is_omarchist(b""));
    }

    #[test]
    fn a_local_run_is_cancelled_by_its_flow_id() {
        use super::{register_in_app, stop_local};
        use crate::system::flows::Flow;
        use crate::system::flows::runner::Cancel;
        let flow = Flow::new("test-stop-local".into(), "Test".into());
        let cancel = Cancel::new();
        let registered = register_in_app(&flow, "Test", cancel.clone());
        if registered.is_none() {
            return; // no runtime directory here
        }
        assert_eq!(stop_local("another"), 0);
        assert_eq!(stop_local("test-stop-local"), 1);
        assert!(cancel.is_cancelled());
        drop(registered);
        assert_eq!(stop_local("test-stop-local"), 0);
    }

    #[test]
    fn only_a_run_of_this_flow_is_its_run() {
        let run = b"/usr/bin/omarchist\0flow\0run\0morning\0";
        assert!(is_flow_run_of(run, "morning", "Morning start"));
        // By name, as the command line takes one.
        assert!(is_flow_run_of(
            b"omarchist\0flow\0run\0Morning start\0",
            "morning-start",
            "Morning start"
        ));
        // A pid reused by another flow's run is not this one.
        assert!(!is_flow_run_of(run, "evening", "Evening"));
        assert!(!is_flow_run_of(
            b"/usr/bin/omarchist\0flow\0list\0",
            "morning",
            "Morning"
        ));
        assert!(!is_flow_run_of(b"/usr/bin/bash\0", "morning", "Morning"));
    }
}
