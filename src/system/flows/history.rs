//! What each run of a flow did: when, what started it, how long it took,
//! and how every step ended. Kept per flow as one JSON line per run under
//! `~/.local/state/omarchist/runs/`, newest last, a bounded number of them.
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::runner::{Outcome, RunEvent, STOPPED, StepStatus};
use super::{Flow, StepPath};

/// Runs kept per flow.
pub const KEEP: usize = 50;
/// Step lines kept per run; a loop can produce any number.
const MAX_STEPS: usize = 300;
/// How much of a step's output or error is kept.
const MAX_DETAIL: usize = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunResult {
    Finished,
    Failed,
    /// A prompt was dismissed.
    Cancelled,
    /// Stopped from outside before it was done.
    Stopped,
}

impl RunResult {
    pub fn label(self) -> &'static str {
        match self {
            RunResult::Finished => "Finished",
            RunResult::Failed => "Failed",
            RunResult::Cancelled => "Cancelled",
            RunResult::Stopped => "Stopped",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepResult {
    Done,
    Failed,
    Cancelled,
}

/// One step of one run. A step inside a loop appears once per round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepRun {
    /// The step's number in the flow, as the editor shows it.
    pub number: usize,
    /// How many blocks the step sits inside.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub depth: usize,
    /// What the step does, with `{{variables}}` as written.
    pub title: String,
    pub result: StepResult,
    /// The start of what it produced, or why it failed.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub detail: String,
    pub ms: u64,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    /// When the run began, in seconds since the Unix epoch.
    pub started: i64,
    pub ms: u64,
    /// What started it: "Editor", "Terminal", an automation's sentence.
    pub trigger: String,
    pub result: RunResult,
    /// The one-line summary a toast or the command line showed.
    pub summary: String,
    #[serde(default)]
    pub steps: Vec<StepRun>,
}

/// Collects a run's events into a [`Run`].
pub struct Recorder {
    started_at: i64,
    started: Instant,
    trigger: String,
    /// Each step's number and title, by path, taken when the run began.
    titles: HashMap<StepPath, (usize, String)>,
    /// Steps that have started and not ended: when, and their line.
    open: HashMap<StepPath, (Instant, usize)>,
    steps: Vec<StepRun>,
}

impl Recorder {
    pub fn new(flow: &Flow, trigger: &str) -> Self {
        let titles = flow
            .walk()
            .into_iter()
            .enumerate()
            .map(|(ix, (path, step))| (path, (ix + 1, step.kind.text())))
            .collect();
        Self {
            started_at: chrono::Local::now().timestamp(),
            started: Instant::now(),
            trigger: trigger.to_string(),
            titles,
            open: HashMap::new(),
            steps: Vec::new(),
        }
    }

    pub fn event(&mut self, event: &RunEvent) {
        match event {
            // A step takes its line when it starts, so a block is listed
            // before the steps it holds and not after them.
            RunEvent::Started { path } => {
                if self.steps.len() >= MAX_STEPS {
                    return;
                }
                let (number, title) = self.titles.get(path).cloned().unwrap_or_default();
                self.open
                    .insert(path.clone(), (Instant::now(), self.steps.len()));
                self.steps.push(StepRun {
                    number,
                    depth: path.len() / 2,
                    title,
                    // What it stays if the run is cut off under it.
                    result: StepResult::Cancelled,
                    detail: String::new(),
                    ms: 0,
                });
            }
            RunEvent::Finished { path, status } => {
                let Some((began, index)) = self.open.remove(path) else {
                    return;
                };
                let Some(step) = self.steps.get_mut(index) else {
                    return;
                };
                let (result, detail) = match status {
                    StepStatus::Done(output) => (
                        StepResult::Done,
                        output
                            .as_deref()
                            .and_then(|o| o.lines().find(|l| !l.trim().is_empty()))
                            .unwrap_or_default()
                            .to_string(),
                    ),
                    StepStatus::Failed(error) => (StepResult::Failed, error.clone()),
                    StepStatus::Cancelled => (StepResult::Cancelled, String::new()),
                };
                step.result = result;
                step.detail = clip(&detail);
                step.ms = began.elapsed().as_millis() as u64;
            }
            RunEvent::Round { .. } => {}
        }
    }

