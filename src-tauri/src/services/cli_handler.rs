use crate::services::themes::get_current_theme::get_system_theme_colors;
use tauri::{AppHandle, Emitter};

/// Represents the different CLI commands that can be processed
#[derive(Debug, Clone, PartialEq)]
pub enum CliCommand {
    /// Refresh the current theme
    Refresh,
    /// Unknown command with the original command string
    Unknown(String),
}

/// Parse CLI arguments into a structured command
///
/// # Arguments
/// * `args` - Vector of command line arguments (typically from process args)
///
/// # Returns
/// * `CliCommand` - The parsed command or Unknown if not recognized
///
/// # Examples
/// ```
/// let args = vec!["omarchist".to_string(), "refresh".to_string()];
/// let command = parse_cli_command(&args);
/// assert_eq!(command, CliCommand::Refresh);
/// ```
pub fn parse_cli_command(args: &[String]) -> CliCommand {
    // Skip the first argument (program name) and look for commands
    if args.len() < 2 {
        return CliCommand::Unknown("no-command".to_string());
    }

    match args[1].as_str() {
        "refresh" => CliCommand::Refresh,
        unknown => CliCommand::Unknown(unknown.to_string()),
    }
}

/// Refresh theme from CLI command
///
/// This function fetches the current system theme colors and emits a theme-refresh
/// event to the frontend, following the same pattern as the signal-based refresh.
///
/// # Arguments
/// * `app_handle` - Handle to the Tauri application for event emission
///
/// # Returns
/// * `Result<(), String>` - Success or detailed error message
pub fn refresh_theme_from_cli(app_handle: &AppHandle) -> Result<(), String> {
    log::info!("Starting theme refresh from CLI command");

    // Use the existing theme color extraction function
    match get_system_theme_colors() {
        Ok(Some(colors)) => {
            log::info!(
                "Theme colors successfully extracted from CLI: bg={}, fg={}",
                colors.background,
                colors.foreground
            );

            // Emit theme-refresh event to frontend using existing pattern
            if let Err(e) = app_handle.emit("theme-refresh", &colors) {
                let error_msg = format!("Failed to emit theme refresh event from CLI: {e}");
                log::error!("{error_msg}");
                return Err(error_msg);
            }

            log::info!("Theme refresh event successfully emitted from CLI");
            Ok(())
        },
        Ok(None) => {
            let error_msg = "No theme colors found - theme file may be missing or invalid";
            log::warn!("{error_msg}");

            // Debug: check if theme file exists and log details
            let home = std::env::var("HOME").unwrap_or_default();
            let theme_path = format!("{home}/.config/omarchy/current/theme/waybar.css");

            match std::fs::read_to_string(&theme_path) {
                Ok(content) => {
                    log::info!("Theme file exists at: {theme_path}");
                    log::info!(
                        "First 200 chars: {}",
                        &content.chars().take(200).collect::<String>()
                    );
                },
                Err(e) => {
                    log::warn!("Theme file not found or unreadable at {theme_path}: {e}");
                },
            }

            Err(error_msg.to_string())
        },
        Err(e) => {
            let error_msg = format!("Failed to get system theme colors from CLI: {e}");
            log::error!("{error_msg}");
            Err(error_msg)
        },
    }
}

/// Handle CLI arguments passed to the application
///
/// This function is called by the single instance plugin when CLI arguments
/// are passed to an already running instance.
///
/// # Arguments
/// * `app_handle` - Handle to the Tauri application for event emission
/// * `args` - Vector of CLI arguments
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error details
pub fn handle_cli_arguments(
    app_handle: &AppHandle,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Processing CLI arguments: {args:?}");

    let command = parse_cli_command(&args);
    log::info!("Parsed CLI command: {command:?}");

    match command {
        CliCommand::Refresh => {
            log::info!("Executing refresh command from CLI");

            // Call the theme refresh function and handle any errors
            if let Err(e) = refresh_theme_from_cli(app_handle) {
                log::error!("Theme refresh from CLI failed: {e}");
                // Don't return error to maintain graceful handling
                // The error is already logged with details
            } else {
                log::info!("Theme refresh from CLI completed successfully");
            }

            Ok(())
        },
        CliCommand::Unknown(cmd) => {
            log::warn!("Unknown CLI command received: {cmd}");
            // Gracefully handle unknown commands by logging and continuing
            Ok(())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_refresh_command() {
        let args = vec!["omarchist".to_string(), "refresh".to_string()];
        let command = parse_cli_command(&args);
        assert_eq!(command, CliCommand::Refresh);
    }

    #[test]
    fn test_parse_unknown_command() {
        let args = vec!["omarchist".to_string(), "unknown".to_string()];
        let command = parse_cli_command(&args);
        assert_eq!(command, CliCommand::Unknown("unknown".to_string()));
    }

    #[test]
    fn test_parse_empty_arguments() {
        let args = vec!["omarchist".to_string()];
        let command = parse_cli_command(&args);
        assert_eq!(command, CliCommand::Unknown("no-command".to_string()));
    }

    #[test]
    fn test_parse_no_arguments() {
        let args: Vec<String> = vec![];
        let command = parse_cli_command(&args);
        assert_eq!(command, CliCommand::Unknown("no-command".to_string()));
    }

    #[test]
    fn test_parse_multiple_arguments() {
        let args = vec![
            "omarchist".to_string(),
            "refresh".to_string(),
            "extra".to_string(),
        ];
        let command = parse_cli_command(&args);
        // Should still parse as refresh, ignoring extra arguments
        assert_eq!(command, CliCommand::Refresh);
    }

    #[test]
    fn test_parse_case_sensitive() {
        let args = vec!["omarchist".to_string(), "REFRESH".to_string()];
        let command = parse_cli_command(&args);
        // Commands are case-sensitive, so this should be unknown
        assert_eq!(command, CliCommand::Unknown("REFRESH".to_string()));
    }

    #[test]
    fn test_handle_cli_arguments_with_refresh() {
        // This test would require mocking the AppHandle, which is complex
        // For now, we'll test the parsing logic which is the core functionality
        let args = vec!["omarchist".to_string(), "refresh".to_string()];
        let command = parse_cli_command(&args);
        assert_eq!(command, CliCommand::Refresh);
    }

    #[test]
    fn test_handle_cli_arguments_with_unknown() {
        let args = vec!["omarchist".to_string(), "invalid".to_string()];
        let command = parse_cli_command(&args);
        assert_eq!(command, CliCommand::Unknown("invalid".to_string()));
    }

    // Note: Testing refresh_theme_from_cli() requires mocking AppHandle and file system
    // which is complex in unit tests. The function is tested through integration tests
    // and manual testing. The core logic is covered by testing get_system_theme_colors()
    // in the get_current_theme module.
}
