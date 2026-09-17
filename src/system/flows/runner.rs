//! Runs a flow's steps in order and reports what happened to each one.
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::error::Result;

use super::store::load_flow;
use super::{Flow, MAX_DEPTH, OnError, StepKind};

/// Progress of one top-level run. Nested flows report as a single step of
/// their parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunEvent {
    Started { index: usize },
    Finished { index: usize, error: Option<String> },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Steps that ran (enabled ones, up to where the flow stopped).
    pub ran: usize,
    pub failures: Vec<(usize, String)>,
    /// The step the flow stopped at, when `OnError::Stop` cut it short.
    pub stopped_at: Option<usize>,
}

impl Outcome {
    pub fn is_ok(&self) -> bool {
        self.failures.is_empty()
    }

    /// One line for a toast or the command line.
    pub fn summary(&self, flow: &Flow) -> String {
        match (self.failures.as_slice(), self.stopped_at) {
            ([], _) => format!("Flow '{}' finished", flow.name),
            ([(index, error)], Some(_)) => {
                format!(
                    "Flow '{}' stopped at step {}: {error}",
                    flow.name,
                    index + 1
                )
            }
            (failures, _) => format!(
                "Flow '{}' finished with {} failed step{}",
                flow.name,
                failures.len(),
                if failures.len() == 1 { "" } else { "s" }
            ),
        }
    }
}

pub type Loader<'a> = &'a dyn Fn(&str) -> Result<Flow>;

pub struct Runner<'a> {
    load: Loader<'a>,
    /// Discard step output rather than inheriting the caller's stdio.
    quiet: bool,
}

impl<'a> Runner<'a> {
    /// Loads nested flows from disk.
    pub fn new(quiet: bool) -> Runner<'static> {
        Runner {
            load: &load_from_disk,
            quiet,
        }
    }

    pub fn with_loader(load: Loader<'a>, quiet: bool) -> Self {
        Self { load, quiet }
    }

    pub fn run(&self, flow: &Flow, on_event: &mut dyn FnMut(RunEvent)) -> Outcome {
        let mut stack = vec![flow.id.clone()];
        self.run_nested(flow, &mut stack, on_event)
    }

    fn run_nested(
        &self,
        flow: &Flow,
        stack: &mut Vec<String>,
        on_event: &mut dyn FnMut(RunEvent),
    ) -> Outcome {
        let mut outcome = Outcome::default();
        for (index, step) in flow.steps.iter().enumerate() {
            if !step.enabled {
                continue;
            }
            on_event(RunEvent::Started { index });
            let result = self.run_step(&step.kind, stack);
            outcome.ran += 1;
            on_event(RunEvent::Finished {
                index,
                error: result.as_ref().err().cloned(),
            });
            if let Err(error) = result {
                outcome.failures.push((index, error));
                if flow.on_error == OnError::Stop {
                    outcome.stopped_at = Some(index);
                    break;
                }
            }
        }
        outcome
    }

    fn run_step(
        &self,
        kind: &StepKind,
        stack: &mut Vec<String>,
    ) -> std::result::Result<(), String> {
        match kind {
            StepKind::Exec { command, detach } => self.exec(command, *detach),
            StepKind::Lua { expr } => dispatch(expr),
            StepKind::Wait { ms } => {
                std::thread::sleep(Duration::from_millis(*ms));
                Ok(())
            }
            StepKind::Notify { title, body } => notify(title, body),
            StepKind::Flow { id } => self.run_flow_step(id, stack),
        }
    }

    fn exec(&self, command: &str, detach: bool) -> std::result::Result<(), String> {
        let mut cmd = if detach {
            // `setsid -f` forks the command into its own session and returns
            // at once, so the flow moves on and nothing is left to reap.
            let mut cmd = Command::new("setsid");
            cmd.args(["-f", "sh", "-c", command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command]);
            cmd
        };
        cmd.stdin(Stdio::null());
        if self.quiet || detach {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }
        let status = cmd
            .status()
            .map_err(|e| format!("Could not start the command: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(match status.code() {
                Some(code) => format!("The command exited with status {code}"),
                None => "The command was killed by a signal".to_string(),
            })
        }
    }

    fn run_flow_step(&self, id: &str, stack: &mut Vec<String>) -> std::result::Result<(), String> {
        if stack.iter().any(|s| s == id) {
            return Err(format!(
                "Flow '{id}' is already running further up this flow"
            ));
        }
        if stack.len() >= MAX_DEPTH {
            return Err(format!("Flows are nested more than {MAX_DEPTH} deep"));
        }
        let nested = (self.load)(id).map_err(|e| e.to_string())?;
        stack.push(nested.id.clone());
        let outcome = self.run_nested(&nested, stack, &mut |_| {});
        stack.pop();
        if outcome.is_ok() {
            Ok(())
        } else {
            Err(outcome.summary(&nested))
        }
    }
}

