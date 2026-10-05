//! Runs a flow's steps in order and reports what happened to each one.
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

use super::store::load_flow;
use super::vars::Vars;
use super::{Flow, MAX_DEPTH, OnError, Step, StepKind};

/// Progress of one top-level run. Nested flows report as a single step of
/// their parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunEvent {
    Started {
        index: usize,
    },
    Finished {
        index: usize,
        error: Option<String>,
        /// What the step produced, when it produces anything.
        output: Option<String>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Steps that ran (enabled ones, up to where the flow stopped).
    pub ran: usize,
    pub failures: Vec<(usize, String)>,
    /// The step the flow stopped at, when `OnError::Stop` cut it short.
    pub stopped_at: Option<usize>,
    /// The output of the last step that produced one; what a nested flow
    /// hands back to the step that ran it.
    pub last_output: Option<String>,
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

/// A way to stop a run from another thread: the UI's Stop button sets the
/// flag and the runner ends after the current step, killing a command it
/// is waiting for.
#[derive(Debug, Clone, Default)]
pub struct Cancel {
    stop: Arc<AtomicBool>,
    /// Pid of the child the runner is waiting on, 0 when none.
    child: Arc<AtomicU32>,
}

impl Cancel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Asks the runner to stop and kills the command it is waiting for.
    pub fn cancel(&self) {
        self.stop.store(true, Ordering::SeqCst);
        self.kill_child();
    }

    /// Kills the waited command and everything it started. The shell does
    /// not always exec its command (dash never does, bash not for a list),
    /// and a child left running keeps stderr open, so the wait would last
    /// as long as that child. The waited shell cannot get its own process
    /// group (Omarchy's `exec setsid` launchers would then fork and the
    /// wait would end at once), so its descendants are found by parent pid.
    fn kill_child(&self) {
        let pid = self.child.load(Ordering::SeqCst);
        if pid == 0 {
            return;
        }
        let mut pids = vec![pid.to_string()];
        let mut ix = 0;
        while ix < pids.len() {
            if let Ok(out) = Command::new("pgrep").args(["-P", &pids[ix]]).output() {
                pids.extend(
                    String::from_utf8_lossy(&out.stdout)
                        .split_whitespace()
                        .map(str::to_string),
                );
            }
            ix += 1;
        }
        let _ = Command::new("kill").args(&pids).status();
    }

    pub fn is_cancelled(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }
}

/// How long a command that is not waited for gets to fail before it counts
/// as started: enough for `sh` to report a missing program.
const START_GRACE: Duration = Duration::from_millis(300);

pub struct Runner<'a> {
    load: Loader<'a>,
    /// Discard step output rather than inheriting the caller's stdio.
    quiet: bool,
    cancel: Cancel,
    /// How long a command that is not waited for gets to fail.
    start_grace: Duration,
}

