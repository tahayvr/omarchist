use std::collections::BTreeMap;

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
    /// The theme Omarchy currently runs (`current/theme.name`).
    #[serde(default)]
    pub applied: bool,
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
    #[serde(default)]
    pub colors: ColorsConfig,
    /// Palette bundles that are on, by `PaletteBundle::id`, with the colors
    /// changed for them (`colors.toml` keys to `#rrggbb`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub palettes: BTreeMap<String, BTreeMap<String, String>>,
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
            colors: ColorsConfig::default(),
            palettes: BTreeMap::new(),
            is_light_theme: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorsConfig {
    // No `cursor` key: Omarchy's resolver unconditionally derives it from
    // `bright_foreground` (color15), so a stored value would never be read.
    // Explicit dark/light marker, written into colors.toml's `mode` key —
    // Omarchy's own resolver otherwise has to fall back to a `light.mode`
    // marker file or background-luminance auto-detection, and Omarchist
    // already tracks this precisely via `EditingTheme::is_light_theme`.
    #[serde(default = "default_mode")]
    pub mode: String,
    pub accent: String,
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
    // Optional Hyprland border overrides. These are the exact keys Quattro's
    // `default/themed/hyprland.lua.tpl` reads (`{{ hypr_gradient
    // hyprland_active_border accent }}`), so they accept any Hyprland color
    // spec including gradients, e.g. `rgba(26a269ee) rgba(2ec27eee) 45deg`.
    // Absent means Omarchy's default: accent for active, neutral grey inactive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyprland_active_border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyprland_inactive_border: Option<String>,
    /// Keys in `colors.toml` that Omarchist does not edit (a user-added
    /// `orange`, say), kept in order so a save never drops them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String)>,
}

fn default_mode() -> String {
    "dark".to_string()
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            accent: "#33A1FF".to_string(),
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
            hyprland_active_border: None,
            hyprland_inactive_border: None,
            extra: Vec::new(),
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
