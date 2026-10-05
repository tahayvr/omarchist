use std::io::{IsTerminal, Read, Write};
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::shell::theme_sh_commands::apply_theme;
use crate::system::flows::catalog;
use crate::system::flows::history::{self, Recorder};
use crate::system::flows::runner::{Cancel, RunEvent, Runner, StepStatus};
use crate::system::flows::running;
use crate::system::flows::share::{ImportSource, export_file_name, export_toml, read_import};
use crate::system::flows::store::{existing_ids, find_flow, load_flows, save_new_flow};
use crate::system::flows::unique_id;
use crate::system::notify;
use crate::system::omarchy_paths::user_themes_dir;
use crate::system::themes::theme_file_ops::is_system_theme;
use crate::system::themes::theme_generator::create_theme_from_image;
use crate::system::themes::theme_management::{
    generate_unique_theme_name, slugify_theme_name, unique_theme_name,
};

#[derive(Parser, Debug, Clone)]
#[command(name = "omarchist")]
#[command(about = "Omarchy system and theme manager")]
#[command(version)]
#[command(args_conflicts_with_subcommands = true)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(short, long, value_enum)]
    pub view: Option<ViewOption>,

    /// Open this theme in the Theme Designer (with `--view themes`)
    #[arg(short, long, requires = "view")]
    pub theme: Option<String>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewOption {
    Themes,
    Settings,
    About,
    Omarchy,
    Config,
    Keybinds,
    Flows,
}

impl ViewOption {
    /// The name used on the command line and in `settings.json`.
    pub fn name(&self) -> &'static str {
        match self {
            ViewOption::Themes => "themes",
            ViewOption::Config => "config",
            ViewOption::Keybinds => "keybinds",
            ViewOption::Flows => "flows",
            ViewOption::Settings => "settings",
            ViewOption::About => "about",
            ViewOption::Omarchy => "omarchy",
        }
    }
}

