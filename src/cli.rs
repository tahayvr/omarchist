use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use crate::system::flows::runner::{RunEvent, Runner};
use crate::system::flows::store::{find_flow, load_flows};

#[derive(Parser, Debug, Clone)]
#[command(name = "omarchist")]
#[command(about = "Omarchy system and theme manager")]
#[command(version)]
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
            let outcome = Runner::new(false).run(&flow, &mut |event| match event {
                RunEvent::Started { index } => {
                    println!(
                        "[{}/{}] {}",
                        index + 1,
                        flow.steps.len(),
                        flow.steps[index].kind.text()
                    );
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

fn notify_failure(message: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["-a", "Omarchist", "-u", "normal", "Flow failed", message])
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