fn load_from_disk(id: &str) -> Result<Flow> {
    load_flow(id)
}

fn dispatch(expr: &str) -> std::result::Result<(), String> {
    let output = Command::new("hyprctl")
        .args(["dispatch", expr])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("Could not run hyprctl: {e}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if output.status.success() && text == "ok" {
        Ok(())
    } else if text.is_empty() {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    } else {
        Err(text.to_string())
    }
}

fn notify(title: &str, body: &str) -> std::result::Result<(), String> {
    let mut cmd = Command::new("notify-send");
    cmd.args(["-a", "Omarchist", title.trim()]);
    if !body.trim().is_empty() {
        cmd.arg(body.trim());
    }
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("Could not send the notification: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("notify-send failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::system::flows::Step;

    fn exec(command: &str) -> Step {
        Step::new(StepKind::Exec {
            command: command.into(),
            detach: false,
        })
    }

    fn events(flow: &Flow, loader: Loader) -> (Outcome, Vec<RunEvent>) {
        let mut seen = Vec::new();
        let outcome = Runner::with_loader(loader, true).run(flow, &mut |e| seen.push(e));
        (outcome, seen)
    }

    fn no_flows(id: &str) -> Result<Flow> {
        Err(Error::Invalid(format!("no flow {id}")))
    }

    #[test]
    fn stops_at_the_first_failure_by_default() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![exec("true"), exec("exit 3"), exec("true")];
        let (outcome, seen) = events(&flow, &no_flows);
        assert_eq!(outcome.ran, 2);
        assert_eq!(outcome.stopped_at, Some(1));
        assert_eq!(outcome.failures.len(), 1);
        assert!(outcome.failures[0].1.contains("status 3"));
        assert_eq!(seen.len(), 4);
        assert_eq!(
            seen[3],
            RunEvent::Finished {
                index: 1,
                error: Some("The command exited with status 3".into())
            }
        );
        assert!(outcome.summary(&flow).contains("stopped at step 2"));
    }

    #[test]
    fn keeps_going_and_skips_disabled_steps() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.on_error = OnError::Continue;
        flow.steps = vec![
            exec("false"),
            Step {
                enabled: false,
                ..exec("exit 9")
            },
            Step::new(StepKind::Wait { ms: 1 }),
            exec("true"),
        ];
        let (outcome, seen) = events(&flow, &no_flows);
        assert_eq!(outcome.ran, 3);
        assert_eq!(outcome.stopped_at, None);
        assert_eq!(outcome.failures.len(), 1);
        assert!(
            seen.iter()
                .all(|e| !matches!(e, RunEvent::Started { index: 1 }))
        );
        assert!(outcome.summary(&flow).contains("1 failed step"));
    }

    #[test]
    fn nested_flows_run_and_cycles_are_refused() {
        let mut inner = Flow::new("inner".into(), "Inner".into());
        inner.steps = vec![exec("true")];
        let mut looping = Flow::new("loop".into(), "Loop".into());
        looping.steps = vec![Step::new(StepKind::Flow { id: "outer".into() })];
        let loader = move |id: &str| match id {
            "inner" => Ok(inner.clone()),
            "loop" => Ok(looping.clone()),
            _ => Err(Error::Invalid(format!("no flow {id}"))),
        };

        let mut outer = Flow::new("outer".into(), "Outer".into());
        outer.steps = vec![
            Step::new(StepKind::Flow { id: "inner".into() }),
            Step::new(StepKind::Flow { id: "loop".into() }),
        ];
        let (outcome, _) = events(&outer, &loader);
        assert_eq!(outcome.failures.len(), 1);
        assert_eq!(outcome.failures[0].0, 1);
        assert!(outcome.failures[0].1.contains("already running"));

        outer.steps = vec![Step::new(StepKind::Flow {
            id: "missing".into(),
        })];
        let (outcome, _) = events(&outer, &loader);
        assert!(outcome.failures[0].1.contains("no flow missing"));
    }
}
