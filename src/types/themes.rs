use serde::{Deserialize, Serialize};

// Where a theme comes from determines what actions are available on it.
//
// - `System`    — shipped with Omarchy, read-only (`~/.local/share/omarchy/themes/`)
// - `Omarchist` — created with this app, fully editable (`~/.config/omarchy/themes/`, has `omarchist.json`)
// - `Community` — installed by the user from an external source, not editable here (`~/.config/omarchy/themes/`, no `omarchist.json`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeOrigin {
    System,
    Omarchist,
    Community,
}

impl ThemeOrigin {
    pub fn badge_text(&self) -> &'static str {
        match self {
            ThemeOrigin::System => "System",
            ThemeOrigin::Omarchist => "Omarchist",
            ThemeOrigin::Community => "Community",
        }
    }

    pub fn is_editable(&self) -> bool {
        matches!(self, ThemeOrigin::Omarchist)
    }

    pub fn is_deletable(&self) -> bool {
        matches!(self, ThemeOrigin::Omarchist | ThemeOrigin::Community)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeEntry {
    pub dir: String,
    pub title: String,
    pub origin: ThemeOrigin,
    pub image: String,
    pub colors: Option<ThemeColors>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawUserTheme {
    #[serde(default = "default_version")]
    pub version: String,
    pub name: String,
    pub image: String,
    pub origin: ThemeOrigin,
    pub created_at: String,
    pub modified_at: String,
    pub author: Option<String>,
    pub apps: serde_json::Value,
    pub colors: Option<ThemeColors>,
}

impl RawUserTheme {
    pub fn into_entry(self, title: String) -> ThemeEntry {
        ThemeEntry {
            dir: self.name,
            title,
            origin: self.origin,
            image: self.image,
            colors: self.colors,
        }
    }
}

fn default_version() -> String {
    "1.0.0".to_string()
}

// Theme colors structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: PrimaryColors,
    pub terminal: TerminalColors,
}

// Hex color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryColors {
    pub background: String,
    pub foreground: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditingTheme {
    #[serde(default = "default_version")]
    pub version: String,
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
    pub author: Option<String>,
    pub apps: AppConfigs,
    #[serde(default)]
    pub colors: ColorsConfig,
    #[serde(skip)] // Runtime-only, not serialized to JSON
    pub is_light_theme: bool,
}

impl Default for EditingTheme {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            name: String::new(),
            created_at: String::new(),
            modified_at: String::new(),
            author: None,
            apps: AppConfigs::default(),
            colors: ColorsConfig::default(),
            is_light_theme: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorsConfig {
    pub accent: String,
    pub cursor: String,
    pub foreground: String,
    pub background: String,
    pub selection_foreground: String,
    pub selection_background: String,
    pub color0: String,
    pub color1: String,
    pub color2: String,
    pub color3: String,
    pub color4: String,
    pub color5: String,
    pub color6: String,
    pub color7: String,
    pub color8: String,
    pub color9: String,
    pub color10: String,
    pub color11: String,
    pub color12: String,
    pub color13: String,
    pub color14: String,
    pub color15: String,
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            accent: "#33A1FF".to_string(),
            cursor: "#EDEDFE".to_string(),
            foreground: "#EDEDFE".to_string(),
            background: "#0F0F19".to_string(),
            selection_foreground: "#EDEDFE".to_string(),
            selection_background: "#202034".to_string(),
            color0: "#0A0A12".to_string(),
            color1: "#FF3366".to_string(),
            color2: "#00F59B".to_string(),
            color3: "#FFEA00".to_string(),
            color4: "#33A1FF".to_string(),
            color5: "#FF66F6".to_string(),
            color6: "#3CFFED".to_string(),
            color7: "#EDEDFE".to_string(),
            color8: "#181824".to_string(),
            color9: "#ff9a8f".to_string(),
            color10: "#57f8bd".to_string(),
            color11: "#ffff80".to_string(),
            color12: "#5a9eff".to_string(),
            color13: "#ff99ff".to_string(),
            color14: "#80ffff".to_string(),
            color15: "#F8F8FF".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaybarConfig {
    pub background: String,
    pub foreground: String,
}

impl Default for WaybarConfig {
    fn default() -> Self {
        Self {
            background: "#0F0F19".to_string(),
            foreground: "#EDEDFE".to_string(),
        }
    }
}

// Hyprland window configuration structure
// Hex color (without #, e.g., "6e6e92")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlandConfig {
    pub active_border: String,
    pub inactive_border: String,
}

impl Default for HyprlandConfig {
    fn default() -> Self {
        Self {
            active_border: "6e6e92".to_string(),
            inactive_border: "5C5C5E".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkerConfig {
    pub background: String,
    pub base: String,
    pub border: String,
    pub foreground: String,
    pub text: String,
    pub selected_text: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub theme_color: String,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            theme_color: "#0F0F19".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlockConfig {
    pub color: String,
    pub inner_color: String,
    pub outer_color: String,
    pub font_color: String,
    pub check_color: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakoConfig {
    pub text_color: String,
    pub border_color: String,
    pub background_color: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwayosdConfig {
    pub background_color: String,
    pub border_color: String,
    pub label: String,
    pub image: String,
    pub progress: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtopConfig {
    pub main_bg: String,
    pub main_fg: String,
    pub title: String,
    pub hi_fg: String,
    pub selected_bg: String,
    pub selected_fg: String,
    pub inactive_fg: String,
    pub proc_misc: String,
    pub cpu_box: String,
    pub mem_box: String,
    pub net_box: String,
    pub proc_box: String,
    pub div_line: String,
    pub temp_start: String,
    pub temp_mid: String,
    pub temp_end: String,
    pub cpu_start: String,
    pub cpu_mid: String,
    pub cpu_end: String,
    pub free_start: String,
    pub free_mid: String,
    pub free_end: String,
    pub cached_start: String,
    pub cached_mid: String,
    pub cached_end: String,
    pub available_start: String,
    pub available_mid: String,
    pub available_end: String,
    pub used_start: String,
    pub used_mid: String,
    pub used_end: String,
    pub download_start: String,
    pub download_mid: String,
    pub download_end: String,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfigs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alacritty: Option<serde_json::Value>,
    pub waybar: Option<WaybarConfig>,
    pub chromium: Option<BrowserConfig>,
    pub btop: Option<BtopConfig>,
    pub hyprland: Option<HyprlandConfig>,
    pub hyprlock: Option<HyprlockConfig>,
    pub mako: Option<MakoConfig>,
    pub walker: Option<WalkerConfig>,
    pub swayosd: Option<SwayosdConfig>,
    pub neovim: Option<serde_json::Value>,
    pub vscode: Option<serde_json::Value>,
    pub icons: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghostty: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kitty: Option<serde_json::Value>,
    pub terminal: Option<TerminalConfig>,
}

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
