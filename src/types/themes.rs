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

/// Walker menu configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkerConfig {
    pub background: String,    // Hex color
    pub base: String,          // Hex color
    pub border: String,        // Hex color
    pub foreground: String,    // Hex color
    pub text: String,          // Hex color
    pub selected_text: String, // Hex color
}

impl Default for WalkerConfig {
    fn default() -> Self {
        Self {
            background: "#0F0F19".to_string(),
            base: "#0F0F19".to_string(),
            border: "#33A1FF".to_string(),
            foreground: "#EDEDFE".to_string(),
            text: "#EDEDFE".to_string(),
            selected_text: "#FF66F6".to_string(),
        }
    }
}

/// Browser (Chromium) configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub theme_color: String, // Hex color for browser theme
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            theme_color: "#0F0F19".to_string(),
        }
    }
}

/// Hyprlock configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlockConfig {
    pub color: String,       // Hex color (rgb format: 0f0f19)
    pub inner_color: String, // Hex color (rgb format: 0f0f19)
    pub outer_color: String, // Hex color (rgb format: 33a0ff)
    pub font_color: String,  // Hex color (rgb format: ff66f5)
    pub check_color: String, // Hex color (rgb format: ffea00)
}

impl Default for HyprlockConfig {
    fn default() -> Self {
        Self {
            color: "0f0f19".to_string(),
            inner_color: "0f0f19".to_string(),
            outer_color: "33a0ff".to_string(),
            font_color: "ff66f5".to_string(),
            check_color: "ffea00".to_string(),
        }
    }
}

/// Mako notification configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakoConfig {
    pub text_color: String,       // Hex color (#EDEDFE)
    pub border_color: String,     // Hex color (#00F59B)
    pub background_color: String, // Hex color (#0F0F19)
}

impl Default for MakoConfig {
    fn default() -> Self {
        Self {
            text_color: "#EDEDFE".to_string(),
            border_color: "#00F59B".to_string(),
            background_color: "#0F0F19".to_string(),
        }
    }
}

/// SwayOSD configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwayosdConfig {
    pub background_color: String, // Hex color (#0F0F19)
    pub border_color: String,     // Hex color (#33A1FF)
    pub label: String,            // Hex color (#8A8A8D)
    pub image: String,            // Hex color (#8A8A8D)
    pub progress: String,         // Hex color (#8A8A8D)
}

impl Default for SwayosdConfig {
    fn default() -> Self {
        Self {
            background_color: "#0F0F19".to_string(),
            border_color: "#33A1FF".to_string(),
            label: "#8A8A8D".to_string(),
            image: "#8A8A8D".to_string(),
            progress: "#8A8A8D".to_string(),
        }
    }
}

/// Btop configuration structure for activity monitor colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtopConfig {
    // Main colors
    pub main_bg: String,     // Main background
    pub main_fg: String,     // Main text color
    pub title: String,       // Title color for boxes
    pub hi_fg: String,       // Highlight color for keyboard shortcuts
    pub selected_bg: String, // Background of selected item in processes
    pub selected_fg: String, // Foreground of selected item in processes
    pub inactive_fg: String, // Color of inactive/disabled text
    pub proc_misc: String,   // Misc colors for processes box
    // Box outline colors
    pub cpu_box: String,  // Cpu box outline color
    pub mem_box: String,  // Memory/disks box outline color
    pub net_box: String,  // Net up/down box outline color
    pub proc_box: String, // Processes box outline color
    pub div_line: String, // Box divider line color
    // Gradient colors - Temperature
    pub temp_start: String,
    pub temp_mid: String,
    pub temp_end: String,
    // Gradient colors - CPU
    pub cpu_start: String,
    pub cpu_mid: String,
    pub cpu_end: String,
    // Gradient colors - Free meter
    pub free_start: String,
    pub free_mid: String,
    pub free_end: String,
    // Gradient colors - Cached meter
    pub cached_start: String,
    pub cached_mid: String,
    pub cached_end: String,
    // Gradient colors - Available meter
    pub available_start: String,
    pub available_mid: String,
    pub available_end: String,
    // Gradient colors - Used meter
    pub used_start: String,
    pub used_mid: String,
    pub used_end: String,
    // Gradient colors - Download
    pub download_start: String,
    pub download_mid: String,
    pub download_end: String,
    // Gradient colors - Upload
    pub upload_start: String,
    pub upload_mid: String,
    pub upload_end: String,
}