    /// The finished record. `stopped` says the run was ended from outside
    /// (the Stop button, `omarchist flow stop`).
    pub fn finish(mut self, flow: &Flow, outcome: &Outcome, stopped: bool) -> Run {
        if stopped {
            // The step the stop cut short did not fail; it was stopped.
            for step in &mut self.steps {
                if step.result == StepResult::Failed && step.detail == STOPPED {
                    step.result = StepResult::Cancelled;
                    step.detail.clear();
                }
            }
        }
        let result = if stopped {
            RunResult::Stopped
        } else if !outcome.is_ok() {
            RunResult::Failed
        } else if outcome.cancelled {
            RunResult::Cancelled
        } else {
            RunResult::Finished
        };
        Run {
            started: self.started_at,
            ms: self.started.elapsed().as_millis() as u64,
            trigger: self.trigger,
            result,
            summary: if stopped {
                format!("Flow '{}' stopped", flow.name)
            } else {
                outcome.summary(flow)
            },
            steps: self.steps,
        }
    }
}

fn clip(text: &str) -> String {
    let text = text.trim();
    if text.len() <= MAX_DETAIL {
        return text.to_string();
    }
    let mut end = MAX_DETAIL;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}

/// `$XDG_STATE_HOME/omarchist` (`~/.local/state/omarchist`).
pub fn state_dir() -> Result<PathBuf> {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .or_else(|| dirs::home_dir().map(|home| home.join(".local/state")))
        .map(|dir| dir.join("omarchist"))
        .ok_or(Error::UnknownDirectory("state"))
}

fn history_path(id: &str) -> Result<PathBuf> {
    Ok(state_dir()?.join("runs").join(format!("{id}.jsonl")))
}

/// Adds a run to the flow's history, dropping the oldest beyond [`KEEP`].
pub fn record(id: &str, run: &Run) -> Result<()> {
    if id.is_empty() || super::store::load_flow(id).is_err() {
        // A flow that was never saved, or was deleted while it ran, has
        // nowhere to keep a history.
        return Ok(());
    }
    let path = history_path(id)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| Error::io("Failed to create the history folder", e))?;
    }
    let line = serde_json::to_string(run)
        .map_err(|e| Error::json("Failed to write the run history", e))?;
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<&str> = existing.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() >= KEEP + KEEP / 2 {
        // Rewritten only now and then, so a run usually costs one append.
        let mut kept: Vec<&str> = lines[lines.len() - (KEEP - 1)..].to_vec();
        kept.push(&line);
        return crate::system::fs::write_atomic(&path, kept.join("\n") + "\n", "the run history");
    }
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| writeln!(file, "{line}"))
        .map_err(|e| Error::io("Failed to write the run history", e))
}

/// The flow's runs, newest first, at most [`KEEP`]. A line that does not
/// parse (a newer Omarchist's, a torn write) is skipped.
pub fn load(id: &str) -> Vec<Run> {
    let Ok(path) = history_path(id) else {
        return Vec::new();
    };
    let mut runs: Vec<Run> = fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    runs.reverse();
    runs.truncate(KEEP);
    runs
}

/// The newest run of the flow, if it ever ran.
pub fn last(id: &str) -> Option<Run> {
    load(id).into_iter().next()
}

/// Forgets the flow's runs.
pub fn clear(id: &str) -> Result<()> {
    match fs::remove_file(history_path(id)?) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io("Failed to clear the run history", e)),
    }
}

/// "just now", "5 min ago", "3 h ago", "2 days ago", then the date.
pub fn ago(started: i64, now: i64) -> String {
    let seconds = (now - started).max(0);
    match seconds {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", seconds / 60),
        3600..=86_399 => format!("{} h ago", seconds / 3600),
        86_400..=172_799 => "yesterday".to_string(),
        172_800..=604_799 => format!("{} days ago", seconds / 86_400),
        _ => chrono::DateTime::from_timestamp(started, 0)
            .map(|time| {
                time.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d")
                    .to_string()
            })
            .unwrap_or_default(),
    }
}