/// Commands that run without opening the window.
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Run or list the flows made on the Flows page
    Flow {
        #[command(subcommand)]
        action: FlowCommand,
    },
    /// Make themes without opening the window
    Theme {
        #[command(subcommand)]
        action: ThemeCommand,
    },
    /// The background service that starts flows by themselves, on a
    /// schedule or when something happens
    Automations {
        #[command(subcommand)]
        action: AutomationsCommand,
    },
    /// Remove everything Omarchist added outside its package (Hyprland
    /// hook, bar widget, flow launchers, settings); themes stay
    Uninstall {
        /// Remove without asking
        #[arg(short, long)]
        yes: bool,
    },
    /// Tools for the repository behind the gallery; what its CI runs
    #[command(hide = true)]
    Catalog {
        #[command(subcommand)]
        action: CatalogCommand,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum CatalogCommand {
    /// Build the index and the versioned flow files from a checkout
    Build {
        /// The catalog repository's folder (it holds `flows/`)
        repo: PathBuf,
        /// Where to write `v1/index.json` and `v1/flows/`
        #[arg(short, long)]
        out: PathBuf,
        /// The index being replaced, to carry dates forward and refuse a
        /// flow that changed without a new version
        #[arg(long)]
        previous: Option<PathBuf>,
    },
    /// Make a signing key: prints the public key, writes the private one
    NewKey {
        /// The file to write the private key to
        #[arg(short, long)]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum AutomationsCommand {
    /// Turn automations on: install the service and start it
    On,
    /// Turn automations off: stop the service and remove it
    Off,
    /// Say whether the service is running and what it watches for
    Status,
    /// Watch and start flows until stopped; what the service itself runs
    Run,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ThemeCommand {
    /// Build a theme from an image's colours, with the image as its wallpaper
    FromImage {
        /// A picture file
        image: PathBuf,
        /// The theme's name (defaults to the image's file name)
        #[arg(short, long)]
        name: Option<String>,
        /// Switch to the new theme once it is written
        #[arg(short, long)]
        apply: bool,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum FlowCommand {
    /// Run a flow by name or id
    Run {
        /// The flow's name (case-insensitive) or id
        name: String,
        /// What started the run, for the run history; set by automations
        #[arg(long, hide = true)]
        trigger: Option<String>,
        /// What the flow gets as its input, after `--`: text, or files,
        /// one per line. Piped text works too
        #[arg(last = true)]
        input: Vec<String>,
    },
    /// Stop a flow that is running
    Stop {
        /// The flow's name (case-insensitive) or id
        name: String,
    },
    /// Show a flow's last runs: how each ended, when, and what started it
    History {
        /// The flow's name (case-insensitive) or id
        name: String,
        /// Print JSON: every kept run with its steps
        #[arg(long)]
        json: bool,
    },
    /// List every flow with its id
    List {
        /// Print JSON: an array of {id, name, icon, glyph, steps, running}
        #[arg(long)]
        json: bool,
    },
    /// Write a flow as a shareable file (to stdout unless --output is given)
    Export {
        /// The flow's name (case-insensitive) or id
        name: String,
        /// A file to write, or a directory to write `<id>.flow.toml` into
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Add a flow from a file or an https:// URL, after showing its steps
    Import {
        /// A `.flow.toml` file or an https:// URL
        source: String,
        /// Save without asking
        #[arg(short, long)]
        yes: bool,
    },
}

impl CliArgs {
    pub fn parse_args() -> Self {
        let args = Self::parse();
        if args.theme.is_some() && args.view != Some(ViewOption::Themes) {
            let mut cmd = <Self as clap::CommandFactory>::command();
            cmd.error(
                clap::error::ErrorKind::ArgumentConflict,
                "--theme only goes with --view themes",
            )
            .exit();
        }
        args
    }
}

/// Runs a `Command` and reports on stdout and stderr. A failed flow also
/// raises a desktop notification, because a keybind gives no other feedback.
pub fn run_command(command: &Command) -> ExitCode {
    match command {
        Command::Flow {
            action: FlowCommand::List { json: true },
        } => match load_flows() {
            Ok(flows) => {
                let running = running::list();
                let entries: Vec<serde_json::Value> = flows
                    .iter()
                    .map(|flow| {
                        serde_json::json!({
                            "id": flow.id,
                            "name": flow.name,
                            "icon": flow.icon,
                            "glyph": crate::system::flows::icon_glyph(&flow.icon).to_string(),
                            "steps": flow.enabled_steps(),
                            "running": running.iter().any(|run| run.id == flow.id),
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        Command::Flow {
            action: FlowCommand::List { json: false },
        } => match load_flows() {
            Ok(flows) if flows.is_empty() => {
                println!("No flows yet. Create one on the Flows page of Omarchist.");
                ExitCode::SUCCESS
            }
            Ok(flows) => {
                let width = flows.iter().map(|f| f.id.len()).max().unwrap_or(0);
                for flow in flows {
                    let steps = flow.enabled_steps();
                    println!(
                        "{:width$}  {}  ({} step{})",
                        flow.id,
                        flow.name,
                        steps,
                        if steps == 1 { "" } else { "s" }
                    );
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        Command::Theme {
            action: ThemeCommand::FromImage { image, name, apply },
        } => theme_from_image(image, name.as_deref(), *apply),
        Command::Uninstall { yes } => uninstall(*yes),
        Command::Catalog { action } => catalog_command(action),
        Command::Automations { action } => automations(action),
        Command::Flow {
            action: FlowCommand::Export { name, output },
        } => export(name, output.as_ref()),
        Command::Flow {
            action: FlowCommand::Import { source, yes },
        } => import(source, *yes),
        Command::Flow {
            action:
                FlowCommand::Run {
                    name,
                    input,
                    trigger,
                },
        } => run_flow(name, input, trigger.as_deref()),
        Command::Flow {
            action: FlowCommand::Stop { name },
        } => match find_flow(name) {
            Ok(flow) => match running::stop(&flow.id) {
                0 => {
                    println!("'{}' is not running.", flow.name);
                    ExitCode::SUCCESS
                }
                _ => {
                    println!("Stopping '{}'.", flow.name);
                    ExitCode::SUCCESS
                }
            },
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        Command::Flow {
            action: FlowCommand::History { name, json },
        } => flow_history(name, *json),
    }
}

/// What started a run nobody labelled: a terminal, or else a keybind
/// (Hyprland starts those detached, so their parent is the session
/// manager) or a script.
fn default_trigger() -> &'static str {
    if std::io::stdin().is_terminal() || std::io::stdout().is_terminal() {
        return "Terminal";
    }
    let parent = std::os::unix::process::parent_id();
    let parent_name = std::fs::read_to_string(format!("/proc/{parent}/comm")).unwrap_or_default();
    if parent == 1 || parent_name.trim() == "systemd" {
        "Keybind"
    } else {
        "Script"
    }
}

fn run_flow(name: &str, input: &[String], trigger: Option<&str>) -> ExitCode {
    // The editor's guard on Lua steps applies to a hand-edited file too, so
    // nothing reaches `hyprctl dispatch` unchecked.
    let flow = match find_flow(name).and_then(|flow| flow.validate().map(|()| flow)) {
        Ok(flow) => flow,
        Err(e) => {
            eprintln!("{e}");
            notify_failure(&e.to_string());
            return ExitCode::FAILURE;
        }
    };
    let trigger = trigger.unwrap_or_else(|| default_trigger());
    let total = flow.enabled_steps();
    println!(
        "Running '{}' ({} step{})",
        flow.name,
        total,
        if total == 1 { "" } else { "s" }
    );
    // While it runs, the bar widget and the Flows page can see it and ask
    // it to stop, which arrives here as a signal.
    let _registered = running::register(&flow, trigger);
    let cancel = Cancel::new();
    let stopped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
        let (cancel, stopped) = (cancel.clone(), stopped.clone());
        if let Ok(mut signals) = signal_hook::iterator::Signals::new([SIGTERM, SIGINT, SIGHUP]) {
            std::thread::spawn(move || {
                if signals.forever().next().is_some() {
                    stopped.store(true, std::sync::atomic::Ordering::SeqCst);
                    cancel.cancel();
                }
            });
        }
    }
    let mut recorder = Recorder::new(&flow, trigger);
    // Steps print under their number, indented by how deep they sit inside
    // other steps.
    let indent = |path: &[usize]| "  ".repeat(path.len().div_ceil(2));
    let outcome = Runner::new(false)
        .cancellable(cancel)
        .input(run_input(input))
        .run(&flow, &mut |event| {
            recorder.event(&event);
            match event {
                RunEvent::Started { path } => {
                    if let Some(step) = flow.step_at(&path) {
                        println!(
                            "{}{}. {}",
                            indent(&path),
                            flow.step_number(&path),
                            step.kind.text()
                        );
                    }
                }
                RunEvent::Round { path, round, of } => {
                    println!("{}  round {round} of {of}", indent(&path));
                }
                RunEvent::Finished {
                    path,
                    status: StepStatus::Failed(error),
                } => eprintln!("{}   failed: {error}", indent(&path)),
                RunEvent::Finished { .. } => {}
            }
        });
    let stopped = stopped.load(std::sync::atomic::Ordering::SeqCst);
    let run = recorder.finish(&flow, &outcome, stopped);
    if let Err(e) = history::record(&flow.id, &run) {
        eprintln!("{e}");
    }
    let summary = run.summary.clone();
    if stopped {
        // Stopping is the person's choice, as dismissing a prompt is.
        println!("{summary}");
        ExitCode::SUCCESS
    } else if outcome.cancelled && outcome.is_ok() {
        // Not a failure, and it needs no notification.
        println!("{summary}");
        ExitCode::SUCCESS
    } else if outcome.is_ok() {
        println!("{summary}");
        if crate::system::config::config_setup::settings().notify_flows {
            notify::send(
                &format!("{} finished", flow.name),
                &summary,
                notify::Urgency::Low,
            );
        }
        ExitCode::SUCCESS
    } else {
        eprintln!("{summary}");
        notify_failure(&summary);
        ExitCode::FAILURE
    }
}

/// Prints a flow's last runs, newest first.
fn flow_history(name: &str, json: bool) -> ExitCode {
    let flow = match find_flow(name) {
        Ok(flow) => flow,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let runs = history::load(&flow.id);
    if json {
        println!(
            "{}",
            serde_json::to_string(&runs).unwrap_or_else(|_| "[]".to_string())
        );
        return ExitCode::SUCCESS;
    }
    if runs.is_empty() {
        println!("'{}' has not run yet.", flow.name);
        return ExitCode::SUCCESS;
    }
    let now = chrono::Local::now().timestamp();
    for run in runs {
        println!(
            "{:<10} {:<14} {:<9} {}",
            run.result.label(),
            history::ago(run.started, now),
            history::took(run.ms),
            run.trigger
        );
        for step in run
            .steps
            .iter()
            .filter(|step| step.result == history::StepResult::Failed)
        {
            println!("           step {}: {}", step.number, step.detail);
        }
    }
    ExitCode::SUCCESS
}

fn automations(action: &AutomationsCommand) -> ExitCode {
    use crate::system::flows::{automations, service};
    let result = match action {
        AutomationsCommand::Run => automations::run(),
        AutomationsCommand::On => service::enable().map(|()| println!("Automations are on.")),
        AutomationsCommand::Off => service::disable().map(|()| println!("Automations are off.")),
        AutomationsCommand::Status => {
            println!(
                "The service is {}.",
                match (service::is_enabled(), service::is_active()) {
                    (true, true) => "on and running",
                    (true, false) => "on but not running",
                    (false, _) => "off",
                }
            );
            for flow in load_flows().unwrap_or_default() {
                for automation in &flow.triggers.automations {
                    println!(
                        "  {}: {}{}",
                        flow.name,
                        automation.event.describe(),
                        if automation.enabled { "" } else { "  (off)" }
                    );
                }
            }
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// A flow's input from the command line: the words after `--`, one per
/// line, or else what is piped in. A terminal, `/dev/null` (a keybind, the
/// launcher), or anything else that is not a pipe or a file is not input.
fn run_input(args: &[String]) -> Option<String> {
    if !args.is_empty() {
        return Some(args.join("\n"));
    }
    use std::os::unix::fs::FileTypeExt;
    let stdin = std::io::stdin();
    let kind = std::fs::File::from(stdin.as_fd().try_clone_to_owned().ok()?)
        .metadata()
        .ok()?
        .file_type();
    if !(kind.is_fifo() || kind.is_file()) {
        return None;
    }
    let mut text = String::new();
    stdin
        .lock()
        .take(crate::system::flows::vars::MAX_VALUE_BYTES as u64)
        .read_to_string(&mut text)
        .ok()?;
    Some(text)
}

fn export(name: &str, output: Option<&PathBuf>) -> ExitCode {
    let flow = match find_flow(name) {
        Ok(flow) => flow,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let text = match export_toml(&flow) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(output) = output else {
        print!("{text}");
        return ExitCode::SUCCESS;
    };
    let path = if output.is_dir() {
        output.join(export_file_name(&flow))
    } else {
        output.clone()
    };
    match std::fs::write(&path, text) {
        Ok(()) => {
            println!("Exported '{}' to {}", flow.name, path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Could not write {}: {e}", path.display());
            ExitCode::FAILURE
        }
    }
}

fn catalog_command(action: &CatalogCommand) -> ExitCode {
    let result = match action {
        CatalogCommand::Build {
            repo,
            out,
            previous,
        } => build_catalog(repo, out, previous.as_deref()),
        CatalogCommand::NewKey { out } => new_catalog_key(out),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// The name of the variable the catalog's CI keeps its private key in.
const SIGNING_KEY_VARIABLE: &str = "OMARCHIST_CATALOG_SIGNING_KEY";

fn build_catalog(repo: &Path, out: &Path, previous: Option<&Path>) -> crate::error::Result<()> {
    use crate::error::Error;
    let previous: Option<catalog::Index> = match previous {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .map_err(|e| Error::io("Failed to read the previous index", e))?;
            Some(catalog::read_published(text.as_bytes())?)
        }
        None => None,
    };
    let now = chrono::Utc::now();
    let index = catalog::build(
        repo,
        out,
        previous.as_ref(),
        &now.format("%Y-%m-%d").to_string(),
        now.timestamp().max(0) as u64,
    )?;
    println!("{} flows in the index.", index.flows.len());
    let key = std::env::var(SIGNING_KEY_VARIABLE)
        .ok()
        .filter(|key| !key.trim().is_empty());
    catalog::write_index(out, &index, key.as_deref())?;
    match &key {
        Some(key) => println!("Signed with the key {}.", catalog::public_key_of(key)?),
        None => println!("Not signed: {SIGNING_KEY_VARIABLE} is not set."),
    }
    Ok(())
}

fn new_catalog_key(out: &Path) -> crate::error::Result<()> {
    use crate::error::Error;
    use std::os::unix::fs::OpenOptionsExt;
    let (private, public) = catalog::new_key()?;
    // Never over a key that exists: losing one orphans the catalog.
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(out)
        .and_then(|mut file| file.write_all(private.as_bytes()))
        .map_err(|e| Error::io(format!("Failed to write {}", out.display()), e))?;
    println!("Private key: {} (keep it secret)", out.display());
    println!("Public key:  {public}");
    Ok(())
}

/// Shows what the flow would do and saves it only after a yes, because a
/// flow is a list of commands from someone else.
fn import(source: &str, yes: bool) -> ExitCode {
    let imported = match ImportSource::parse(source).and_then(|s| read_import(&s)) {
        Ok(imported) => imported,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let flow = &imported.flow;
    println!("'{}' from {}", flow.name, imported.origin);
    if !flow.description.is_empty() {
        println!("  {}", flow.description);
    }
    if !flow.meta.author.is_empty() {
        println!("  by {}", flow.meta.author);
    }
    println!("Steps:");
    for (number, (path, step)) in flow.walk().into_iter().enumerate() {
        let off = if step.enabled { "" } else { "  (off)" };
        println!(
            "{}{}. {}{off}",
            "  ".repeat(path.len().div_ceil(2)),
            number + 1,
            step.kind.text()
        );
    }
    if !flow.meta.requires.is_empty() {
        println!("Needs: {}", flow.meta.requires.join(", "));
    }
    let risks = crate::system::flows::risks::risks(flow);
    if !risks.is_empty() {
        println!("Worth knowing:");
        for risk in &risks {
            println!("  Step {}: {}", risk.step, risk.what);
        }
    }
    if !yes {
        if !std::io::stdin().is_terminal() {
            eprintln!("Not a terminal; pass --yes to import without confirmation");
            return ExitCode::FAILURE;
        }
        if !confirm("Save this flow?") {
            println!("Not saved.");
            return ExitCode::SUCCESS;
        }
    }
    let mut flow = imported.flow;
    flow.id = unique_id(&flow.name, &existing_ids());
    match save_new_flow(&flow) {
        Ok(()) => {
            println!("Saved as '{}'. Run it with: {}", flow.id, flow.command());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Could not save the flow: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The same steps as the New theme dialog: a folder named after the
/// image (or `--name`), a palette extracted from it, and the image copied
/// in as the wallpaper.
fn theme_from_image(image: &Path, name: Option<&str>, apply: bool) -> ExitCode {
    if !image.is_file() {
        eprintln!("No such image: {}", image.display());
        return ExitCode::FAILURE;
    }
    let theme_name = match name {
        Some(name) => {
            let slug = slugify_theme_name(name);
            if is_system_theme(&slug) {
                eprintln!("'{slug}' is one of Omarchy's own themes; choose another name");
                return ExitCode::FAILURE;
            }
            slug
        }
        None => image
            .file_stem()
            .and_then(|s| s.to_str())
            .map(slugify_theme_name)
            .map(|base| unique_theme_name(&base))
            .unwrap_or_else(generate_unique_theme_name),
    };
    if let Err(e) = create_theme_from_image(image, &theme_name) {
        eprintln!("Could not create the theme: {e}");
        return ExitCode::FAILURE;
    }
    match user_themes_dir() {
        Some(dir) => println!(
            "Created '{theme_name}' in {}",
            dir.join(&theme_name).display()
        ),
        None => println!("Created '{theme_name}'"),
    }
    if apply {
        if let Err(e) = smol::block_on(apply_theme(theme_name.clone())) {
            eprintln!("Could not apply the theme: {e}");
            return ExitCode::FAILURE;
        }
        println!("Applied.");
    } else {
        println!("Apply it with: omarchy-theme-set {theme_name}");
    }
    println!("Edit it with: omarchist --view themes --theme {theme_name}");
    ExitCode::SUCCESS
}

/// Lists what Omarchist added to this machine, asks, and removes it in
/// order, stopping at the first failure so nothing is half done. A running
/// window is told nothing: its next save would recreate the files, so the
/// user is asked to quit it first.
fn uninstall(yes: bool) -> ExitCode {
    let steps = crate::system::uninstall::plan();
    if steps.is_empty() {
        println!("Nothing to remove: Omarchist has not changed this machine.");
        return ExitCode::SUCCESS;
    }
    if crate::system::instance::is_running() {
        eprintln!("Quit Omarchist first, then run this again.");
        return ExitCode::FAILURE;
    }
    println!("This will:");
    for step in &steps {
        println!("  - {}", step.describe());
    }
    println!("Themes made with Omarchist stay in ~/.config/omarchy/themes.");
    if !yes {
        if !std::io::stdin().is_terminal() {
            eprintln!("Not a terminal; pass --yes to remove without confirmation");
            return ExitCode::FAILURE;
        }
        if !confirm("Remove all of this?") {
            println!("Nothing removed.");
            return ExitCode::SUCCESS;
        }
    }
    let mut reload = false;
    for step in &steps {
        if let Err(e) = crate::system::uninstall::run(step) {
            eprintln!("{}: {e}", step.describe());
            eprintln!("Stopped; run this again once that is fixed.");
            return ExitCode::FAILURE;
        }
        reload |= matches!(
            step,
            crate::system::uninstall::Step::RequireLine(_)
                | crate::system::uninstall::Step::OmarchistLua(_)
        );
        println!("Done: {}", step.describe());
    }
    if reload {
        crate::system::hyprland_config::manager::reload_hyprland_blocking();
    }
    println!("Omarchist's files are gone. Remove the package with: sudo pacman -R omarchist-bin");
    ExitCode::SUCCESS
}

fn confirm(question: &str) -> bool {
    print!("{question} [y/N] ");
    std::io::stdout().flush().ok();
    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer).is_err() {
        return false;
    }
    matches!(answer.trim().to_lowercase().as_str(), "y" | "yes")
}

fn notify_failure(message: &str) {
    notify::send("Flow failed", message, notify::Urgency::Normal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_themes_view() {
        let args = CliArgs::parse_from(["omarchist", "--view", "themes"]);
        assert_eq!(args.view, Some(ViewOption::Themes));
        assert_eq!(args.theme, None);
    }

    #[test]
    fn test_parse_config_view() {
        let args = CliArgs::parse_from(["omarchist", "--view", "config"]);
        assert_eq!(args.view, Some(ViewOption::Config));
    }

    #[test]
    fn test_parse_keybinds_view() {
        let args = CliArgs::parse_from(["omarchist", "--view", "keybinds"]);
        assert_eq!(args.view, Some(ViewOption::Keybinds));
    }

    #[test]
    fn test_parse_flow_export_and_import() {
        let args =
            CliArgs::parse_from(["omarchist", "flow", "export", "morning-start", "-o", "/tmp"]);
        assert_eq!(
            args.command,
            Some(Command::Flow {
                action: FlowCommand::Export {
                    name: "morning-start".into(),
                    output: Some(PathBuf::from("/tmp")),
                },
            })
        );
        let args = CliArgs::parse_from(["omarchist", "flow", "import", "a.flow.toml", "--yes"]);
        assert_eq!(
            args.command,
            Some(Command::Flow {
                action: FlowCommand::Import {
                    source: "a.flow.toml".into(),
                    yes: true,
                },
            })
        );
    }

    #[test]
    fn test_parse_theme_from_image() {
        let args = CliArgs::parse_from(["omarchist", "theme", "from-image", "pic.png"]);
        assert_eq!(
            args.command,
            Some(Command::Theme {
                action: ThemeCommand::FromImage {
                    image: PathBuf::from("pic.png"),
                    name: None,
                    apply: false,
                },
            })
        );
        let args = CliArgs::parse_from([
            "omarchist",
            "theme",
            "from-image",
            "pic.png",
            "--name",
            "Sunset",
            "--apply",
        ]);
        assert_eq!(
            args.command,
            Some(Command::Theme {
                action: ThemeCommand::FromImage {
                    image: PathBuf::from("pic.png"),
                    name: Some("Sunset".into()),
                    apply: true,
                },
            })
        );
        assert!(CliArgs::try_parse_from(["omarchist", "theme", "from-image"]).is_err());
        assert!(CliArgs::try_parse_from(["omarchist", "theme"]).is_err());
    }

    #[test]
    fn test_parse_flows_view() {
        let args = CliArgs::parse_from(["omarchist", "--view", "flows"]);
        assert_eq!(args.view, Some(ViewOption::Flows));
        assert_eq!(args.command, None);
    }

    #[test]
    fn test_parse_flow_subcommands() {
        let args = CliArgs::parse_from(["omarchist", "flow", "run", "Morning start"]);
        assert_eq!(
            args.command,
            Some(Command::Flow {
                action: FlowCommand::Run {
                    name: "Morning start".into(),
                    trigger: None,
                    input: Vec::new(),
                }
            })
        );
        // What follows `--` is the flow's input, dashes and all.
        let args = CliArgs::parse_from([
            "omarchist",
            "flow",
            "run",
            "archive",
            "--",
            "a b.txt",
            "--list",
            "-x",
        ]);
        assert_eq!(
            args.command,
            Some(Command::Flow {
                action: FlowCommand::Run {
                    name: "archive".into(),
                    trigger: None,
                    input: vec!["a b.txt".into(), "--list".into(), "-x".into()],
                }
            })
        );
        assert_eq!(
            run_input(&["a b.txt".to_string(), "c".to_string()]).as_deref(),
            Some("a b.txt\nc")
        );
        assert!(
            CliArgs::try_parse_from(["omarchist", "flow", "run", "archive", "stray"]).is_err(),
            "input needs the `--`, so a typo is not taken for it"
        );
        let args = CliArgs::parse_from(["omarchist", "flow", "list"]);
        assert_eq!(
            args.command,
            Some(Command::Flow {
                action: FlowCommand::List { json: false }
            })
        );
        assert!(CliArgs::try_parse_from(["omarchist", "flow", "run"]).is_err());
        assert!(CliArgs::try_parse_from(["omarchist", "flow"]).is_err());
        assert!(
            CliArgs::try_parse_from(["omarchist", "--view", "flows", "flow", "list"]).is_err(),
            "a view cannot be combined with a subcommand that never opens the window"
        );
    }

    #[test]
    fn test_unknown_view_is_rejected() {
        assert!(CliArgs::try_parse_from(["omarchist", "--view", "system"]).is_err());
    }

    #[test]
    fn test_parse_with_theme() {
        let args = CliArgs::parse_from(["omarchist", "--view", "themes", "--theme", "my-theme"]);
        assert_eq!(args.view, Some(ViewOption::Themes));
        assert_eq!(args.theme, Some("my-theme".to_string()));
    }
}
