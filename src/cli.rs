use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(name = "omarchist")]
#[command(about = "Omarchy system and theme manager")]
#[command(version)]
pub struct CliArgs {
    #[arg(short, long, value_enum)]
    pub view: Option<ViewOption>,

    #[arg(short, long, requires = "view")]
    pub theme: Option<String>,
}

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