impl<'a> Runner<'a> {
    /// Loads nested flows from disk.
    pub fn new(quiet: bool) -> Runner<'static> {
        Runner {
            load: &load_from_disk,
            quiet,
            cancel: Cancel::new(),
            start_grace: START_GRACE,
        }
    }

    pub fn with_loader(load: Loader<'a>, quiet: bool) -> Self {
        Self {
            load,
            quiet,
            cancel: Cancel::new(),
            start_grace: START_GRACE,
        }
    }

    /// A longer grace for tests on a loaded machine.
    #[cfg(test)]
    fn start_grace(mut self, grace: Duration) -> Self {
        self.start_grace = grace;
        self
    }

    /// Lets another thread stop the run through `cancel`.
    pub fn cancellable(mut self, cancel: Cancel) -> Self {
        self.cancel = cancel;
        self
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
        // Each flow, nested ones included, has its own variables.
        let mut vars = Vars::new();
        for (index, step) in flow.steps.iter().enumerate() {
            if !step.enabled {
                continue;
            }
            if self.cancel.is_cancelled() {
                outcome.stopped_at = Some(index);
                break;
            }
            on_event(RunEvent::Started { index });
            let result = self
                .run_step(step, &mut vars, stack)
                .map_err(|e| e.to_string());
            outcome.ran += 1;
            let output = result.as_ref().ok().cloned().flatten();
            if let Some(output) = &output {
                if let Some(name) = &step.output {
                    vars.set(name, output);
                }
                outcome.last_output = Some(output.clone());
            }
            on_event(RunEvent::Finished {
                index,
                error: result.as_ref().err().cloned(),
                output,
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

    /// Runs one step with its references filled in, and returns its
    /// output if it produces one.
    fn run_step(
        &self,
        step: &Step,
        vars: &mut Vars,
        stack: &mut Vec<String>,
    ) -> Result<Option<String>> {
        match &step.kind {
            StepKind::Exec { command, wait } => {
                let (command, env) = vars.shell(command)?;
                self.exec(&command, &env, *wait)
            }
            StepKind::Lua { expr } => dispatch(&vars.lua(expr)?).map(|()| None),
            StepKind::Wait { ms } => self.wait(Duration::from_millis(*ms)).map(|()| None),
            StepKind::Notify { title, body } => {
                notify(&vars.text(title)?, &vars.text(body)?).map(|()| None)
            }
            StepKind::Flow { id } => self.run_flow_step(id, stack),
        }
    }

    /// Sleeps in slices so a Stop does not wait out a long pause.
    fn wait(&self, total: Duration) -> Result<()> {
        let slice = Duration::from_millis(100);
        let deadline = Instant::now() + total;
        while Instant::now() < deadline {
            if self.cancel.is_cancelled() {
                return Err(Error::Invalid("Stopped".to_string()));
            }
            std::thread::sleep(slice.min(deadline.saturating_duration_since(Instant::now())));
        }
        Ok(())
    }

    /// A waited step runs `sh -c command` as a plain child: Omarchy's launch
    /// scripts end in `exec setsid ...`, and `setsid` only forks when the
    /// caller already leads a process group, which this child does not, so
    /// the wait lasts until the launched window closes, which is what
    /// "wait until it finishes" means. It reports the exit status and the
    /// last lines of stderr.
    ///
    /// A step that is not waited for runs under `setsid` (no `-f`: it execs
    /// in place) so it outlives the flow and Omarchist, but still gets a
    /// moment to fail: `sh` reporting a missing program exits at once, and
    /// that must not show as a green tick.
    /// A waited command's stdout is its output, also printed when the run
    /// is not quiet (the command line). `env` holds the values its
    /// `{{references}}` expand to.
    fn exec(&self, command: &str, env: &[(String, String)], wait: bool) -> Result<Option<String>> {
        let mut cmd = if wait {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command]);
            cmd
        } else {
            let mut cmd = Command::new("setsid");
            cmd.args(["sh", "-c", command]);
            cmd
        };
        cmd.stdin(Stdio::null());
        cmd.envs(env.iter().map(|(k, v)| (k, v)));
        cmd.stdout(if wait { Stdio::piped() } else { Stdio::null() });
        cmd.stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| Error::io("Could not start the command", e))?;

        if wait {
            self.cancel.child.store(child.id(), Ordering::SeqCst);
            // A Stop that came before the pid was known killed nothing.
            if self.cancel.is_cancelled() {
                self.cancel.kill_child();
            }
            let output = child
                .wait_with_output()
                .map_err(|e| Error::io("Could not wait for the command", e));
            self.cancel.child.store(0, Ordering::SeqCst);
            let output = output?;
            if self.cancel.is_cancelled() {
                return Err(Error::Invalid("Stopped".to_string()));
            }
            exit_result(output.status, &output.stderr)?;
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            if !self.quiet && !stdout.is_empty() {
                print!("{stdout}");
                if !stdout.ends_with('\n') {
                    println!();
                }
            }
            return Ok(Some(stdout));
        }

        let deadline = Instant::now() + self.start_grace;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let mut stderr = Vec::new();
                    if let Some(mut pipe) = child.stderr.take() {
                        let _ = pipe.read_to_end(&mut stderr);
                    }
                    return if status.success() {
                        Ok(None)
                    } else {
                        exit_result(status, &stderr).map(|()| None)
                    };
                }
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Ok(None) => break,
                Err(e) => return Err(Error::io("Could not wait for the command", e)),
            }
        }
        // Started: reap it from a thread when it eventually exits, so it
        // never lingers as a zombie, and drain stderr so it never blocks.
        std::thread::spawn(move || {
            if let Some(mut pipe) = child.stderr.take() {
                let _ = std::io::copy(&mut pipe, &mut std::io::sink());
            }
            let _ = child.wait();
        });
        Ok(None)
    }

    /// Runs a nested flow; its output is the output of its last step that
    /// produced one.
    fn run_flow_step(&self, id: &str, stack: &mut Vec<String>) -> Result<Option<String>> {
        if stack.iter().any(|s| s == id) {
            return Err(Error::Invalid(format!(
                "Flow '{id}' is already running further up this flow"
            )));
        }
        if stack.len() >= MAX_DEPTH {
            return Err(Error::Invalid(format!(
                "Flows are nested more than {MAX_DEPTH} deep"
            )));
        }
        let nested = (self.load)(id)?;
        stack.push(nested.id.clone());
        let outcome = self.run_nested(&nested, stack, &mut |_| {});
        stack.pop();
        if outcome.is_ok() {
            Ok(outcome.last_output)
        } else {
            Err(Error::Invalid(outcome.summary(&nested)))
        }
    }
}

