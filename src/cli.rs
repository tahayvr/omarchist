//! CLI argument parsing for Omarchist
//!
//! This module provides command-line argument parsing using clap.
//! It allows users to specify initial view options when launching the app.

use clap::{Parser, ValueEnum};

/// Command-line arguments for Omarchist
#[derive(Parser, Debug, Clone)]
#[command(name = "omarchist")]
#[command(about = "Omarchy system and theme manager")]
#[command(version)]
pub struct CliArgs {
    /// Initial view to open on startup
    #[arg(short, long, value_enum)]
    pub view: Option<ViewOption>,

    /// Theme name to edit (requires --view theme)
    #[arg(short, long, requires = "view")]
    pub theme: Option<String>,
}

/// Available view options for the initial page
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewOption {
    Themes,
    System,
    Settings,
    About,
    Omarchy,
    Config,
}

impl CliArgs {
    /// Parse CLI arguments from the environment
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_system_view() {
        let args = CliArgs::parse_from(["omarchist", "--view", "system"]);
        assert_eq!(args.view, Some(ViewOption::System));
        assert_eq!(args.theme, None);
    }

    #[test]
    fn test_parse_themes_view() {
        let args = CliArgs::parse_from(["omarchist", "--view", "themes"]);
        assert_eq!(args.view, Some(ViewOption::Themes));
    }

    #[test]
    fn test_parse_with_theme() {
        let args = CliArgs::parse_from(["omarchist", "--view", "themes", "--theme", "my-theme"]);
        assert_eq!(args.view, Some(ViewOption::Themes));
        assert_eq!(args.theme, Some("my-theme".to_string()));
    }
}