impl Default for BtopConfig {
    fn default() -> Self {
        Self {
            main_bg: "#0F0F19".to_string(),
            main_fg: "#EDEDFE".to_string(),
            title: "#6e6e92".to_string(),
            hi_fg: "#33A1FF".to_string(),
            selected_bg: "#f59e0b".to_string(),
            selected_fg: "#EDEDFE".to_string(),
            inactive_fg: "#333333".to_string(),
            proc_misc: "#8a8a8d".to_string(),
            cpu_box: "#6e6e92".to_string(),
            mem_box: "#6e6e92".to_string(),
            net_box: "#6e6e92".to_string(),
            proc_box: "#6e6e92".to_string(),
            div_line: "#6e6e92".to_string(),
            temp_start: "#00F59B".to_string(),
            temp_mid: "#FF66F6".to_string(),
            temp_end: "#FF3366".to_string(),
            cpu_start: "#00F59B".to_string(),
            cpu_mid: "#FF66F6".to_string(),
            cpu_end: "#FF3366".to_string(),
            free_start: "#00F59B".to_string(),
            free_mid: "#FF66F6".to_string(),
            free_end: "#FF3366".to_string(),
            cached_start: "#00F59B".to_string(),
            cached_mid: "#FF66F6".to_string(),
            cached_end: "#FF3366".to_string(),
            available_start: "#00F59B".to_string(),
            available_mid: "#FF66F6".to_string(),
            available_end: "#FF3366".to_string(),
            used_start: "#00F59B".to_string(),
            used_mid: "#FF66F6".to_string(),
            used_end: "#FF3366".to_string(),
            download_start: "#00F59B".to_string(),
            download_mid: "#FF66F6".to_string(),
            download_end: "#FF3366".to_string(),
            upload_start: "#00F59B".to_string(),
            upload_mid: "#FF66F6".to_string(),
            upload_end: "#FF3366".to_string(),
        }
    }
}

/// Terminal color palette (8 standard ANSI colors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPalette {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

impl Default for TerminalPalette {
    fn default() -> Self {
        Self {
            black: "#0A0A12".to_string(),
            red: "#FF3366".to_string(),
            green: "#00F59B".to_string(),
            yellow: "#FFEA00".to_string(),
            blue: "#33A1FF".to_string(),
            magenta: "#FF66F6".to_string(),
            cyan: "#3CFFED".to_string(),
            white: "#EDEDFE".to_string(),
        }
    }
}

/// Terminal cursor colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCursor {
    pub cursor: String,
    pub text: String,
}

impl Default for TerminalCursor {
    fn default() -> Self {
        Self {
            cursor: "#EDEDFE".to_string(),
            text: "#0F0F19".to_string(),
        }
    }
}

/// Terminal selection colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSelection {
    pub background: String,
    pub foreground: String,
}

impl Default for TerminalSelection {
    fn default() -> Self {
        Self {
            background: "#202034".to_string(),
            foreground: "#EDEDFE".to_string(),
        }
    }
}

/// Terminal primary colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPrimary {
    pub background: String,
    pub foreground: String,
}

impl Default for TerminalPrimary {
    fn default() -> Self {
        Self {
            background: "#0F0F19".to_string(),
            foreground: "#EDEDFE".to_string(),
        }
    }
}

/// Unified terminal configuration for all terminal emulators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    #[serde(default)]
    pub primary: TerminalPrimary,
    #[serde(default)]
    pub cursor: TerminalCursor,
    #[serde(default)]
    pub selection: TerminalSelection,
    #[serde(default)]
    pub normal: TerminalPalette,
    #[serde(default)]
    pub bright: TerminalPalette,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            primary: TerminalPrimary::default(),
            cursor: TerminalCursor::default(),
            selection: TerminalSelection::default(),
            normal: TerminalPalette::default(),
            bright: TerminalPalette {
                black: "#181824".to_string(),
                red: "#FF9A8F".to_string(),
                green: "#57F8BD".to_string(),
                yellow: "#FFFF80".to_string(),
                blue: "#5A9EFF".to_string(),
                magenta: "#FF99FF".to_string(),
                cyan: "#80FFFF".to_string(),
                white: "#F8F8FF".to_string(),
            },
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
    pub chromium: Option<BrowserConfig>,
    #[serde(rename = "btop")]
    pub btop: Option<BtopConfig>,
    #[serde(rename = "hyprland")]
    pub hyprland: Option<HyprlandConfig>,
    #[serde(rename = "hyprlock")]
    pub hyprlock: Option<HyprlockConfig>,
    #[serde(rename = "mako")]
    pub mako: Option<MakoConfig>,
    #[serde(rename = "walker")]
    pub walker: Option<WalkerConfig>,
    #[serde(rename = "swayosd")]
    pub swayosd: Option<SwayosdConfig>,
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
    #[serde(rename = "terminal")]
    pub terminal: Option<TerminalConfig>,
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
            terminal: None,
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
