#[cfg(test)]
mod tests {
    use crate::types::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.auto_apply_theme, true);
    }

    #[test]
    fn test_app_settings_serialization() {
        let settings = AppSettings {
            auto_apply_theme: false,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_theme_colors_serialization() {
        let colors = ThemeColors {
            primary: PrimaryColors {
                background: "#000000".to_string(),
                foreground: "#ffffff".to_string(),
            },
            terminal: TerminalColors {
                red: "#ff0000".to_string(),
                green: "#00ff00".to_string(),
                yellow: "#ffff00".to_string(),
                blue: "#0000ff".to_string(),
                magenta: "#ff00ff".to_string(),
                cyan: "#00ffff".to_string(),
            },
        };

        let json = serde_json::to_string(&colors).unwrap();
        let deserialized: ThemeColors = serde_json::from_str(&json).unwrap();

        assert_eq!(colors.primary.background, deserialized.primary.background);
        assert_eq!(colors.terminal.red, deserialized.terminal.red);
    }

    #[test]
    fn test_startup_cli_result() {
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
    fn test_startup_command_equality() {
        assert_eq!(StartupCommand::Normal, StartupCommand::Normal);
        assert_eq!(StartupCommand::Refresh, StartupCommand::Refresh);
        assert_eq!(
            StartupCommand::Unknown("test".to_string()),
            StartupCommand::Unknown("test".to_string())
        );

        assert_ne!(StartupCommand::Normal, StartupCommand::Refresh);
    }

    #[test]
    fn test_error_types() {
        let theme_error = ThemeError::NotFound("test".to_string());
        let app_error = AppError::Theme(theme_error);

        assert!(matches!(app_error, AppError::Theme(_)));
    }

    #[test]
    fn test_custom_theme_serialization() {
        let theme = CustomTheme {
            name: "test".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            modified_at: "2023-01-01T00:00:00Z".to_string(),
            apps: serde_json::json!({}),
            colors: None,
        };

        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: CustomTheme = serde_json::from_str(&json).unwrap();

        assert_eq!(theme.name, deserialized.name);
    }
}