/// The step's error for a non-zero exit, with what the command said.
fn exit_result(status: std::process::ExitStatus, stderr: &[u8]) -> Result<()> {
    if status.success() {
        return Ok(());
    }
    let mut message = match status.code() {
        Some(code) => format!("The command exited with status {code}"),
        None => "The command was killed by a signal".to_string(),
    };
    let stderr = String::from_utf8_lossy(stderr);
    let tail: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let tail = &tail[tail.len().saturating_sub(3)..];
    if !tail.is_empty() {
        message.push_str(": ");
        message.push_str(&tail.join(" · "));
    }
    Err(Error::Invalid(message))
}

/// Validates the flow, then runs it on a thread of its own (a run blocks
/// for as long as the flow takes) and resolves with the outcome. For runs
/// that need no per-step progress: cards, the title-bar menu.
pub async fn run_in_thread(flow: Flow) -> Result<Outcome> {
    flow.validate()?;
    let (tx, rx) = smol::channel::bounded(1);
    std::thread::spawn(move || {
        let outcome = Runner::new(true).run(&flow, &mut |_| {});
        let _ = tx.send_blocking(outcome);
    });
    rx.recv()
        .await
        .map_err(|_| Error::Invalid("The run ended without a result".to_string()))
}

fn load_from_disk(id: &str) -> Result<Flow> {
    load_flow(id)
}

