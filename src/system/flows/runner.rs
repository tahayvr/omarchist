//! Runs a flow's steps in order and reports what happened to each one.
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

use std::collections::BTreeMap;

use super::actions;
use super::condition::{Machine, Probe};
use super::history::{self, Recorder};
use super::prompt::{Desktop, Prompter, menu_options};
use super::store::load_flow;
use super::vars::Vars;
use super::{
    Flow, InputFallback, MAX_DEPTH, MAX_ROUNDS, OnClick, OnError, Step, StepKind, StepPath,
};

/// How one step ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    /// With what the step produced, when it produces anything.
    Done(Option<String>),
    Failed(String),
    /// The person dismissed a prompt, which ends the flow without an error.
    Cancelled,
}

/// Progress of one top-level run. Nested flows report as a single step of
/// their parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunEvent {
    Started {
        path: StepPath,
    },
    Finished {
        path: StepPath,
        status: StepStatus,
    },
    /// A loop step starts round `round` of `of`; the steps inside it
    /// report again for every round.
    Round {
        path: StepPath,
        round: u32,
        of: u32,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Steps that ran (enabled ones, up to where the flow stopped).
    pub ran: usize,
    pub failures: Vec<(StepPath, String)>,
    /// The step the flow stopped at, when `OnError::Stop` cut it short.
    pub stopped_at: Option<StepPath>,
    /// A prompt was dismissed, so the flow ended early on purpose.
    pub cancelled: bool,
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
        match (self.failures.as_slice(), &self.stopped_at) {
            ([], _) if self.cancelled => format!("Flow '{}' cancelled", flow.name),
            ([], _) => format!("Flow '{}' finished", flow.name),
            ([(path, error)], Some(_)) if path.is_empty() => {
                format!("Flow '{}' could not start: {error}", flow.name)
            }
            ([(path, error)], Some(_)) => {
                format!(
                    "Flow '{}' stopped at step {}: {error}",
                    flow.name,
                    flow.step_number(path)
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

    /// Names the child a Stop should kill. A Stop that came before the pid
    /// was known killed nothing, so it is applied now.
    pub(crate) fn watch(&self, pid: u32) {
        self.child.store(pid, Ordering::SeqCst);
        if self.is_cancelled() {
            self.kill_child();
        }
    }

    pub(crate) fn unwatch(&self) {
        self.child.store(0, Ordering::SeqCst);
    }
}

/// Why a step did not finish.
enum Halt {
    Failed(String),
    /// A prompt was dismissed.
    Cancelled,
    /// A `stop` step: the flow ends here, as finished.
    Stop,
}

/// How a step that did finish left the flow.
enum StepEnd {
    /// With its output, when it produces one.
    Done(Option<String>),
    /// A step inside it ended the flow; the outcome already says why.
    Ended,
}

/// Whether the steps after a list should still run.
#[derive(PartialEq, Eq)]
enum Flowing {
    On,
    Ended,
}

/// What one flow's run carries from step to step.
struct Run<'r> {
    /// Each flow, nested ones included, has its own variables.
    vars: Vars,
    stack: &'r mut Vec<String>,
    on_event: &'r mut dyn FnMut(RunEvent),
    outcome: Outcome,
    on_error: OnError,
}

impl From<Error> for Halt {
    fn from(error: Error) -> Self {
        Halt::Failed(error.to_string())
    }
}

/// What a step that was cut short from outside fails with.
pub(crate) const STOPPED: &str = "Stopped";

/// The choices a `confirm` step offers.
const CONTINUE: &str = "Continue";
const CANCEL: &str = "Cancel";

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
    /// What the flow is started with: its `{{input}}`.
    input: Option<String>,
    /// Answers the steps that ask.
    prompter: &'a dyn Prompter,
    /// What `if` steps check the machine through.
    probe: &'a dyn Probe,
}

impl<'a> Runner<'a> {
    /// Loads nested flows from disk.
    pub fn new(quiet: bool) -> Runner<'static> {
        Runner {
            load: &load_from_disk,
            quiet,
            cancel: Cancel::new(),
            start_grace: START_GRACE,
            input: None,
            prompter: &Desktop,
            probe: &Machine,
        }
    }

    pub fn with_loader(load: Loader<'a>, quiet: bool) -> Self {
        Self {
            load,
            quiet,
            cancel: Cancel::new(),
            start_grace: START_GRACE,
            input: None,
            prompter: &Desktop,
            probe: &Machine,
        }
    }

    /// Answers the steps that ask through `prompter` instead of the
    /// desktop's menus.
    pub fn prompter(mut self, prompter: &'a dyn Prompter) -> Self {
        self.prompter = prompter;
        self
    }

    /// Starts the flow with `input` as its `{{input}}`: arguments, piped
    /// text, or the files picked in the file manager, one per line.
    pub fn input(mut self, input: Option<String>) -> Self {
        self.input = input.filter(|text| !text.trim().is_empty());
        self
    }

    /// Lets `if` steps check the machine through `probe`.
    pub fn probe(mut self, probe: &'a dyn Probe) -> Self {
        self.probe = probe;
        self
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
        self.run_nested(flow, self.input.clone(), &mut stack, on_event)
    }

    fn run_nested(
        &self,
        flow: &Flow,
        input: Option<String>,
        stack: &mut Vec<String>,
        on_event: &mut dyn FnMut(RunEvent),
    ) -> Outcome {
        let mut vars = Vars::new();
        // Started with nothing, the flow says where its input comes from.
        let input = match input.filter(|text| !text.trim().is_empty()) {
            Some(input) => input,
            None => match flow.input {
                InputFallback::None => String::new(),
                InputFallback::Selection => vars.get("selection").unwrap_or_default(),
                InputFallback::Clipboard => vars.get("clipboard").unwrap_or_default(),
                InputFallback::Ask => match self.prompter.ask(&flow.name, &self.cancel) {
                    Ok(Some(answer)) => answer,
                    Ok(None) => {
                        return Outcome {
                            cancelled: true,
                            ..Outcome::default()
                        };
                    }
                    Err(e) => {
                        return Outcome {
                            failures: vec![(Vec::new(), e.to_string())],
                            stopped_at: Some(Vec::new()),
                            ..Outcome::default()
                        };
                    }
                },
            },
        };
        vars.set("input", &input);
        let mut run = Run {
            vars,
            stack,
            on_event,
            outcome: Outcome::default(),
            on_error: flow.on_error,
        };
        self.run_list(&flow.steps, &[], &mut run);
        run.outcome
    }

    /// Runs only the step at `path`, with the steps it holds, to try it
    /// out. The steps before it do not run, so `values` gives the
    /// variables it uses what they would hold. A step that is switched off
    /// runs all the same: trying it is how one decides to switch it on.
    pub fn run_only(
        &self,
        flow: &Flow,
        path: &[usize],
        values: &[(String, String)],
        on_event: &mut dyn FnMut(RunEvent),
    ) -> Outcome {
        let Some(step) = flow.step_at(path) else {
            return Outcome::default();
        };
        let mut vars = Vars::new();
        vars.set("input", self.input.as_deref().unwrap_or_default());
        for (name, value) in values {
            vars.set(name, value);
        }
        let mut stack = vec![flow.id.clone()];
        let mut run = Run {
            vars,
            stack: &mut stack,
            on_event,
            outcome: Outcome::default(),
            on_error: flow.on_error,
        };
        self.run_at(step, path.to_vec(), &mut run);
        run.outcome
    }

    /// Runs the steps of one list in order: the flow's own, or a branch
    /// of a step that holds steps (`base` is that branch's path).
    fn run_list(&self, steps: &[Step], base: &[usize], run: &mut Run) -> Flowing {
        for (index, step) in steps.iter().enumerate() {
            if !step.enabled {
                continue;
            }
            let mut path: StepPath = base.to_vec();
            path.push(index);
            if self.run_at(step, path, run) == Flowing::Ended {
                return Flowing::Ended;
            }
        }
        Flowing::On
    }

    /// Runs the step at `path`, reports it, and keeps what it produced.
    fn run_at(&self, step: &Step, path: StepPath, run: &mut Run) -> Flowing {
        if self.cancel.is_cancelled() {
            run.outcome.stopped_at = Some(path);
            return Flowing::Ended;
        }
        (run.on_event)(RunEvent::Started { path: path.clone() });
        let result = self.run_step(step, &path, run);
        run.outcome.ran += 1;
        let (status, flowing) = match result {
            Ok(StepEnd::Done(output)) => {
                if let Some(output) = &output {
                    if let Some(name) = &step.output {
                        run.vars.set(name, output);
                    }
                    run.outcome.last_output = Some(output.clone());
                }
                (StepStatus::Done(output), Flowing::On)
            }
            Ok(StepEnd::Ended) | Err(Halt::Stop) => (StepStatus::Done(None), Flowing::Ended),
            Err(Halt::Cancelled) => {
                run.outcome.cancelled = true;
                run.outcome.stopped_at = Some(path.clone());
                (StepStatus::Cancelled, Flowing::Ended)
            }
            Err(Halt::Failed(error)) => {
                run.outcome.failures.push((path.clone(), error.clone()));
                let flowing = if run.on_error == OnError::Stop {
                    run.outcome.stopped_at = Some(path.clone());
                    Flowing::Ended
                } else {
                    Flowing::On
                };
                (StepStatus::Failed(error), flowing)
            }
        };
        (run.on_event)(RunEvent::Finished { path, status });
        flowing
    }

    /// Runs the steps of branch `branch` of the step at `path`.
    fn run_branch(&self, steps: &[Step], path: &[usize], branch: usize, run: &mut Run) -> Flowing {
        let mut list = path.to_vec();
        list.push(branch);
        self.run_list(steps, &list, run)
    }

    /// Runs `steps` once per round of a loop step, with `{{index}}` (and
    /// `{{item}}` when there are items) set for them, and puts back what
    /// those names held around the loop.
    fn run_rounds(
        &self,
        steps: &[Step],
        path: &[usize],
        items: Option<&[String]>,
        rounds: u32,
        run: &mut Run,
    ) -> StepEnd {
        let before = (run.vars.peek("index"), run.vars.peek("item"));
        let mut end = StepEnd::Done(None);
        for round in 1..=rounds {
            if self.cancel.is_cancelled() {
                run.outcome.stopped_at = Some(path.to_vec());
                end = StepEnd::Ended;
                break;
            }
            (run.on_event)(RunEvent::Round {
                path: path.to_vec(),
                round,
                of: rounds,
            });
            run.vars.set("index", &round.to_string());
            if let Some(item) = items.and_then(|items| items.get(round as usize - 1)) {
                run.vars.set("item", item);
            }
            if self.run_branch(steps, path, 0, run) == Flowing::Ended {
                end = StepEnd::Ended;
                break;
            }
        }
        run.vars.restore("index", before.0);
        if items.is_some() {
            run.vars.restore("item", before.1);
        }
        end
    }

    /// Runs one step with its references filled in, and returns its
    /// output if it produces one.
    fn run_step(
        &self,
        step: &Step,
        path: &[usize],
        run: &mut Run,
    ) -> std::result::Result<StepEnd, Halt> {
        let vars = &mut run.vars;
        let output = match &step.kind {
            StepKind::Exec { command, wait } => {
                let (command, env) = vars.shell(command)?;
                self.exec(&command, &env, *wait)?
            }
            StepKind::Lua { expr } => {
                dispatch(&vars.lua(expr)?)?;
                None
            }
            StepKind::Wait { ms } => {
                self.wait(Duration::from_millis(*ms))?;
                None
            }
            StepKind::Notify {
                title,
                body,
                on_click,
                target,
            } => {
                let title = vars.text(title)?;
                let body = vars.text(body)?;
                let click = match on_click {
                    Some(action) => {
                        let target = vars.text(target)?;
                        // Without a target the notification acts on what
                        // it shows.
                        let target = [target, body.clone(), title.clone()]
                            .into_iter()
                            .find(|t| !t.trim().is_empty())
                            .unwrap_or_default();
                        Some((*action, target.trim().to_string()))
                    }
                    None => None,
                };
                notify(&title, &body, click)?;
                None
            }
            StepKind::Flow { id, input } => {
                let input = vars.text(input)?;
                self.run_flow_step(id, input, run.stack)?
            }
            StepKind::Ask { prompt } => {
                let prompt = vars.text(prompt)?;
                Some(
                    self.prompter
                        .ask(&prompt, &self.cancel)?
                        .ok_or(Halt::Cancelled)?,
                )
            }
            StepKind::Choose {
                prompt,
                options,
                from,
            } => {
                let prompt = vars.text(prompt)?;
                let options = if options.iter().any(|o| !o.trim().is_empty()) {
                    options
                        .iter()
                        .map(|o| vars.text(o))
                        .collect::<Result<Vec<String>>>()?
                } else {
                    vars.text(from)?.lines().map(str::to_string).collect()
                };
                let options = menu_options(&options);
                if options.is_empty() {
                    return Err(Halt::Failed("There is nothing to choose from".to_string()));
                }
                Some(
                    self.prompter
                        .choose(&prompt, &options, &self.cancel)?
                        .ok_or(Halt::Cancelled)?,
                )
            }
            StepKind::Confirm { prompt } => {
                let prompt = vars.text(prompt)?;
                let options = [CONTINUE.to_string(), CANCEL.to_string()];
                match self.prompter.choose(&prompt, &options, &self.cancel)? {
                    Some(answer) if answer == CONTINUE => None,
                    _ => return Err(Halt::Cancelled),
                }
            }
            StepKind::Pick { prompt, folder } => {
                let prompt = vars.text(prompt)?;
                Some(
                    self.prompter
                        .pick(&prompt, *folder)?
                        .ok_or(Halt::Cancelled)?,
                )
            }
            StepKind::Stop => return Err(Halt::Stop),
            StepKind::Action { action, args } => {
                let Some(def) = actions::find(action) else {
                    return Err(Halt::Failed(format!(
                        "This Omarchist has no action '{action}'"
                    )));
                };
                // Each field with its variables filled in. Text fields are
                // the only ones that take them.
                let mut values: BTreeMap<&'static str, String> = BTreeMap::new();
                for field in def.fields {
                    values.insert(field.key, vars.text(&def.raw(field, args))?);
                }
                match def.run {
                    actions::Run::Shell(script) => {
                        let env: Vec<(String, String)> = values
                            .iter()
                            .map(|(key, value)| {
                                (format!("ARG_{}", key.to_uppercase()), value.clone())
                            })
                            .collect();
                        let printed = self.exec(script, &env, true)?;
                        printed.filter(|_| def.has_output())
                    }
                    actions::Run::Lua(template) => {
                        dispatch(&actions::lua_call(template, &values)?)?;
                        None
                    }
                    actions::Run::Native(run) => Some(run(&values)),
                }
            }
            StepKind::If {
                condition,
                not,
                then,
                otherwise,
            } => {
                let holds = condition.evaluate(vars, self.probe)? != *not;
                let (branch, steps) = if holds { (0, then) } else { (1, otherwise) };
                return Ok(match self.run_branch(steps, path, branch, run) {
                    Flowing::On => StepEnd::Done(None),
                    Flowing::Ended => StepEnd::Ended,
                });
            }
            StepKind::Repeat { times, steps } => {
                return Ok(self.run_rounds(steps, path, None, (*times).min(MAX_ROUNDS), run));
            }
            StepKind::Each { items, steps } => {
                let items: Vec<String> = vars
                    .text(items)?
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();
                if items.len() > MAX_ROUNDS as usize {
                    return Err(Halt::Failed(format!(
                        "There are {} items; a step repeats at most {MAX_ROUNDS} times",
                        items.len()
                    )));
                }
                let rounds = items.len() as u32;
                return Ok(self.run_rounds(steps, path, Some(&items), rounds, run));
            }
            StepKind::Menu { prompt, choices } => {
                let prompt = vars.text(prompt)?;
                let labels = choices
                    .iter()
                    .map(|c| vars.text(&c.label))
                    .collect::<Result<Vec<String>>>()?;
                // As the menu shows them, which is also how it answers.
                let shown: Vec<String> = labels
                    .iter()
                    .map(|label| menu_options(std::slice::from_ref(label)).join(" "))
                    .collect();
                let pick = self
                    .prompter
                    .choose(&prompt, &shown, &self.cancel)?
                    .ok_or(Halt::Cancelled)?;
                let Some(branch) = shown.iter().position(|label| *label == pick) else {
                    return Err(Halt::Failed(format!("'{pick}' is not one of the choices")));
                };
                // The pick is known to the steps it runs.
                if let Some(name) = &step.output {
                    vars.set(name, &pick);
                }
                run.outcome.last_output = Some(pick.clone());
                return Ok(
                    match self.run_branch(&choices[branch].steps, path, branch, run) {
                        Flowing::On => StepEnd::Done(Some(pick)),
                        Flowing::Ended => StepEnd::Ended,
                    },
                );
            }
        };
        Ok(StepEnd::Done(output))
    }

    /// Sleeps in slices so a Stop does not wait out a long pause.
    fn wait(&self, total: Duration) -> Result<()> {
        let slice = Duration::from_millis(100);
        let deadline = Instant::now() + total;
        while Instant::now() < deadline {
            if self.cancel.is_cancelled() {
                return Err(Error::Invalid(STOPPED.to_string()));
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
            self.cancel.watch(child.id());
            let output = child
                .wait_with_output()
                .map_err(|e| Error::io("Could not wait for the command", e));
            self.cancel.unwatch();
            let output = output?;
            if self.cancel.is_cancelled() {
                return Err(Error::Invalid(STOPPED.to_string()));
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
    /// produced one. A prompt dismissed inside it ends this flow too.
    fn run_flow_step(
        &self,
        id: &str,
        input: String,
        stack: &mut Vec<String>,
    ) -> std::result::Result<Option<String>, Halt> {
        if stack.iter().any(|s| s == id) {
            return Err(Halt::Failed(format!(
                "Flow '{id}' is already running further up this flow"
            )));
        }
        if stack.len() >= MAX_DEPTH {
            return Err(Halt::Failed(format!(
                "Flows are nested more than {MAX_DEPTH} deep"
            )));
        }
        let nested = (self.load)(id)?;
        stack.push(nested.id.clone());
        let outcome = self.run_nested(&nested, Some(input), stack, &mut |_| {});
        stack.pop();
        if outcome.cancelled {
            Err(Halt::Cancelled)
        } else if outcome.is_ok() {
            Ok(outcome.last_output)
        } else {
            Err(Halt::Failed(outcome.summary(&nested)))
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
        let cancel = Cancel::new();
        // Seen by the bar widget and the Flows page, which can stop it.
        let _registered = super::running::register_in_app(&flow, "Omarchist", cancel.clone());
        let mut recorder = Recorder::new(&flow, "Omarchist");
        let outcome = Runner::new(true)
            .cancellable(cancel.clone())
            .run(&flow, &mut |event| recorder.event(&event));
        let run = recorder.finish(&flow, &outcome, cancel.is_cancelled());
        if let Err(e) = history::record(&flow.id, &run) {
            eprintln!("{e}");
        }
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

/// A plain notification goes through `notify-send`. One that does
/// something when clicked goes through Omarchy's own sender, whose
/// `--exec` takes the click command as separate words, so the target is
/// only ever one argument of it.
fn notify(title: &str, body: &str, click: Option<(OnClick, String)>) -> Result<()> {
    let mut cmd = match &click {
        None => {
            let mut cmd = Command::new("notify-send");
            // `--` keeps a title such as "-t 5 minutes" from being read as options.
            cmd.args(["-a", "Omarchist", "--", title.trim()]);
            if !body.trim().is_empty() {
                cmd.arg(body.trim());
            }
            cmd
        }
        Some((action, target)) => {
            let mut cmd = Command::new("omarchy-notification-send");
            cmd.args(["--app-name", "Omarchist", "-u", "normal"]);
            // The script reads a leading dash as one of its options.
            let title = title.trim();
            if title.starts_with('-') {
                cmd.arg(format!("\u{200b}{title}"));
            } else {
                cmd.arg(title);
            }
            if !body.trim().is_empty() {
                cmd.arg(body.trim());
            }
            cmd.arg("--exec");
            match action {
                OnClick::Copy => cmd.args(["wl-copy", "--", target]),
                OnClick::Open => cmd.args(["xdg-open", &expand_home(target)]),
            };
            cmd
        }
    };
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| Error::io("Could not send the notification", e))?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Invalid(
            "The notification could not be sent".to_string(),
        ))
    }
}

/// `~/notes` as a full path; `xdg-open` gets no shell to expand it.
fn expand_home(target: &str) -> String {
    match (target.strip_prefix("~/"), dirs::home_dir()) {
        (Some(rest), Some(home)) => home.join(rest).display().to_string(),
        _ => target.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::system::flows::prompt::Prompter;
    use crate::system::flows::{Step, StepPath};

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
        assert_eq!(outcome.stopped_at, Some(vec![1]));
        assert_eq!(outcome.failures.len(), 1);
        assert!(outcome.failures[0].1.contains("status 3"));
        assert_eq!(seen.len(), 4);
        assert_eq!(
            seen[3],
            RunEvent::Finished {
                path: vec![1],
                status: StepStatus::Failed("The command exited with status 3".into()),
            }
        );
        assert!(outcome.summary(&flow).contains("stopped at step 2"));
    }

    fn saving(step: Step, name: &str) -> Step {
        step.saving(name)
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
            path: vec![0],
            status: StepStatus::Done(Some("a b; $(false)".into())),
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
            saving(Step::new(StepKind::flow("child")), "got"),
            exec("test {{got}} = from-child"),
        ];
        let outcome = Runner::with_loader(&load, true).run(&flow, &mut |_| {});
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
    }

    #[test]
    fn a_name_whose_step_did_not_run_fails_the_step() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.on_error = OnError::Continue;
        flow.steps = vec![saving(exec("echo x"), "x").off(), exec("echo {{x}}")];
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
            exec("exit 9").off(),
            Step::new(StepKind::Wait { ms: 1 }),
            exec("true"),
        ];
        let (outcome, seen) = events(&flow, &no_flows);
        assert_eq!(outcome.ran, 3);
        assert_eq!(outcome.stopped_at, None);
        assert_eq!(outcome.failures.len(), 1);
        assert!(
            seen.iter()
                .all(|e| *e != RunEvent::Started { path: vec![1] })
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
                command: "sleep 30".into(),
                wait: false,
            }),
        ];
        flow.on_error = OnError::Continue;
        let started = Instant::now();
        // The whole test suite runs in parallel, on a busy machine; give
        // `sh` time to report.
        let runner = Runner::with_loader(&no_flows, true).start_grace(Duration::from_secs(2));
        let outcome = runner.run(&flow, &mut |_| {});
        assert_eq!(outcome.failures.len(), 1, "{:?}", outcome.failures);
        assert_eq!(outcome.failures[0].0, vec![0]);
        assert!(
            outcome.failures[0].1.contains("not found"),
            "{}",
            outcome.failures[0].1
        );
        assert!(
            started.elapsed() < Duration::from_secs(15),
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
        assert_eq!(outcome.stopped_at, Some(vec![0]));
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

    /// Answers from a script instead of the desktop: `None` dismisses.
    struct Scripted {
        answers: std::sync::Mutex<Vec<Option<String>>>,
        asked: std::sync::Mutex<Vec<String>>,
    }

    impl Scripted {
        fn new(answers: &[Option<&str>]) -> Self {
            Self {
                answers: std::sync::Mutex::new(
                    answers
                        .iter()
                        .rev()
                        .map(|a| a.map(str::to_string))
                        .collect(),
                ),
                asked: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn next(&self, what: String) -> Result<Option<String>> {
            self.asked.lock().unwrap().push(what);
            Ok(self.answers.lock().unwrap().pop().flatten())
        }

        fn asked(&self) -> Vec<String> {
            self.asked.lock().unwrap().clone()
        }
    }

    impl Prompter for Scripted {
        fn ask(&self, prompt: &str, _: &Cancel) -> Result<Option<String>> {
            self.next(format!("ask {prompt}"))
        }

        fn choose(&self, prompt: &str, options: &[String], _: &Cancel) -> Result<Option<String>> {
            self.next(format!("choose {prompt}: {}", options.join("|")))
        }

        fn pick(&self, prompt: &str, folder: bool) -> Result<Option<String>> {
            self.next(format!("pick {prompt} folder={folder}"))
        }
    }

    #[test]
    fn answers_become_outputs_for_later_steps() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(
                Step::new(StepKind::Ask {
                    prompt: "Name?".into(),
                }),
                "name",
            ),
            saving(
                Step::new(StepKind::Choose {
                    prompt: "Size for {{name}}".into(),
                    options: vec!["Small".into(), "Large".into()],
                    from: String::new(),
                }),
                "size",
            ),
            saving(exec("printf 'a\\nb\\n'"), "lines"),
            saving(
                Step::new(StepKind::Choose {
                    prompt: "Line".into(),
                    options: Vec::new(),
                    from: "{{lines}}".into(),
                }),
                "line",
            ),
            saving(
                Step::new(StepKind::Pick {
                    prompt: String::new(),
                    folder: true,
                }),
                "dir",
            ),
            Step::new(StepKind::Confirm {
                prompt: "Go?".into(),
            }),
            exec("test \"{{name}} {{size}} {{line}} {{dir}}\" = 'Ada Large b /tmp'"),
        ];
        let prompts = Scripted::new(&[
            Some("Ada"),
            Some("Large"),
            Some("b"),
            Some("/tmp"),
            Some("Continue"),
        ]);
        let outcome = Runner::with_loader(&no_flows, true)
            .prompter(&prompts)
            .run(&flow, &mut |_| {});
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert!(!outcome.cancelled);
        assert_eq!(outcome.ran, 7);
        assert_eq!(
            prompts.asked(),
            vec![
                "ask Name?",
                "choose Size for Ada: Small|Large",
                "choose Line: a|b",
                "pick  folder=true",
                "choose Go?: Continue|Cancel",
            ]
        );
    }

    #[test]
    fn dismissing_a_prompt_ends_the_flow_without_a_failure() {
        for cancelled_step in [
            Step::new(StepKind::Ask {
                prompt: "Name?".into(),
            }),
            Step::new(StepKind::Confirm {
                prompt: "Go?".into(),
            }),
        ] {
            let mut flow = Flow::new("t".into(), "T".into());
            flow.steps = vec![exec("true"), cancelled_step, exec("exit 7")];
            let prompts = Scripted::new(&[None]);
            let mut seen = Vec::new();
            let outcome = Runner::with_loader(&no_flows, true)
                .prompter(&prompts)
                .run(&flow, &mut |e| seen.push(e));
            assert!(outcome.is_ok(), "a dismissed prompt is not a failure");
            assert!(outcome.cancelled);
            assert_eq!(outcome.ran, 2, "the step after the prompt never ran");
            assert_eq!(
                seen.last(),
                Some(&RunEvent::Finished {
                    path: vec![1],
                    status: StepStatus::Cancelled,
                })
            );
            assert_eq!(outcome.summary(&flow), "Flow 'T' cancelled");
        }
    }

    #[test]
    fn a_confirm_answered_cancel_ends_the_flow() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            Step::new(StepKind::Confirm {
                prompt: "Go?".into(),
            }),
            exec("exit 7"),
        ];
        let prompts = Scripted::new(&[Some("Cancel")]);
        let outcome = Runner::with_loader(&no_flows, true)
            .prompter(&prompts)
            .run(&flow, &mut |_| {});
        assert!(outcome.cancelled && outcome.is_ok());
    }

    #[test]
    fn a_prompt_dismissed_in_a_nested_flow_ends_the_parent_too() {
        let mut child = Flow::new("child".into(), "Child".into());
        child.steps = vec![Step::new(StepKind::Ask {
            prompt: "Name?".into(),
        })];
        let load = move |id: &str| -> Result<Flow> {
            if id == "child" {
                Ok(child.clone())
            } else {
                Err(Error::Invalid(format!("no flow {id}")))
            }
        };
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![Step::new(StepKind::flow("child")), exec("exit 7")];
        let prompts = Scripted::new(&[None]);
        let outcome = Runner::with_loader(&load, true)
            .prompter(&prompts)
            .run(&flow, &mut |_| {});
        assert!(outcome.cancelled && outcome.is_ok());
        assert_eq!(outcome.ran, 1);
    }

    #[test]
    fn choosing_from_an_empty_list_fails_the_step() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(exec("true"), "nothing"),
            Step::new(StepKind::Choose {
                prompt: "Pick".into(),
                options: Vec::new(),
                from: "{{nothing}}".into(),
            }),
        ];
        let prompts = Scripted::new(&[]);
        let outcome = Runner::with_loader(&no_flows, true)
            .prompter(&prompts)
            .run(&flow, &mut |_| {});
        assert_eq!(outcome.failures.len(), 1);
        assert!(outcome.failures[0].1.contains("nothing to choose"));
        assert!(prompts.asked().is_empty());
    }

    // MARK: Steps that hold steps

    use crate::system::flows::MenuChoice;
    use crate::system::flows::condition::Condition;
    use crate::system::flows::condition::tests::Fake;

    fn when(condition: Condition, then: Vec<Step>, otherwise: Vec<Step>) -> Step {
        Step::new(StepKind::If {
            condition,
            not: false,
            then,
            otherwise,
        })
    }

    fn repeat(times: u32, steps: Vec<Step>) -> Step {
        Step::new(StepKind::Repeat { times, steps })
    }

    fn each(items: &str, steps: Vec<Step>) -> Step {
        Step::new(StepKind::Each {
            items: items.into(),
            steps,
        })
    }

    /// A file a flow's commands can append to, removed when dropped.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("omarchist-runner-{}-{name}", std::process::id()));
            let _ = std::fs::remove_file(&path);
            Self(path)
        }

        /// A step that appends `text` (with its variables filled in).
        fn append(&self, text: &str) -> Step {
            exec(&format!("echo {text} >> {}", self.0.display()))
        }

        fn lines(&self) -> Vec<String> {
            std::fs::read_to_string(&self.0)
                .unwrap_or_default()
                .lines()
                .map(str::to_string)
                .collect()
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn started(seen: &[RunEvent]) -> Vec<StepPath> {
        seen.iter()
            .filter_map(|e| match e {
                RunEvent::Started { path } => Some(path.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn an_if_runs_the_branch_its_check_picks() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            when(
                Condition::OnBattery,
                vec![exec("true")],
                vec![exec("exit 9")],
            ),
            exec("true"),
        ];
        let on_battery = Fake {
            battery: true,
            ..Fake::default()
        };
        let mut seen = Vec::new();
        let outcome = Runner::with_loader(&no_flows, true)
            .probe(&on_battery)
            .run(&flow, &mut |e| seen.push(e));
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(started(&seen), vec![vec![0], vec![0, 0, 0], vec![1]]);
        assert_eq!(outcome.ran, 3);

        // Plugged in, the other branch runs, and its failure is the flow's.
        let plugged_in = Fake::default();
        let mut seen = Vec::new();
        let outcome = Runner::with_loader(&no_flows, true)
            .probe(&plugged_in)
            .run(&flow, &mut |e| seen.push(e));
        assert_eq!(started(&seen), vec![vec![0], vec![0, 1, 0]]);
        assert_eq!(outcome.failures.len(), 1);
        assert_eq!(outcome.failures[0].0, vec![0, 1, 0]);
        assert_eq!(outcome.stopped_at, Some(vec![0, 1, 0]));
        assert!(
            outcome.summary(&flow).contains("stopped at step 3"),
            "{}",
            outcome.summary(&flow)
        );
    }

    #[test]
    fn a_negated_if_takes_the_other_branch() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![Step::new(StepKind::If {
            condition: Condition::OnBattery,
            not: true,
            then: vec![exec("true")],
            otherwise: vec![exec("exit 9")],
        })];
        let plugged_in = Fake::default();
        let outcome = Runner::with_loader(&no_flows, true)
            .probe(&plugged_in)
            .run(&flow, &mut |_| {});
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
    }

    #[test]
    fn a_check_that_cannot_be_made_fails_the_step() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.on_error = OnError::Continue;
        flow.steps = vec![
            when(
                Condition::Equals {
                    value: "{{never}}".into(),
                    to: "x".into(),
                },
                vec![exec("exit 9")],
                vec![exec("exit 9")],
            ),
            exec("true"),
        ];
        let (outcome, seen) = events(&flow, &no_flows);
        assert_eq!(outcome.failures.len(), 1, "{:?}", outcome.failures);
        assert_eq!(outcome.failures[0].0, vec![0]);
        assert_eq!(started(&seen), vec![vec![0], vec![1]], "neither branch ran");
    }

    #[test]
    fn repeat_runs_its_steps_once_per_round_with_the_round_number() {
        let scratch = Scratch::new("repeat");
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![repeat(3, vec![scratch.append("round-{{index}}")])];
        let (outcome, seen) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(scratch.lines(), vec!["round-1", "round-2", "round-3"]);
        assert_eq!(outcome.ran, 4);
        let rounds: Vec<(u32, u32)> = seen
            .iter()
            .filter_map(|e| match e {
                RunEvent::Round { round, of, .. } => Some((*round, *of)),
                _ => None,
            })
            .collect();
        assert_eq!(rounds, vec![(1, 3), (2, 3), (3, 3)]);
    }

    #[test]
    fn a_step_run_alone_gets_its_values_and_reports_its_own_path() {
        let scratch = Scratch::new("alone");
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            // Never runs: it would fail the flow.
            saving(exec("exit 9"), "name"),
            repeat(1, vec![scratch.append("{{name}}-{{index}}")]),
        ];
        let mut seen = Vec::new();
        let values = vec![
            ("name".to_string(), "Ada; $(false)".to_string()),
            ("index".to_string(), "7".to_string()),
        ];
        let outcome =
            Runner::with_loader(&no_flows, true)
                .run_only(&flow, &[1, 0, 0], &values, &mut |e| seen.push(e));
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(outcome.ran, 1);
        assert_eq!(scratch.lines(), vec!["Ada; $(false)-7"]);
        assert_eq!(started(&seen), vec![vec![1, 0, 0]]);

        // The whole block alone: its loop sets the round itself.
        let outcome =
            Runner::with_loader(&no_flows, true).run_only(&flow, &[1], &values, &mut |_| {});
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(
            scratch.lines().last().map(String::as_str),
            Some("Ada; $(false)-1")
        );

        // Without its value the step fails, as it would in a run.
        let outcome = Runner::with_loader(&no_flows, true).run_only(&flow, &[1], &[], &mut |_| {});
        assert_eq!(outcome.failures.len(), 1);
    }

    #[test]
    fn each_goes_through_the_lines_of_a_value() {
        let scratch = Scratch::new("each");
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(exec("printf 'a b\\n\\n  c  \\n$(false)\\n'"), "lines"),
            each("{{lines}}", vec![scratch.append("{{index}}={{item}}")]),
        ];
        let (outcome, _) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        // Blank lines are skipped, and a line is text, never a command.
        assert_eq!(scratch.lines(), vec!["1=a b", "2=c", "3=$(false)"]);
    }

    #[test]
    fn each_over_nothing_runs_nothing() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(exec("true"), "nothing"),
            each("{{nothing}}", vec![exec("exit 9")]),
        ];
        let (outcome, _) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(outcome.ran, 2);
    }

    #[test]
    fn loops_inside_loops_keep_their_own_round() {
        let scratch = Scratch::new("nested");
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![repeat(
            2,
            vec![
                each("x\ny", vec![scratch.append("in-{{index}}-{{item}}")]),
                // The outer round again once the inner loop is done.
                scratch.append("out-{{index}}"),
            ],
        )];
        let (outcome, _) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(
            scratch.lines(),
            vec!["in-1-x", "in-2-y", "out-1", "in-1-x", "in-2-y", "out-2"]
        );
    }

    #[test]
    fn a_loops_names_are_gone_after_it() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.on_error = OnError::Continue;
        // Not valid to save (validation refuses it), but the runner must
        // not leak the names either.
        flow.steps = vec![repeat(1, vec![exec("true")]), exec("echo {{index}}")];
        let (outcome, _) = events(&flow, &no_flows);
        assert_eq!(outcome.failures.len(), 1);
        assert!(outcome.failures[0].1.contains("{{index}}"));
    }

    #[test]
    fn stop_ends_the_flow_as_finished() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            exec("true"),
            when(
                Condition::Command {
                    command: "true".into(),
                },
                vec![Step::new(StepKind::Stop), exec("exit 9")],
                vec![],
            ),
            exec("exit 9"),
        ];
        let (outcome, seen) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert!(!outcome.cancelled);
        assert_eq!(outcome.stopped_at, None);
        assert_eq!(started(&seen), vec![vec![0], vec![1], vec![1, 0, 0]]);
        assert_eq!(outcome.summary(&flow), "Flow 'T' finished");
    }

    #[test]
    fn stop_inside_a_loop_ends_every_round() {
        let scratch = Scratch::new("stop-loop");
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![repeat(
            5,
            vec![
                scratch.append("{{index}}"),
                when(
                    Condition::Equals {
                        value: "{{index}}".into(),
                        to: "2".into(),
                    },
                    vec![Step::new(StepKind::Stop)],
                    vec![],
                ),
            ],
        )];
        let (outcome, _) = events(&flow, &no_flows);
        assert!(outcome.is_ok());
        assert_eq!(scratch.lines(), vec!["1", "2"]);
    }

    #[test]
    fn a_failure_inside_a_loop_follows_the_flows_error_setting() {
        let scratch = Scratch::new("fail-loop");
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![repeat(3, vec![exec("exit 4"), scratch.append("after")])];
        let (outcome, _) = events(&flow, &no_flows);
        assert_eq!(outcome.failures.len(), 1);
        assert_eq!(outcome.stopped_at, Some(vec![0, 0, 0]));
        assert!(scratch.lines().is_empty(), "nothing ran after the failure");

        flow.on_error = OnError::Continue;
        let (outcome, _) = events(&flow, &no_flows);
        assert_eq!(outcome.failures.len(), 3, "once per round");
        assert_eq!(scratch.lines().len(), 3);
    }

    #[test]
    fn a_switched_off_block_runs_none_of_its_steps() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![repeat(2, vec![exec("exit 9")]).off(), exec("true")];
        let (outcome, seen) = events(&flow, &no_flows);
        assert!(outcome.is_ok());
        assert_eq!(started(&seen), vec![vec![1]]);
    }

    #[test]
    fn a_menu_runs_the_steps_of_the_choice_picked() {
        let scratch = Scratch::new("menu");
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(
                Step::new(StepKind::Menu {
                    prompt: "Power".into(),
                    choices: vec![
                        MenuChoice {
                            label: "Lock".into(),
                            steps: vec![exec("exit 9")],
                        },
                        MenuChoice {
                            label: " Sleep ".into(),
                            steps: vec![scratch.append("inside-{{pick}}")],
                        },
                    ],
                }),
                "pick",
            ),
            scratch.append("after-{{pick}}"),
        ];
        let prompts = Scripted::new(&[Some("Sleep")]);
        let mut seen = Vec::new();
        let outcome = Runner::with_loader(&no_flows, true)
            .prompter(&prompts)
            .run(&flow, &mut |e| seen.push(e));
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(prompts.asked(), vec!["choose Power: Lock|Sleep"]);
        assert_eq!(started(&seen), vec![vec![0], vec![0, 1, 0], vec![1]]);
        assert_eq!(scratch.lines(), vec!["inside-Sleep", "after-Sleep"]);

        // Dismissing the menu ends the flow quietly.
        let prompts = Scripted::new(&[None]);
        let outcome = Runner::with_loader(&no_flows, true)
            .prompter(&prompts)
            .run(&flow, &mut |_| {});
        assert!(outcome.cancelled && outcome.is_ok());
    }

    // MARK: Input

    use crate::system::flows::InputFallback;

    #[test]
    fn a_flow_reads_what_it_was_started_with() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![exec("test {{input}} = 'two words; $(false)'")];
        let outcome = Runner::with_loader(&no_flows, true)
            .input(Some("two words; $(false)".into()))
            .run(&flow, &mut |_| {});
        assert!(outcome.is_ok(), "{:?}", outcome.failures);

        // Started with nothing, the input is empty rather than missing.
        flow.steps = vec![exec("test -z {{input}}")];
        let (outcome, _) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        let outcome = Runner::with_loader(&no_flows, true)
            .input(Some("  \n".into()))
            .run(&flow, &mut |_| {});
        assert!(outcome.is_ok(), "blank input counts as none");
    }

    #[test]
    fn a_flow_started_with_nothing_can_ask_for_its_input() {
        let mut flow = Flow::new("t".into(), "Translate".into());
        flow.input = InputFallback::Ask;
        flow.steps = vec![exec("test {{input}} = typed")];
        let prompts = Scripted::new(&[Some("typed")]);
        let outcome = Runner::with_loader(&no_flows, true)
            .prompter(&prompts)
            .run(&flow, &mut |_| {});
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(prompts.asked(), vec!["ask Translate"]);

        // Given input, it does not ask.
        let prompts = Scripted::new(&[]);
        let outcome = Runner::with_loader(&no_flows, true)
            .prompter(&prompts)
            .input(Some("typed".into()))
            .run(&flow, &mut |_| {});
        assert!(outcome.is_ok());
        assert!(prompts.asked().is_empty());

        // Dismissing the question ends the flow before its first step.
        let prompts = Scripted::new(&[None]);
        let outcome = Runner::with_loader(&no_flows, true)
            .prompter(&prompts)
            .run(&flow, &mut |_| {});
        assert!(outcome.cancelled && outcome.is_ok());
        assert_eq!(outcome.ran, 0);
    }

    #[test]
    fn a_flow_hands_input_to_the_flow_it_runs() {
        let mut child = Flow::new("child".into(), "Child".into());
        child.steps = vec![exec("printf '%s!' {{input}}")];
        let load = move |id: &str| -> Result<Flow> {
            if id == "child" {
                Ok(child.clone())
            } else {
                Err(Error::Invalid(format!("no flow {id}")))
            }
        };
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(
                Step::new(StepKind::Flow {
                    id: "child".into(),
                    input: "from {{input}}".into(),
                }),
                "said",
            ),
            exec("test {{said}} = 'from parent!'"),
        ];
        let outcome = Runner::with_loader(&load, true)
            .input(Some("parent".into()))
            .run(&flow, &mut |_| {});
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
    }

    // MARK: Ready-made actions

    use crate::system::flows::actions::{Arg, Args};

    fn action(id: &str, pairs: &[(&str, &str)]) -> Step {
        let args: Args = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Arg::Text(v.to_string())))
            .collect();
        Step::new(StepKind::Action {
            action: id.into(),
            args,
        })
    }

    #[test]
    fn text_actions_chain_through_variables() {
        let scratch = Scratch::new("actions");
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            saving(action("text", &[("text", " a-b, c-d ")]), "raw"),
            saving(action("text.trim", &[("text", "{{raw}}")]), "trimmed"),
            saving(
                action(
                    "text.replace",
                    &[("text", "{{trimmed}}"), ("find", "-"), ("with", "+")],
                ),
                "swapped",
            ),
            saving(
                action("text.split", &[("text", "{{swapped}}"), ("by", ",")]),
                "lines",
            ),
            each(
                "{{lines}}",
                vec![
                    saving(
                        action("text.case", &[("text", "{{item}}"), ("to", "upper")]),
                        "loud",
                    ),
                    scratch.append("{{loud}}"),
                ],
            ),
        ];
        assert!(flow.validate().is_ok(), "{:?}", flow.validate());
        let (outcome, _) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
        assert_eq!(scratch.lines(), vec!["A+B", "C+D"]);
    }

    #[test]
    fn a_shell_action_gets_its_fields_as_data() {
        if !crate::system::flows::requirements::is_installed("jq") {
            return;
        }
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![
            // A value that would end the script if it were pasted into it.
            saving(
                action("text", &[("text", r#"{"name": "a\"; exit 9; \""}"#)]),
                "json",
            ),
            saving(
                action("json.get", &[("json", "{{json}}"), ("path", ".name")]),
                "name",
            ),
            exec(r#"test {{name}} = 'a"; exit 9; "'"#),
        ];
        let (outcome, _) = events(&flow, &no_flows);
        assert!(outcome.is_ok(), "{:?}", outcome.failures);
    }

    #[test]
    fn an_action_this_build_does_not_have_fails_its_step() {
        let mut flow = Flow::new("t".into(), "T".into());
        flow.steps = vec![action("from.the.future", &[])];
        assert!(flow.validate().is_err());
        let (outcome, _) = events(&flow, &no_flows);
        assert_eq!(outcome.failures.len(), 1);
        assert!(outcome.failures[0].1.contains("from.the.future"));
    }

    #[test]
    fn nested_flows_run_and_cycles_are_refused() {
        let mut inner = Flow::new("inner".into(), "Inner".into());
        inner.steps = vec![exec("true")];
        let mut looping = Flow::new("loop".into(), "Loop".into());
        looping.steps = vec![Step::new(StepKind::flow("outer"))];
        let loader = move |id: &str| match id {
            "inner" => Ok(inner.clone()),
            "loop" => Ok(looping.clone()),
            _ => Err(Error::Invalid(format!("no flow {id}"))),
        };

        let mut outer = Flow::new("outer".into(), "Outer".into());
        outer.steps = vec![
            Step::new(StepKind::flow("inner")),
            Step::new(StepKind::flow("loop")),
        ];
        let (outcome, _) = events(&outer, &loader);
        assert_eq!(outcome.failures.len(), 1);
        assert_eq!(outcome.failures[0].0, vec![1]);
        assert!(outcome.failures[0].1.contains("already running"));

        outer.steps = vec![Step::new(StepKind::flow("missing"))];
        let (outcome, _) = events(&outer, &loader);
        assert!(outcome.failures[0].1.contains("no flow missing"));
    }
}
