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
    "2.0.0".to_string()
}

// Theme colors structure, used for the read-only preview swatches shown in
// the theme gallery (any theme folder, not just ones Omarchist authored).
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
            version: default_version(),
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
    // Explicit dark/light marker, written into colors.toml's `mode` key —
    // Omarchy's own resolver otherwise has to fall back to a `light.mode`
    // marker file or background-luminance auto-detection, and Omarchist
    // already tracks this precisely via `EditingTheme::is_light_theme`.
    #[serde(default = "default_mode")]
    pub mode: String,
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

fn default_mode() -> String {
    "dark".to_string()
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
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

// Lock screen colors — Quattro's `shell.lock.toml`, confirmed keys and format
// against the real omacom/omarchy@quattro source (e.g. themes/tokyo-night/shell.lock.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockScreenConfig {
    pub text: String,
    pub placeholder: String,
    pub text_error: String,
    pub border: String,
    pub border_active: String,
    pub border_error: String,
}

impl Default for LockScreenConfig {
    fn default() -> Self {
        Self {
            text: "#EDEDFE".to_string(),
            placeholder: "#EDEDFE".to_string(),
            text_error: "#FF3366".to_string(),
            border: "#33A1FF".to_string(),
            border_active: "#33A1FF".to_string(),
            border_error: "#FF3366".to_string(),
        }
    }
}

// A generic 8-slot ANSI color palette, used as an intermediate value type by
// the image-based color extractor (`color_extractor.rs`) — independent of
// any specific app's config file format.
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

// A theme folder's contents beyond `colors.toml`. Terminal configs
// (alacritty.toml/foot.ini/kitty.conf/ghostty.conf), the bar, notifications,
// the launcher menu, lock screen PAM flow, and window border colors are all
// template-generated by Omarchy's own `omarchy-theme-set-templates` from
// `colors.toml` — Omarchist writes colors.toml and leaves those to Omarchy,
// rather than reimplementing its template engine. What's left are files
// Quattro's theme folder format still expects verbatim, confirmed against
// real theme folders in omacom/omarchy@quattro (btop.theme and chromium.theme
// are legitimate optional per-theme overrides, not template output).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfigs {
    pub btop: Option<BtopConfig>,
    pub chromium: Option<BrowserConfig>,
    pub lock: Option<LockScreenConfig>,
    pub neovim: Option<serde_json::Value>,
    pub vscode: Option<serde_json::Value>,
    pub icons: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeEditTab {
    General,
    Colors,
    Windows,
    Browser,
    FileManager,
    LockScreen,
    Editor,
    Btop,
    Backgrounds,
}

impl ThemeEditTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeEditTab::General => "General",
            ThemeEditTab::Colors => "Colors",
            ThemeEditTab::Windows => "Windows",
            ThemeEditTab::Browser => "Browser",
            ThemeEditTab::FileManager => "File Manager",
            ThemeEditTab::LockScreen => "Lock Screen",
            ThemeEditTab::Editor => "Editor",
            ThemeEditTab::Btop => "Btop",
            ThemeEditTab::Backgrounds => "Backgrounds",
        }
    }

    pub fn all() -> Vec<ThemeEditTab> {
        vec![
            ThemeEditTab::General,
            ThemeEditTab::Colors,
            ThemeEditTab::Windows,
            ThemeEditTab::Browser,
            ThemeEditTab::FileManager,
            ThemeEditTab::LockScreen,
            ThemeEditTab::Editor,
            ThemeEditTab::Btop,
            ThemeEditTab::Backgrounds,
        ]
    }
}
