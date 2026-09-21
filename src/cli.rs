use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::system::flows::runner::{RunEvent, Runner};
use crate::system::flows::share::{ImportSource, export_file_name, export_toml, read_import};
use crate::system::flows::store::{existing_ids, find_flow, load_flows, save_flow};
use crate::system::flows::unique_id;

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

/// Commands that run without opening the window.
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Run or list the flows made on the Flows page
    Flow {
        #[command(subcommand)]
        action: FlowCommand,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum FlowCommand {
    /// Run a flow by name or id
    Run {
        /// The flow's name (case-insensitive) or id
        name: String,
    },
    /// List every flow with its id
    List,
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
        Self::parse()
    }
}

/// Runs a `Command` and reports on stdout and stderr. A failed flow also
/// raises a desktop notification, because a keybind gives no other feedback.
pub fn run_command(command: &Command) -> ExitCode {
    match command {
        Command::Flow {
            action: FlowCommand::List,
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
        Command::Flow {
            action: FlowCommand::Export { name, output },
        } => export(name, output.as_ref()),
        Command::Flow {
            action: FlowCommand::Import { source, yes },
        } => import(source, *yes),
        Command::Flow {
            action: FlowCommand::Run { name },
        } => {
            let flow = match find_flow(name) {
                Ok(flow) => flow,
                Err(e) => {
                    eprintln!("{e}");
                    notify_failure(&e.to_string());
                    return ExitCode::FAILURE;
                }
            };
            let total = flow.enabled_steps();
            println!(
                "Running '{}' ({} step{})",
                flow.name,
                total,
                if total == 1 { "" } else { "s" }
            );
            let mut ran = 0;
            let outcome = Runner::new(false).run(&flow, &mut |event| match event {
                RunEvent::Started { index } => {
                    ran += 1;
                    println!("[{ran}/{total}] {}", flow.steps[index].kind.text());
                }
                RunEvent::Finished {
                    error: Some(error), ..
                } => eprintln!("      failed: {error}"),
                RunEvent::Finished { .. } => {}
            });
            let summary = outcome.summary(&flow);
            if outcome.is_ok() {
                println!("{summary}");
                ExitCode::SUCCESS
            } else {
                eprintln!("{summary}");
                notify_failure(&summary);
                ExitCode::FAILURE
            }
        }
    }
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
    for (ix, step) in flow.steps.iter().enumerate() {
        let off = if step.enabled { "" } else { "  (off)" };
        println!("  {}. {}{off}", ix + 1, step.kind.text());
    }
    let mut triggers = Vec::new();
    if flow.triggers.startup {
        triggers.push("at startup");
    }
    if flow.triggers.launcher {
        triggers.push("the app launcher");
    }
    if !triggers.is_empty() {
        println!("Would also run from: {}", triggers.join(", "));
    }
    if !flow.meta.requires.is_empty() {
        println!("Needs: {}", flow.meta.requires.join(", "));
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
    match save_flow(&flow) {
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
    let _ = std::process::Command::new("notify-send")
        .args([
            "-a",
            "Omarchist",
            "-u",
            "normal",
            "--",
            "Flow failed",
            message,
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
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
                    name: "Morning start".into()
                }
            })
        );
        let args = CliArgs::parse_from(["omarchist", "flow", "list"]);
        assert_eq!(
            args.command,
            Some(Command::Flow {
                action: FlowCommand::List
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