fn dispatch(expr: &str) -> Result<()> {
    let output = Command::new("hyprctl")
        .args(["dispatch", expr])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error::io("Could not run hyprctl", e))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if output.status.success() && text == "ok" {
        Ok(())
    } else if text.is_empty() {
        Err(Error::Invalid(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    } else {
        Err(Error::Invalid(text.to_string()))
    }
}

fn notify(title: &str, body: &str) -> Result<()> {
    let mut cmd = Command::new("notify-send");
    // `--` keeps a title such as "-t 5 minutes" from being read as options.
    cmd.args(["-a", "Omarchist", "--", title.trim()]);
    if !body.trim().is_empty() {
        cmd.arg(body.trim());
    }
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| Error::io("Could not send the notification", e))?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Invalid("notify-send failed".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::system::flows::Step;

    /// A waited-for command, so its exit status reaches the outcome.
    fn exec(command: &str) -> Step {
        Step::new(StepKind::Exec {
            command: command.into(),
            wait: true,
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
                error: Some("The command exited with status 3".into()),
                output: None,
            }
        );
        assert!(outcome.summary(&flow).contains("stopped at step 2"));
    }

    fn saving(step: Step, name: &str) -> Step {
        Step {
            output: Some(name.into()),
            ..step
        }
    }

    #[test]
    fn a_saved_output_reaches_later_steps_as_plain_text() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(exec("printf '%s' 'a b; $(false)'"), "text"),
            saving(exec("printf '%s' {{text}} | wc -c"), "length"),
        ];
        let (outcome, seen) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        // The value arrived intact (13 bytes), never run as a command.
        assert_eq!(outcome.last_output.as_deref().map(str::trim), Some("13"));
        assert!(seen.contains(&RunEvent::Finished {
            index: 0,
            error: None,
            output: Some("a b; $(false)".into()),
        }));
    }

    #[test]
    fn a_nested_flow_hands_back_its_last_output() {
        let mut child = Flow::new("child".into(), "Child".into());
        child.steps = vec![exec("echo from-child")];
        let load = move |id: &str| -> Result<Flow> {
            if id == "child" {
                Ok(child.clone())
            } else {
                Err(Error::Invalid(format!("no flow {id}")))
            }
        };
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(Step::new(StepKind::Flow { id: "child".into() }), "got"),
            exec("test {{got}} = from-child"),
        ];
        let outcome = Runner::with_loader(&load, true).run(&flow, &mut |_| {});
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
    }

    #[test]
    fn a_name_whose_step_did_not_run_fails_the_step() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.on_error = OnError::Continue;
        flow.steps = vec![
            Step {
                enabled: false,
                ..saving(exec("echo x"), "x")
            },
            exec("echo {{x}}"),
        ];
        let (outcome, _) = events(&flow, &no_flows);
        assert_eq!(outcome.failures.len(), 1);
        assert!(
            outcome.failures[0].1.contains("{{x}}"),
            "{}",
            outcome.failures[0].1
        );
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
    fn a_command_that_is_not_waited_for_still_fails_when_it_cannot_start() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            Step::new(StepKind::Exec {
                command: "omarchy-launch-browserr".into(),
                wait: false,
            }),
            Step::new(StepKind::Exec {
                command: "sleep 2".into(),
                wait: false,
            }),
        ];
        flow.on_error = OnError::Continue;
        let started = Instant::now();
        // The whole test suite runs in parallel; give `sh` time to report.
        let runner = Runner::with_loader(&no_flows, true).start_grace(Duration::from_secs(1));
        let outcome = runner.run(&flow, &mut |_| {});
        assert_eq!(outcome.failures.len(), 1, "{:?}", outcome.failures);
        assert_eq!(outcome.failures[0].0, 0);
        assert!(
            outcome.failures[0].1.contains("not found"),
            "{}",
            outcome.failures[0].1
        );
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "a started command is not waited for"
        );
    }

    #[test]
    fn a_waited_command_reports_its_stderr() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![exec("echo oops >&2; exit 4")];
        let (outcome, _) = events(&flow, &no_flows);
        assert_eq!(
            outcome.failures[0].1,
            "The command exited with status 4: oops"
        );
    }

    #[test]
    fn cancel_stops_a_wait_and_kills_a_waited_command() {
        let mut flow = Flow::new("t".into(), "T".into());
        // Two commands, so even bash forks `sleep` as a child of the shell (dash
        // always does): Stop must reach it, not only the shell.
        flow.steps = vec![exec("sleep 8; true"), exec("true")];
        let cancel = Cancel::new();
        let stopper = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            stopper.cancel();
        });
        let started = Instant::now();
        let runner = Runner::with_loader(&no_flows, true).cancellable(cancel);
        let outcome = runner.run(&flow, &mut |_| {});
        assert!(started.elapsed() < Duration::from_secs(4));
        assert_eq!(outcome.ran, 1);
        assert_eq!(outcome.stopped_at, Some(0));
        assert!(outcome.failures[0].1.contains("Stopped"));

        let mut flow = Flow::new("w".into(), "W".into());
        flow.steps = vec![Step::new(StepKind::Wait { ms: 30_000 })];
        let cancel = Cancel::new();
        cancel.cancel();
        let outcome = Runner::with_loader(&no_flows, true)
            .cancellable(cancel)
            .run(&flow, &mut |_| {});
        assert_eq!(outcome.ran, 0);
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
