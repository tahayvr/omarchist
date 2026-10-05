//! The flows that are running now, so the bar widget and `omarchist flow
//! stop` can show them and stop them. Every `omarchist flow run` leaves a small
//! file in `$XDG_RUNTIME_DIR/omarchist/runs/` for as long as it runs; the
//! runtime directory is cleared at logout, and a file whose process is
//! gone is ignored and removed.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use super::Flow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Running {
    pub id: String,
    pub name: String,
    pub pid: u32,
    /// When the run began, in seconds since the Unix epoch.
    pub started: i64,
    pub trigger: String,
}

fn dir() -> Option<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir())
        .or_else(dirs::runtime_dir)?;
    Some(runtime.join("omarchist/runs"))
}

/// Removes a run's file when the run ends, however it ends.
pub struct Registered(PathBuf);

impl Drop for Registered {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Marks this process as running `flow` until the returned value is
/// dropped. `None` when there is no runtime directory to mark it in.
pub fn register(flow: &Flow, trigger: &str) -> Option<Registered> {
    let dir = dir()?;
    fs::create_dir_all(&dir).ok()?;
    let pid = std::process::id();
    let path = dir.join(format!("{pid}.json"));
    let running = Running {
        id: flow.id.clone(),
        name: flow.name.clone(),
        pid,
        started: chrono::Local::now().timestamp(),
        trigger: trigger.to_string(),
    };
    fs::write(&path, serde_json::to_string(&running).ok()?).ok()?;
    Some(Registered(path))
}

/// Whether `pid` is still a run of a flow. A pid can be reused after a
/// crash, by anything, Omarchist's own window included, and `stop` sends
/// it a signal: so its command line must say `flow run`.
fn alive(pid: u32) -> bool {
    fs::read(format!("/proc/{pid}/cmdline")).is_ok_and(|cmdline| is_flow_run(&cmdline))
}

/// Whether a NUL-separated command line is `… flow run …`.
fn is_flow_run(cmdline: &[u8]) -> bool {
    let args: Vec<&[u8]> = cmdline.split(|byte| *byte == 0).collect();
    args.windows(2)
        .any(|pair| pair[0] == b"flow" && pair[1] == b"run")
}

/// Every run in progress, oldest first.
pub fn list() -> Vec<Running> {
    let Some(dir) = dir() else {
        return Vec::new();
    };
    let mut running: Vec<Running> = Vec::new();
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let parsed = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Running>(&text).ok());
        match parsed {
            Some(run) if alive(run.pid) => running.push(run),
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
/// records itself as stopped.
pub fn stop(id: &str) -> usize {
    let runs: Vec<Running> = list().into_iter().filter(|run| run.id == id).collect();
    for run in &runs {
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
    use super::is_flow_run;

    #[test]
    fn only_a_flow_run_is_a_run() {
        assert!(is_flow_run(b"/usr/bin/omarchist\0flow\0run\0morning\0"));
        assert!(is_flow_run(
            b"omarchist\0flow\0run\0morning\0--trigger\0Bar\0"
        ));
        // The window, the service, and anything a reused pid may be.
        assert!(!is_flow_run(b"/usr/bin/omarchist\0--view\0flows\0"));
        assert!(!is_flow_run(b"omarchist\0automations\0run\0"));
        assert!(!is_flow_run(b"omarchist\0flow\0list\0--json\0"));
        assert!(!is_flow_run(b"bash\0-c\0flow run\0"));
        assert!(!is_flow_run(b""));
    }
}