/// "0.4 s", "12 s", "3 min 5 s".
pub fn took(ms: u64) -> String {
    match ms {
        0..=9_949 => format!("{:.1} s", ms as f64 / 1000.0),
        9_950..=59_499 => format!("{} s", (ms + 500) / 1000),
        _ => {
            let seconds = (ms + 500) / 1000;
            format!("{} min {} s", seconds / 60, seconds % 60)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::flows::{Step, StepKind};

    fn event_pair(recorder: &mut Recorder, path: &[usize], status: StepStatus) {
        recorder.event(&RunEvent::Started {
            path: path.to_vec(),
        });
        recorder.event(&RunEvent::Finished {
            path: path.to_vec(),
            status,
        });
    }

    fn flow() -> Flow {
        let mut flow = Flow::new("demo".into(), "Demo".into());
        flow.steps = vec![
            Step::new(StepKind::Exec {
                command: "echo {{input}}".into(),
                wait: true,
            }),
            Step::new(StepKind::Repeat {
                times: 2,
                steps: vec![Step::new(StepKind::Wait { ms: 1 })],
            }),
        ];
        flow
    }

    #[test]
    fn a_run_is_recorded_step_by_step() {
        let flow = flow();
        let mut recorder = Recorder::new(&flow, "Editor");
        event_pair(
            &mut recorder,
            &[0],
            StepStatus::Done(Some("\n first line \nsecond".into())),
        );
        recorder.event(&RunEvent::Started { path: vec![1] });
        recorder.event(&RunEvent::Round {
            path: vec![1],
            round: 1,
            of: 2,
        });
        event_pair(&mut recorder, &[1, 0, 0], StepStatus::Done(None));
        event_pair(
            &mut recorder,
            &[1, 0, 0],
            StepStatus::Failed("it broke".into()),
        );
        recorder.event(&RunEvent::Finished {
            path: vec![1],
            status: StepStatus::Done(None),
        });
        let outcome = Outcome {
            ran: 4,
            failures: vec![(vec![1, 0, 0], "it broke".into())],
            stopped_at: Some(vec![1, 0, 0]),
            ..Outcome::default()
        };
        let run = recorder.finish(&flow, &outcome, false);
        assert_eq!(run.result, RunResult::Failed);
        assert_eq!(run.trigger, "Editor");
        assert!(run.summary.contains("stopped at step 3"), "{}", run.summary);
        let steps: Vec<(usize, usize, StepResult, &str)> = run
            .steps
            .iter()
            .map(|s| (s.number, s.depth, s.result, s.detail.as_str()))
            .collect();
        assert_eq!(
            steps,
            vec![
                (1, 0, StepResult::Done, "first line"),
                // The loop, then what ran inside it, once per round.
                (2, 0, StepResult::Done, ""),
                (3, 1, StepResult::Done, ""),
                (3, 1, StepResult::Failed, "it broke"),
            ]
        );
        assert_eq!(run.steps[0].title, "echo {{input}}");

        // Dismissing a prompt and stopping from outside are told apart.
        let cancelled = Outcome {
            cancelled: true,
            ..Outcome::default()
        };
        assert_eq!(
            Recorder::new(&flow, "x")
                .finish(&flow, &cancelled, false)
                .result,
            RunResult::Cancelled
        );
        let mut recorder = Recorder::new(&flow, "x");
        recorder.event(&RunEvent::Started { path: vec![0] });
        recorder.event(&RunEvent::Finished {
            path: vec![0],
            status: StepStatus::Failed(STOPPED.into()),
        });
        let stopped = recorder.finish(&flow, &outcome, true);
        assert_eq!(stopped.result, RunResult::Stopped);
        assert_eq!(stopped.summary, "Flow 'Demo' stopped");
        // The step that was cut short is not listed as a failure.
        assert_eq!(stopped.steps[0].result, StepResult::Cancelled);
        assert_eq!(stopped.steps[0].detail, "");
    }

    #[test]
    fn long_details_are_clipped_on_a_character() {
        let clipped = clip(&"é".repeat(400));
        assert!(clipped.len() <= MAX_DETAIL + "…".len());
        assert!(clipped.ends_with('…'));
        assert_eq!(clip("  short  "), "short");
    }

    #[test]
    fn times_read_plainly() {
        assert_eq!(ago(1000, 1030), "just now");
        assert_eq!(ago(1000, 1000 + 5 * 60), "5 min ago");
        assert_eq!(ago(1000, 1000 + 3 * 3600), "3 h ago");
        assert_eq!(ago(1000, 1000 + 30 * 3600), "yesterday");
        assert_eq!(ago(1000, 1000 + 4 * 86_400), "4 days ago");
        assert_eq!(ago(2000, 1000), "just now", "a clock that went back");
        assert_eq!(took(420), "0.4 s");
        assert_eq!(took(12_300), "12 s");
        assert_eq!(took(185_000), "3 min 5 s");
    }

    #[test]
    fn a_run_survives_a_round_trip_and_unknown_lines_are_skipped() {
        let run = Run {
            started: 1_800_000_000,
            ms: 1234,
            trigger: "At 09:00".into(),
            result: RunResult::Finished,
            summary: "Flow 'Demo' finished".into(),
            steps: vec![StepRun {
                number: 1,
                depth: 0,
                title: "echo hi".into(),
                result: StepResult::Done,
                detail: "hi".into(),
                ms: 12,
            }],
        };
        let line = serde_json::to_string(&run).unwrap();
        assert!(!line.contains('\n'));
        let parsed: Vec<Run> = format!("{line}\nnot json\n{{\"future\": true}}\n{line}\n")
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        assert_eq!(parsed, vec![run.clone(), run]);
    }
}
