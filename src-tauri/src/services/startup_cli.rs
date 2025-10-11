use crate::types::{StartupCliResult, StartupCommand};
use tauri::{App, Manager};

/// Check CLI arguments during application startup and determine if app should continue
///
/// This function is called early in the application startup process to check if
/// CLI arguments indicate that the app should exit early (e.g., refresh command
/// on first instance).
///
/// # Arguments
/// * `app` - Reference to the Tauri App instance
///
/// # Returns
/// * `Result<StartupCliResult, String>` - Result indicating whether to continue startup
///
/// # Examples
/// ```text
/// let cli_result = check_cli_args(app)?;
/// if !cli_result.should_continue {
///     std::process::exit(cli_result.exit_code);
/// }
/// ```
pub fn check_cli_args(app: &App) -> Result<StartupCliResult, String> {
    log::info!("Checking CLI arguments during startup");

    // First try to get CLI matches from the Tauri CLI plugin
    if let Some(cli_matches) = app.try_state::<tauri_plugin_cli::Matches>() {
        log::info!("Found CLI matches from Tauri plugin");

        // Parse the CLI command using the structured approach
        let command = parse_startup_command(&cli_matches);
        log::info!("Detected startup command from plugin: {command:?}");

        // Determine if we should exit early
        let should_exit = should_exit_early(&command);

        if should_exit {
            let exit_reason = match command {
                StartupCommand::Refresh => {
                    "Refresh command detected on first instance - exiting without launching UI"
                        .to_string()
                },
                StartupCommand::Unknown(cmd) => {
                    format!("Unknown command '{cmd}' detected - exiting without launching UI")
                },
                _ => "Early exit requested".to_string(),
            };
            log_early_exit_reason(&exit_reason);

            return Ok(StartupCliResult {
                should_continue: false,
                exit_reason: Some(exit_reason),
                exit_code: 0,
            });
        }
    } else {
        log::info!("No CLI matches from Tauri plugin - checking raw arguments");

        // Fallback: check raw command line arguments
        let args: Vec<String> = std::env::args().collect();
        log::info!("Raw CLI arguments: {args:?}");

        let command = parse_raw_startup_command(&args);
        log::info!("Detected startup command from raw args: {command:?}");

        // Determine if we should exit early
        let should_exit = should_exit_early(&command);

        if should_exit {
            let exit_reason = match command {
                StartupCommand::Refresh => {
                    "Refresh command detected on first instance - exiting without launching UI"
                        .to_string()
                },
                StartupCommand::Unknown(cmd) => {
                    format!("Unknown command '{cmd}' detected - exiting without launching UI")
                },
                _ => "Early exit requested".to_string(),
            };
            log_early_exit_reason(&exit_reason);

            return Ok(StartupCliResult {
                should_continue: false,
                exit_reason: Some(exit_reason),
                exit_code: 0,
            });
        }
    }

    log::info!("No early exit required - continuing with normal startup");
    Ok(StartupCliResult {
        should_continue: true,
        exit_reason: None,
        exit_code: 0,
    })
}

/// Parse CLI matches to determine the startup command
///
/// # Arguments
/// * `matches` - CLI matches from Tauri CLI plugin
///
/// # Returns
/// * `StartupCommand` - The parsed startup command
fn parse_startup_command(matches: &tauri_plugin_cli::Matches) -> StartupCommand {
    // Check if refresh subcommand was used
    if matches
        .subcommand
        .as_ref()
        .and_then(|sub| {
            if sub.name == "refresh" {
                Some(sub)
            } else {
                None
            }
        })
        .is_some()
    {
        log::info!("Refresh subcommand detected in CLI arguments");
        return StartupCommand::Refresh;
    }

    // Check for any other subcommands
    if let Some(subcommand) = &matches.subcommand {
        log::warn!("Unknown subcommand detected: {}", subcommand.name);
        return StartupCommand::Unknown(subcommand.name.clone());
    }

    // No subcommands found - normal startup
    log::info!("No CLI subcommands detected - normal startup");
    StartupCommand::Normal
}

/// Parse raw CLI arguments to determine the startup command
///
/// This is a fallback when the Tauri CLI plugin doesn't provide structured matches.
///
/// # Arguments
/// * `args` - Raw command line arguments from std::env::args()
///
/// # Returns
/// * `StartupCommand` - The parsed startup command
fn parse_raw_startup_command(args: &[String]) -> StartupCommand {
    // Skip the first argument (program name) and look for commands
    if args.len() < 2 {
        log::info!("No CLI arguments provided - normal startup");
        return StartupCommand::Normal;
    }

    match args[1].as_str() {
        "refresh" => {
            log::info!("Refresh command detected in raw CLI arguments");
            StartupCommand::Refresh
        },
        unknown => {
            log::warn!("Unknown command detected in raw CLI arguments: {unknown}");
            StartupCommand::Unknown(unknown.to_string())
        },
    }
}

/// Determine if the application should exit early based on the startup command
///
/// The application should exit early when:
/// - A refresh command is detected on the first instance (no other instance running)
/// - This prevents the UI from launching when only a theme refresh was requested
///
/// # Arguments
/// * `command` - The parsed startup command
///
/// # Returns
/// * `bool` - True if the application should exit early
pub fn should_exit_early(command: &StartupCommand) -> bool {
    match command {
        StartupCommand::Refresh => {
            log::info!("Refresh command detected - checking if this is first instance");
            // For refresh commands, we should exit early on the first instance
            // The single-instance plugin will handle routing to existing instances
            true
        },
        StartupCommand::Unknown(cmd) => {
            log::warn!("Unknown command '{cmd}' - exiting early to avoid UI launch");
            // For unknown commands, exit early to avoid launching UI
            true
        },
        StartupCommand::Normal => {
            log::info!("Normal startup - no early exit required");
            false
        },
    }
}

/// Log the reason for early exit with appropriate detail
///
/// # Arguments
/// * `reason` - The reason for early exit
pub fn log_early_exit_reason(reason: &str) {
    log::info!("Early exit decision: {reason}");
    log::info!("Application will exit without launching UI");
    log::info!(
        "If an instance is already running, the command will be routed via single-instance plugin"
    );
}
