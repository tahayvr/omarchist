use serde::{Deserialize, Serialize};

/// Represents the active tab in the themes page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeTab {
    System,
    Custom,
}

/// System theme data structure from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysTheme {
    pub dir: String,
    pub title: String,
    pub description: String,
    pub image: String, // Absolute file path or empty
    pub is_system: bool,
    pub is_custom: bool,
    pub colors: Option<ThemeColors>,
}

/// Custom theme data structure from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    pub name: String,
    pub image: String,
    pub created_at: String,
    pub modified_at: String,
    pub author: Option<String>,
    pub apps: serde_json::Value,
    pub colors: Option<ThemeColors>,
}

/// Theme colors structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: PrimaryColors,
    pub terminal: TerminalColors,
}

/// Primary colors (background and foreground)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryColors {
    pub background: String, // Hex color
    pub foreground: String, // Hex color
}

/// Terminal colors (8 standard colors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColors {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

/// Unified theme representation for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeData {
    System(SysTheme),
    Custom(CustomTheme),
}

impl ThemeData {
    pub fn dir(&self) -> &str {
        match self {
            ThemeData::System(theme) => &theme.dir,
            ThemeData::Custom(theme) => &theme.name,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            ThemeData::System(theme) => &theme.title,
            ThemeData::Custom(theme) => &theme.name,
        }
    }

    pub fn image(&self) -> Option<&str> {
        match self {
            ThemeData::System(theme) => {
                if theme.image.is_empty() {
                    None
                } else {
                    Some(&theme.image)
                }
            }
            ThemeData::Custom(theme) => {
                if theme.image.is_empty() {
                    None
                } else {
                    Some(&theme.image)
                }
            }
        }
    }

    pub fn colors(&self) -> Option<&ThemeColors> {
        match self {
            ThemeData::System(theme) => theme.colors.as_ref(),
            ThemeData::Custom(theme) => theme.colors.as_ref(),
        }
    }

    pub fn is_system(&self) -> bool {
        match self {
            ThemeData::System(theme) => theme.is_system,
            ThemeData::Custom(_) => false,
        }
    }

    pub fn is_custom(&self) -> bool {
        match self {
            ThemeData::System(theme) => theme.is_custom,
            ThemeData::Custom(_) => true,
        }
    }

    pub fn badge_text(&self) -> &str {
        match self {
            ThemeData::System(theme) => {
                if theme.is_system {
                    "System"
                } else if theme.is_custom {
                    "Custom"
                } else {
                    "Community"
                }
            }
            ThemeData::Custom(_) => "Custom",
        }
    }
}

/// Structure for editing a theme - matches custom_theme.json format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditingTheme {
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
    pub author: Option<String>,
    pub apps: AppConfigs,
    #[serde(skip)] // Runtime-only, not serialized to JSON
    pub is_light_theme: bool,
}

impl Default for EditingTheme {
    fn default() -> Self {
        Self {
            name: String::new(),
            created_at: String::new(),
            modified_at: String::new(),
            author: None,
            apps: AppConfigs::default(),
            is_light_theme: false, // Dark theme by default
        }
    }
}

/// Waybar configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaybarConfig {
    pub background: String, // Hex color
    pub foreground: String, // Hex color
}

impl Default for WaybarConfig {
    fn default() -> Self {
        Self {
            background: "#0F0F19".to_string(),
            foreground: "#EDEDFE".to_string(),
        }
    }
}

/// Hyprland window configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlandConfig {
    pub active_border: String,   // Hex color (without #, e.g., "6e6e92")
    pub inactive_border: String, // Hex color (without #, e.g., "5C5C5E")
    pub border_size: i32,
    pub gaps_in: i32,
    pub gaps_out: i32,
    pub rounding: i32,
}

impl Default for HyprlandConfig {
    fn default() -> Self {
        Self {
            active_border: "6e6e92".to_string(),
            inactive_border: "5C5C5E".to_string(),
            border_size: 1,
            gaps_in: 5,
            gaps_out: 10,
            rounding: 0,
        }
    }
}

/// All app configurations for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfigs {
    #[serde(rename = "alacritty")]
    pub alacritty: Option<serde_json::Value>,
    #[serde(rename = "waybar")]
    pub waybar: Option<WaybarConfig>,
    #[serde(rename = "chromium")]
    pub chromium: Option<serde_json::Value>,
    #[serde(rename = "btop")]
    pub btop: Option<serde_json::Value>,
    #[serde(rename = "hyprland")]
    pub hyprland: Option<HyprlandConfig>,
    #[serde(rename = "hyprlock")]
    pub hyprlock: Option<serde_json::Value>,
    #[serde(rename = "mako")]
    pub mako: Option<serde_json::Value>,
    #[serde(rename = "walker")]
    pub walker: Option<serde_json::Value>,
    #[serde(rename = "swayosd")]
    pub swayosd: Option<serde_json::Value>,
    #[serde(rename = "neovim")]
    pub neovim: Option<serde_json::Value>,
    #[serde(rename = "vscode")]
    pub vscode: Option<serde_json::Value>,
    #[serde(rename = "icons")]
    pub icons: Option<serde_json::Value>,
    #[serde(rename = "ghostty")]
    pub ghostty: Option<serde_json::Value>,
    #[serde(rename = "kitty")]
    pub kitty: Option<serde_json::Value>,
}

impl Default for AppConfigs {
    fn default() -> Self {
        Self {
            alacritty: None,
            waybar: None,
            chromium: None,
            btop: None,
            hyprland: None,
            hyprlock: None,
            mako: None,
            walker: None,
            swayosd: None,
            neovim: None,
            vscode: None,
            icons: None,
            ghostty: None,
            kitty: None,
        }
    }
}

/// Tab identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeEditTab {
    General,
    Waybar,
    Windows,
    Menu,
    Terminal,
    Browser,
    FileManager,
    LockScreen,
    Notification,
    Editor,
    Btop,
    Swayosd,
    Backgrounds,
}

impl ThemeEditTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeEditTab::General => "General",
            ThemeEditTab::Waybar => "Waybar",
            ThemeEditTab::Windows => "Windows",
            ThemeEditTab::Menu => "Menu",
            ThemeEditTab::Terminal => "Terminal",
            ThemeEditTab::Browser => "Browser",
            ThemeEditTab::FileManager => "File Manager",
            ThemeEditTab::LockScreen => "Lock Screen",
            ThemeEditTab::Notification => "Notification",
            ThemeEditTab::Editor => "Editor",
            ThemeEditTab::Btop => "Btop",
            ThemeEditTab::Swayosd => "SwayOSD",
            ThemeEditTab::Backgrounds => "Backgrounds",
        }
    }

    pub fn all() -> Vec<ThemeEditTab> {
        vec![
            ThemeEditTab::General,
            ThemeEditTab::Waybar,
            ThemeEditTab::Windows,
            ThemeEditTab::Menu,
            ThemeEditTab::Terminal,
            ThemeEditTab::Browser,
            ThemeEditTab::FileManager,
            ThemeEditTab::LockScreen,
            ThemeEditTab::Notification,
            ThemeEditTab::Editor,
            ThemeEditTab::Btop,
            ThemeEditTab::Swayosd,
            ThemeEditTab::Backgrounds,
        ]
    }
}
