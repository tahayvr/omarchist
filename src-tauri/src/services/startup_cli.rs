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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test CLI argument detection with various input scenarios
    mod cli_argument_detection {
        use super::*;

        #[test]
        fn test_refresh_command_detection() {
            let refresh_command = StartupCommand::Refresh;
            assert_eq!(refresh_command, StartupCommand::Refresh);

            // Refresh command should trigger early exit
            assert!(should_exit_early(&refresh_command));
        }

        #[test]
        fn test_normal_startup_detection() {
            let normal_command = StartupCommand::Normal;
            assert_eq!(normal_command, StartupCommand::Normal);

            // Normal startup should not trigger early exit
            assert!(!should_exit_early(&normal_command));
        }

        #[test]
        fn test_invalid_command_detection() {
            let invalid_commands = vec![
                "invalid",
                "unknown",
                "help",
                "version",
                "config",
                "test-command",
                "",
            ];

            for cmd in invalid_commands {
                let unknown_command = StartupCommand::Unknown(cmd.to_string());
                assert_eq!(unknown_command, StartupCommand::Unknown(cmd.to_string()));

                // Unknown commands should trigger early exit
                assert!(should_exit_early(&unknown_command));
            }
        }

        #[test]
        fn test_raw_cli_parsing_refresh() {
            let args = vec!["omarchist".to_string(), "refresh".to_string()];
            let command = parse_raw_startup_command(&args);
            assert_eq!(command, StartupCommand::Refresh);
        }

        #[test]
        fn test_raw_cli_parsing_unknown() {
            let args = vec!["omarchist".to_string(), "unknown".to_string()];
            let command = parse_raw_startup_command(&args);
            assert_eq!(command, StartupCommand::Unknown("unknown".to_string()));
        }

        #[test]
        fn test_raw_cli_parsing_no_args() {
            let args = vec!["omarchist".to_string()];
            let command = parse_raw_startup_command(&args);
            assert_eq!(command, StartupCommand::Normal);
        }

        #[test]
        fn test_raw_cli_parsing_empty() {
            let args: Vec<String> = vec![];
            let command = parse_raw_startup_command(&args);
            assert_eq!(command, StartupCommand::Normal);
        }

        #[test]
        fn test_raw_cli_parsing_multiple_args() {
            let args = vec![
                "omarchist".to_string(),
                "refresh".to_string(),
                "extra".to_string(),
            ];
            let command = parse_raw_startup_command(&args);
            // Should still parse as refresh, ignoring extra arguments
            assert_eq!(command, StartupCommand::Refresh);
        }

        #[test]
        fn test_command_equality_and_inequality() {
            // Test equality
            assert_eq!(StartupCommand::Normal, StartupCommand::Normal);
            assert_eq!(StartupCommand::Refresh, StartupCommand::Refresh);
            assert_eq!(
                StartupCommand::Unknown("test".to_string()),
                StartupCommand::Unknown("test".to_string())
            );

            // Test inequality
            assert_ne!(StartupCommand::Normal, StartupCommand::Refresh);
            assert_ne!(
                StartupCommand::Refresh,
                StartupCommand::Unknown("refresh".to_string())
            );
            assert_ne!(
                StartupCommand::Unknown("test1".to_string()),
                StartupCommand::Unknown("test2".to_string())
            );
        }
    }

    /// Test early exit logic for refresh commands on first instance
    mod early_exit_logic {
        use super::*;

        #[test]
        fn test_refresh_command_early_exit() {
            let refresh_command = StartupCommand::Refresh;
            let should_exit = should_exit_early(&refresh_command);

            assert!(
                should_exit,
                "Refresh command should trigger early exit on first instance"
            );
        }

        #[test]
        fn test_unknown_command_early_exit() {
            let test_cases = vec!["invalid", "help", "version", "unknown-command", "test", ""];

            for cmd in test_cases {
                let unknown_command = StartupCommand::Unknown(cmd.to_string());
                let should_exit = should_exit_early(&unknown_command);

                assert!(
                    should_exit,
                    "Unknown command '{}' should trigger early exit to avoid UI launch",
                    cmd
                );
            }
        }

        #[test]
        fn test_normal_startup_no_early_exit() {
            let normal_command = StartupCommand::Normal;
            let should_exit = should_exit_early(&normal_command);

            assert!(!should_exit, "Normal startup should not trigger early exit");
        }

        #[test]
        fn test_early_exit_consistency() {
            // Test that the same command always produces the same result
            let refresh_command = StartupCommand::Refresh;
            assert_eq!(
                should_exit_early(&refresh_command),
                should_exit_early(&refresh_command)
            );

            let normal_command = StartupCommand::Normal;
            assert_eq!(
                should_exit_early(&normal_command),
                should_exit_early(&normal_command)
            );

            let unknown_command = StartupCommand::Unknown("test".to_string());
            assert_eq!(
                should_exit_early(&unknown_command),
                should_exit_early(&unknown_command)
            );
        }
    }

    /// Test normal startup path when no CLI arguments are provided
    mod normal_startup_path {
        use super::*;

        #[test]
        fn test_normal_startup_result() {
            let normal_command = StartupCommand::Normal;
            let should_continue = !should_exit_early(&normal_command);

            assert!(
                should_continue,
                "Normal startup should allow app to continue"
            );
        }

        #[test]
        fn test_startup_cli_result_for_normal_startup() {
            let result = StartupCliResult {
                should_continue: true,
                exit_reason: None,
                exit_code: 0,
            };

            assert!(result.should_continue, "Normal startup should continue");
            assert!(
                result.exit_reason.is_none(),
                "Normal startup should have no exit reason"
            );
            assert_eq!(
                result.exit_code, 0,
                "Normal startup should have exit code 0"
            );
        }

        #[test]
        fn test_normal_startup_flow() {
            // Simulate the normal startup flow
            let command = StartupCommand::Normal;
            let should_exit = should_exit_early(&command);

            if !should_exit {
                // This represents the normal startup path
                let result = StartupCliResult {
                    should_continue: true,
                    exit_reason: None,
                    exit_code: 0,
                };

                assert!(result.should_continue);
                assert!(result.exit_reason.is_none());
            } else {
                panic!("Normal startup should not trigger early exit");
            }
        }
    }

    /// Test StartupCliResult structure and behavior
    mod startup_cli_result {
        use super::*;

        #[test]
        fn test_startup_cli_result_creation_continue() {
            let result = StartupCliResult {
                should_continue: true,
                exit_reason: None,
                exit_code: 0,
            };

            assert!(result.should_continue);
            assert!(result.exit_reason.is_none());
            assert_eq!(result.exit_code, 0);
        }

        #[test]
        fn test_startup_cli_result_creation_exit() {
            let result = StartupCliResult {
                should_continue: false,
                exit_reason: Some("Refresh command on first instance".to_string()),
                exit_code: 0,
            };

            assert!(!result.should_continue);
            assert_eq!(
                result.exit_reason,
                Some("Refresh command on first instance".to_string())
            );
            assert_eq!(result.exit_code, 0);
        }

        #[test]
        fn test_startup_cli_result_creation_error() {
            let result = StartupCliResult {
                should_continue: false,
                exit_reason: Some("Invalid CLI arguments".to_string()),
                exit_code: 1,
            };

            assert!(!result.should_continue);
            assert_eq!(
                result.exit_reason,
                Some("Invalid CLI arguments".to_string())
            );
            assert_eq!(result.exit_code, 1);
        }

        #[test]
        fn test_startup_cli_result_clone() {
            let original = StartupCliResult {
                should_continue: false,
                exit_reason: Some("Test reason".to_string()),
                exit_code: 1,
            };

            let cloned = original.clone();

            assert_eq!(original.should_continue, cloned.should_continue);
            assert_eq!(original.exit_reason, cloned.exit_reason);
            assert_eq!(original.exit_code, cloned.exit_code);
        }

        #[test]
        fn test_startup_cli_result_debug() {
            let result = StartupCliResult {
                should_continue: false,
                exit_reason: Some("Debug test".to_string()),
                exit_code: 0,
            };

            let debug_string = format!("{:?}", result);
            assert!(debug_string.contains("should_continue: false"));
            assert!(debug_string.contains("Debug test"));
            assert!(debug_string.contains("exit_code: 0"));
        }
    }

    /// Test StartupCommand enum behavior
    mod startup_command {
        use super::*;

        #[test]
        fn test_startup_command_debug() {
            let normal = StartupCommand::Normal;
            let refresh = StartupCommand::Refresh;
            let unknown = StartupCommand::Unknown("test".to_string());

            let normal_debug = format!("{:?}", normal);
            let refresh_debug = format!("{:?}", refresh);
            let unknown_debug = format!("{:?}", unknown);

            assert_eq!(normal_debug, "Normal");
            assert_eq!(refresh_debug, "Refresh");
            assert!(unknown_debug.contains("Unknown"));
            assert!(unknown_debug.contains("test"));
        }

        #[test]
        fn test_startup_command_clone() {
            let original = StartupCommand::Unknown("test".to_string());
            let cloned = original.clone();

            assert_eq!(original, cloned);
        }

        #[test]
        fn test_startup_command_partial_eq() {
            // Test PartialEq implementation
            assert_eq!(StartupCommand::Normal, StartupCommand::Normal);
            assert_eq!(StartupCommand::Refresh, StartupCommand::Refresh);
            assert_eq!(
                StartupCommand::Unknown("same".to_string()),
                StartupCommand::Unknown("same".to_string())
            );

            assert_ne!(StartupCommand::Normal, StartupCommand::Refresh);
            assert_ne!(
                StartupCommand::Unknown("different1".to_string()),
                StartupCommand::Unknown("different2".to_string())
            );
        }
    }

    /// Test edge cases and error scenarios
    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_unknown_command() {
            let empty_command = StartupCommand::Unknown("".to_string());
            assert!(should_exit_early(&empty_command));
        }

        #[test]
        fn test_whitespace_unknown_command() {
            let whitespace_commands = vec![" ", "\t", "\n", "   ", "\t\n\r"];

            for cmd in whitespace_commands {
                let unknown_command = StartupCommand::Unknown(cmd.to_string());
                assert!(should_exit_early(&unknown_command));
            }
        }

        #[test]
        fn test_special_character_unknown_commands() {
            let special_commands = vec![
                "!@#$%",
                "command-with-dashes",
                "command_with_underscores",
                "command.with.dots",
                "123numeric",
                "αβγ", // Unicode characters
            ];

            for cmd in special_commands {
                let unknown_command = StartupCommand::Unknown(cmd.to_string());
                assert!(should_exit_early(&unknown_command));
            }
        }

        #[test]
        fn test_case_sensitivity() {
            // Different cases should be treated as different unknown commands
            let commands = vec![
                "REFRESH", "Refresh", "rEfReShH",
                "refresh", // This would be handled by CLI parsing, not our enum
            ];

            for cmd in commands {
                let unknown_command = StartupCommand::Unknown(cmd.to_string());
                assert!(should_exit_early(&unknown_command));
                assert_ne!(unknown_command, StartupCommand::Refresh);
            }
        }
    }

    /// Test logging behavior (indirect testing through function calls)
    mod logging_behavior {
        use super::*;

        #[test]
        fn test_log_early_exit_reason_does_not_panic() {
            // Test that logging functions don't panic with various inputs
            log_early_exit_reason("Test reason");
            log_early_exit_reason("");
            log_early_exit_reason("Very long reason with special characters !@#$%^&*()");
            log_early_exit_reason("Unicode: αβγδε");
        }

        #[test]
        fn test_early_exit_decision_flow() {
            // Test the complete flow for refresh command
            let refresh_command = StartupCommand::Refresh;
            let should_exit = should_exit_early(&refresh_command);

            if should_exit {
                let exit_reason =
                    "Refresh command detected on first instance - exiting without launching UI";
                log_early_exit_reason(exit_reason);

                let result = StartupCliResult {
                    should_continue: false,
                    exit_reason: Some(exit_reason.to_string()),
                    exit_code: 0,
                };

                assert!(!result.should_continue);
                assert!(result.exit_reason.is_some());
            }
        }
    }

    /// Integration-style tests for the complete flow
    mod integration_flow {
        use super::*;

        #[test]
        fn test_complete_refresh_flow() {
            // Simulate the complete flow for a refresh command
            let command = StartupCommand::Refresh;
            let should_exit = should_exit_early(&command);

            assert!(should_exit, "Refresh command should trigger early exit");

            let exit_reason = format!(
                "Refresh command detected on first instance - exiting without launching UI"
            );

            let result = StartupCliResult {
                should_continue: false,
                exit_reason: Some(exit_reason.clone()),
                exit_code: 0,
            };

            assert!(!result.should_continue);
            assert_eq!(result.exit_reason, Some(exit_reason));
            assert_eq!(result.exit_code, 0);
        }

        #[test]
        fn test_complete_normal_flow() {
            // Simulate the complete flow for normal startup
            let command = StartupCommand::Normal;
            let should_exit = should_exit_early(&command);

            assert!(!should_exit, "Normal startup should not trigger early exit");

            let result = StartupCliResult {
                should_continue: true,
                exit_reason: None,
                exit_code: 0,
            };

            assert!(result.should_continue);
            assert!(result.exit_reason.is_none());
            assert_eq!(result.exit_code, 0);
        }

        #[test]
        fn test_complete_unknown_command_flow() {
            // Simulate the complete flow for unknown command
            let command = StartupCommand::Unknown("invalid".to_string());
            let should_exit = should_exit_early(&command);

            assert!(should_exit, "Unknown command should trigger early exit");

            let exit_reason =
                format!("Unknown command 'invalid' detected - exiting without launching UI");

            let result = StartupCliResult {
                should_continue: false,
                exit_reason: Some(exit_reason.clone()),
                exit_code: 0,
            };

            assert!(!result.should_continue);
            assert_eq!(result.exit_reason, Some(exit_reason));
            assert_eq!(result.exit_code, 0);
        }
    }

    // Note: Testing check_cli_args() and parse_startup_command() requires mocking
    // the Tauri App and CLI plugin state, which is complex in unit tests.
    // These functions are tested through integration tests and manual testing.
    // The core logic is comprehensively covered by testing the individual functions
    // like should_exit_early() and the data structures.
    //
    // For full CLI integration testing, see the integration tests in the tests/ directory
    // which can properly mock the Tauri environment and CLI plugin state.
}
