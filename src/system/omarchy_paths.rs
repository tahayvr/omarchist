// Canonical path resolution for everything Omarchy (Quattro/v4) exposes on disk.
//
// This is the single source of truth for these paths — do not re-derive them
// elsewhere. Quattro ships as a pacman package rather than a git checkout, so
// the install root is resolved the same way Omarchy's own scripts resolve it:
// via the `OMARCHY_PATH` environment variable, falling back to
// `/usr/share/omarchy` (confirmed against `bin/omarchy-theme-set` and
// `default/hypr/bootstrap.lua` in the real omacom/omarchy@quattro source).
use std::path::PathBuf;

// The Omarchy install root: `$OMARCHY_PATH`, or `/usr/share/omarchy` for a
// standard packaged install.
pub fn omarchy_install_dir() -> PathBuf {
    std::env::var("OMARCHY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/usr/share/omarchy"))
}

// `$OMARCHY_PATH/themes` — themes shipped with Omarchy itself, read-only.
pub fn system_themes_dir() -> PathBuf {
    omarchy_install_dir().join("themes")
}

// `$OMARCHY_PATH/version` — a plain version string (e.g. `4.0.0.alpha`).
pub fn omarchy_version_file() -> PathBuf {
    omarchy_install_dir().join("version")
}

// `$OMARCHY_PATH/default/themed` — the `*.tpl` files `omarchy-theme-set-templates`
// renders into a theme that does not ship the file itself.
pub fn themed_templates_dir() -> PathBuf {
    omarchy_install_dir().join("default").join("themed")
}

// `~/.config/omarchy/themed` — the user's own templates, which win over the
// built-in ones of the same name.
pub fn user_themed_templates_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("themed"))
}

// `~/.config/omarchy/themes` — user-installed and Omarchist-created themes.
pub fn user_themes_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("themes"))
}

// `$OMARCHY_PATH/default/omarchy/omarchy-menu.jsonc` — the shell menu's definition.
pub fn menu_file() -> PathBuf {
    omarchy_install_dir().join("default/omarchy/omarchy-menu.jsonc")
}

// `~/.config/omarchy/extensions/omarchy-menu.jsonc` — the user's menu additions.
pub fn user_menu_extensions_file() -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".config")
            .join("omarchy")
            .join("extensions")
            .join("omarchy-menu.jsonc")
    })
}

// `~/.config/omarchy/plugins` — third-party shell plugins, one folder per id.
pub fn user_plugins_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("plugins"))
}

// `~/.config/omarchy/shell.json` — the shell's bar layout and plugin state.
pub fn shell_json_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("shell.json"))
}

// `~/.local/state/omarchy` — Quattro's runtime state directory.
pub fn omarchy_state_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".local").join("state").join("omarchy"))
}

// `~/.local/state/omarchy/current/theme` — the active theme's staged directory.
pub fn current_theme_dir() -> Option<PathBuf> {
    omarchy_state_dir().map(|d| d.join("current").join("theme"))
}

// `~/.local/state/omarchy/current/theme.name` — the active theme's directory name.
pub fn current_theme_name_file() -> Option<PathBuf> {
    omarchy_state_dir().map(|d| d.join("current").join("theme.name"))
}

// `~/.local/state/omarchy/current/next-theme` — theme-switch staging directory.
pub fn next_theme_dir() -> Option<PathBuf> {
    omarchy_state_dir().map(|d| d.join("current").join("next-theme"))
}

// `~/.config/hypr` — the user's live Hyprland config directory.
pub fn user_hyprland_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("hypr"))
}

// `~/.config/omarchist/hyprland` — Omarchist's own Hyprland settings state.
pub fn omarchist_hyprland_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchist").join("hyprland"))
}
