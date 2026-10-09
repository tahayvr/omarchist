use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui::App;
use gpui_component::{Theme, ThemeConfig, ThemeMode, ThemeSet};
use smol::Timer;

use crate::system::themes::color_utils::{
    adjust_lightness, darken, is_dark_color, lighten, with_alpha,
};
use crate::system::themes::theme_management::colors::read_colors_toml;
use crate::types::themes::ColorsConfig;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// `~/.local/state/omarchy/current/theme.name`: the folder name of the
/// theme Omarchy runs.
pub fn get_active_omarchy_theme_name() -> Option<String> {
    let name_file = crate::system::omarchy_paths::current_theme_name_file()?;
    let name = std::fs::read_to_string(&name_file).ok()?;
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

// `~/.local/state/omarchy/current/theme/colors.toml`
fn get_colors_toml_path() -> Option<PathBuf> {
    Some(crate::system::omarchy_paths::current_theme_dir()?.join("colors.toml"))
}

fn parse_colors_toml(path: &PathBuf) -> Option<HashMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(resolved_colors(&content))
}

/// The palette as Omarchy's own resolver reads it. Quattro's themes name
/// their colours (`blue`, `bright_blue`, `selection`) rather than numbering
/// them, so the file goes through the same alias-aware reader the Theme
/// Designer uses; only keys the file sets end up in the map, and
/// `build_theme_config` derives the rest.
fn resolved_colors(content: &str) -> HashMap<String, String> {
    let blank = ColorsConfig {
        mode: String::new(),
        accent: String::new(),
        foreground: String::new(),
        background: String::new(),
        selection_foreground: String::new(),
        selection_background: String::new(),
        color0: String::new(),
        color1: String::new(),
        color2: String::new(),
        color3: String::new(),
        color4: String::new(),
        color5: String::new(),
        color6: String::new(),
        color7: String::new(),
        color8: String::new(),
        color9: String::new(),
        color10: String::new(),
        color11: String::new(),
        color12: String::new(),
        color13: String::new(),
        color14: String::new(),
        color15: String::new(),
        hyprland_active_border: None,
        hyprland_inactive_border: None,
        extra: Vec::new(),
    };
    let colors = read_colors_toml(content, &blank);
    let extra = |key: &str| {
        colors
            .extra
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    let mut map = HashMap::new();
    let mut put = |key: &str, value: String| {
        if !value.trim().is_empty() {
            map.insert(key.to_string(), value);
        }
    };
    put("mode", colors.mode.clone());
    put("accent", colors.accent.clone());
    put("foreground", colors.foreground.clone());
    put("background", colors.background.clone());
    // Omarchy reads `selection` where a theme has no `selection_background`.
    put(
        "selection_background",
        if colors.selection_background.is_empty() {
            extra("selection").unwrap_or_default()
        } else {
            colors.selection_background.clone()
        },
    );
    for (ix, value) in [
        &colors.color0,
        &colors.color1,
        &colors.color2,
        &colors.color3,
        &colors.color4,
        &colors.color5,
        &colors.color6,
        &colors.color7,
        &colors.color8,
        &colors.color9,
        &colors.color10,
        &colors.color11,
        &colors.color12,
        &colors.color13,
        &colors.color14,
        &colors.color15,
    ]
    .into_iter()
    .enumerate()
    {
        put(&format!("color{ix}"), value.clone());
    }
    map
}

fn build_theme_config(colors: &HashMap<String, String>, theme_name: &str) -> ThemeConfig {
    let bg = colors
        .get("background")
        .cloned()
        .unwrap_or_else(|| "#1f1f28".to_string());
    let fg = colors
        .get("foreground")
        .cloned()
        .unwrap_or_else(|| "#dcd7ba".to_string());
    let accent_color = colors.get("accent").cloned().unwrap_or_else(|| {
        colors
            .get("color4")
            .cloned()
            .unwrap_or_else(|| "#7e9cd8".to_string())
    });

    // Terminal palette from colors.toml
    let c1 = colors
        .get("color1")
        .cloned()
        .unwrap_or("#c34043".to_string()); // red
    let c2 = colors
        .get("color2")
        .cloned()
        .unwrap_or("#76946a".to_string()); // green
    let c3 = colors
        .get("color3")
        .cloned()
        .unwrap_or("#c0a36e".to_string()); // yellow
    let c4 = colors
        .get("color4")
        .cloned()
        .unwrap_or("#7e9cd8".to_string()); // blue
    let c5 = colors
        .get("color5")
        .cloned()
        .unwrap_or("#957fb8".to_string()); // magenta
    let c6 = colors
        .get("color6")
        .cloned()
        .unwrap_or("#6a9589".to_string()); // cyan

    // Bright variants (color8–color15)
    let c9 = colors
        .get("color9")
        .cloned()
        .unwrap_or_else(|| lighten(&c1, 0.2));
    let c10 = colors
        .get("color10")
        .cloned()
        .unwrap_or_else(|| lighten(&c2, 0.2));
    let c11 = colors
        .get("color11")
        .cloned()
        .unwrap_or_else(|| lighten(&c3, 0.2));
    let c12 = colors
        .get("color12")
        .cloned()
        .unwrap_or_else(|| lighten(&c4, 0.2));
    let c13 = colors
        .get("color13")
        .cloned()
        .unwrap_or_else(|| lighten(&c5, 0.2));
    let c14 = colors
        .get("color14")
        .cloned()
        .unwrap_or_else(|| lighten(&c6, 0.2));

    // The file says which mode it is for; the background decides otherwise.
    let is_dark = match colors.get("mode").map(String::as_str) {
        Some("dark") => true,
        Some("light") => false,
        _ => is_dark_color(&bg),
    };
    let mode_str = if is_dark { "dark" } else { "light" };

    // Make UI surface colors from the background
    let (muted_bg, popover_bg, border_color, input_border) = if is_dark {
        (
            adjust_lightness(&bg, 0.05), // muted bg
            adjust_lightness(&bg, 0.02), // popover
            adjust_lightness(&bg, 0.10), // border
            adjust_lightness(&bg, 0.12), // input border
        )
    } else {
        (
            adjust_lightness(&bg, -0.05),
            adjust_lightness(&bg, -0.02),
            adjust_lightness(&bg, -0.12),
            adjust_lightness(&bg, -0.14),
        )
    };

    let muted_fg = if is_dark {
        darken(&fg, 0.30)
    } else {
        lighten(&fg, 0.30)
    };

    let primary_bg = fg.clone();
    let primary_fg = bg.clone();
    let primary_hover = darken(&primary_bg, 0.16);
    let primary_active = darken(&primary_bg, 0.20);

    let secondary_bg = if is_dark {
        adjust_lightness(&bg, 0.08)
    } else {
        adjust_lightness(&bg, -0.08)
    };
    let switch_bg = if is_dark {
        adjust_lightness(&bg, 0.18)
    } else {
        adjust_lightness(&bg, -0.18)
    };
    let secondary_hover = if is_dark {
        adjust_lightness(&bg, 0.18)
    } else {
        adjust_lightness(&bg, -0.18)
    };
    let secondary_active = if is_dark {
        adjust_lightness(&bg, 0.15)
    } else {
        adjust_lightness(&bg, -0.15)
    };

    let tab_bar_bg = if is_dark {
        adjust_lightness(&bg, 0.05)
    } else {
        adjust_lightness(&bg, -0.05)
    };

    let list_active_bg = with_alpha(&secondary_bg, "22");
    let list_active_border = lighten(&border_color, 0.08);
    let list_even_bg = with_alpha(&muted_bg, "99");
    let list_head_bg = if is_dark {
        adjust_lightness(&bg, 0.07)
    } else {
        adjust_lightness(&bg, -0.07)
    };

    let selection_bg = colors
        .get("selection_background")
        .cloned()
        .unwrap_or_else(|| {
            if is_dark {
                adjust_lightness(&bg, 0.12)
            } else {
                adjust_lightness(&bg, -0.12)
            }
        });

    // Build the theme JSON the same way the theme.json file is structured,
    let mut colors_obj = serde_json::Map::new();
    let insert = |map: &mut serde_json::Map<_, _>, k: &str, v: String| {
        map.insert(k.to_string(), serde_json::Value::String(v));
    };

    insert(&mut colors_obj, "background", bg.clone());
    insert(&mut colors_obj, "foreground", fg.clone());
    insert(&mut colors_obj, "border", border_color.clone());
    insert(&mut colors_obj, "input.border", input_border);
    insert(&mut colors_obj, "muted.background", muted_bg);
    insert(&mut colors_obj, "muted.foreground", muted_fg);
    insert(&mut colors_obj, "popover.background", popover_bg);
    insert(&mut colors_obj, "popover.foreground", fg.clone());
    insert(
        &mut colors_obj,
        "accent.background",
        with_alpha(&secondary_bg, "22"),
    );
    insert(&mut colors_obj, "accent.foreground", fg.clone());
    insert(&mut colors_obj, "primary.background", primary_bg);
    insert(&mut colors_obj, "primary.foreground", primary_fg);
    insert(&mut colors_obj, "primary.hover.background", primary_hover);
    insert(&mut colors_obj, "primary.active.background", primary_active);
    insert(
        &mut colors_obj,
        "secondary.background",
        secondary_bg.clone(),
    );
    insert(&mut colors_obj, "secondary.foreground", fg.clone());
    insert(
        &mut colors_obj,
        "secondary.hover.background",
        secondary_hover,
    );
    insert(
        &mut colors_obj,
        "secondary.active.background",
        secondary_active,
    );
    insert(&mut colors_obj, "switch.background", switch_bg);
    insert(&mut colors_obj, "ring", lighten(&accent_color, 0.05));
    insert(
        &mut colors_obj,
        "scrollbar.background",
        with_alpha(&bg, "00"),
    );
    insert(
        &mut colors_obj,
        "scrollbar.thumb.background",
        with_alpha(&fg, "4c"),
    );
    insert(&mut colors_obj, "list.active.background", list_active_bg);
    insert(&mut colors_obj, "list.active.border", list_active_border);
    insert(&mut colors_obj, "list.even.background", list_even_bg);
    insert(&mut colors_obj, "list.head.background", list_head_bg);
    insert(&mut colors_obj, "tab.background", with_alpha(&bg, "00"));
    insert(&mut colors_obj, "tab.active.background", bg.clone());
    insert(&mut colors_obj, "tab.active.foreground", fg.clone());
    insert(&mut colors_obj, "tab_bar.background", tab_bar_bg);
    insert(&mut colors_obj, "title_bar.background", bg.clone());
    insert(&mut colors_obj, "title_bar.border", border_color.clone());
    insert(&mut colors_obj, "selection.background", selection_bg);
    insert(&mut colors_obj, "base.red", c1);
    insert(&mut colors_obj, "base.red.light", c9);
    insert(&mut colors_obj, "base.green", c2);
    insert(&mut colors_obj, "base.green.light", c10);
    insert(&mut colors_obj, "base.yellow", c3);
    insert(&mut colors_obj, "base.yellow.light", c11);
    insert(&mut colors_obj, "base.blue", c4);
    insert(&mut colors_obj, "base.blue.light", c12);
    insert(&mut colors_obj, "base.magenta", c5);
    insert(&mut colors_obj, "base.magenta.light", c13);
    insert(&mut colors_obj, "base.cyan", c6);
    insert(&mut colors_obj, "base.cyan.light", c14);

    let mut theme_map = serde_json::Map::new();
    theme_map.insert(
        "name".to_string(),
        serde_json::Value::String(format!(
            "{} {}",
            theme_name,
            if is_dark { "Dark" } else { "Light" }
        )),
    );
    theme_map.insert(
        "mode".to_string(),
        serde_json::Value::String(mode_str.to_string()),
    );
    theme_map.insert("radius".to_string(), serde_json::Value::Number(0.into()));
    theme_map.insert("radius.lg".to_string(), serde_json::Value::Number(0.into()));
    theme_map.insert(
        "font.family".to_string(),
        serde_json::Value::String("JetBrainsMono Nerd Font Mono".to_string()),
    );
    theme_map.insert(
        "font.size".to_string(),
        serde_json::Value::Number(14.into()),
    );
    theme_map.insert(
        "mono_font.family".to_string(),
        serde_json::Value::String("JetBrainsMono Nerd Font Mono".to_string()),
    );
    theme_map.insert(
        "mono_font.size".to_string(),
        serde_json::Value::Number(13.into()),
    );
    theme_map.insert("colors".to_string(), serde_json::Value::Object(colors_obj));

    let theme_json = serde_json::Value::Object(theme_map);

    serde_json::from_value(theme_json).unwrap_or_else(|e| {
        eprintln!(
            "[ui_theme_watcher] Failed to build ThemeConfig from colors.toml: {}",
            e
        );
        ThemeConfig::default()
    })
}

/// The app's own look, a light and a dark theme.
const THEME_FILE: &str = include_str!("../../ui_themes/theme.json");

fn embedded_themes() -> (Option<ThemeConfig>, Option<ThemeConfig>) {
    let theme_set: ThemeSet = match serde_json::from_str(THEME_FILE) {
        Ok(theme_set) => theme_set,
        Err(err) => {
            eprintln!("Failed to parse the Omarchist theme JSON: {err}");
            return (None, None);
        }
    };
    let mut light = None;
    let mut dark = None;
    for theme in theme_set.themes {
        if theme.mode.is_dark() {
            dark = Some(theme);
        } else {
            light = Some(theme);
        }
    }
    (light, dark)
}

/// Applies the look the Settings page holds (`theme_mode`): `light` and
/// `dark` are the app's own themes, with their own palette; `omarchy`
/// follows the desktop theme, built from its `colors.toml` with its
/// palette, and falls back to the app's dark theme when that cannot be
/// read. Every `Theme::change` re-applies the theme's own `font.size`, so
/// the user's size is put back afterwards.
pub fn apply_ui_theme(cx: &mut App) {
    // Start from the app's own themes every time, so a forced mode never
    // shows the desktop theme's colours left behind by an earlier follow.
    let (light, dark) = embedded_themes();
    if let Some(light) = light {
        Theme::global_mut(cx).light_theme = Rc::new(light);
    }
    if let Some(dark) = dark {
        Theme::global_mut(cx).dark_theme = Rc::new(dark);
    }
    match crate::system::config::config_setup::settings()
        .theme_mode
        .as_str()
    {
        "light" => Theme::change(ThemeMode::Light, None, cx),
        "dark" => Theme::change(ThemeMode::Dark, None, cx),
        _ => {
            let theme_name =
                get_active_omarchy_theme_name().unwrap_or_else(|| "omarchy".to_string());
            let omarchy = get_colors_toml_path()
                .and_then(|path| parse_colors_toml(&path))
                .map(|colors| build_theme_config(&colors, &theme_name));
            match omarchy {
                Some(config) => {
                    let mode = config.mode;
                    let config = Rc::new(config);
                    if mode.is_dark() {
                        Theme::global_mut(cx).dark_theme = config;
                    } else {
                        Theme::global_mut(cx).light_theme = config;
                    }
                    Theme::change(mode, None, cx);
                }
                None => Theme::change(ThemeMode::Dark, None, cx),
            }
        }
    }
    apply_font_size(cx);
    // Toasts rise from the bottom right, away from every page's toolbar
    // and the editors' Save and Run, which sit top right.
    Theme::global_mut(cx).notification.placement = gpui::Anchor::BottomRight;
}

/// The text size the Settings page holds, in pixels.
pub fn font_size_px(size: &str) -> f32 {
    match size {
        "small" => 14.0,
        "large" => 18.0,
        _ => 16.0,
    }
}

/// Applies the saved font size over whatever the theme config carries.
pub fn apply_font_size(cx: &mut App) {
    let size = crate::system::config::config_setup::settings().font_size;
    Theme::global_mut(cx).font_size = gpui::px(font_size_px(&size));
}

/// The modification time of the user's themes folder: a theme created,
/// renamed, installed or deleted by anything (the CLI, `omarchy theme
/// install`, a file manager) changes it.
fn user_themes_dir_modified() -> Option<std::time::SystemTime> {
    let dir = crate::system::omarchy_paths::user_themes_dir()?;
    std::fs::metadata(dir).and_then(|m| m.modified()).ok()
}

/// When the running theme's `colors.toml` last changed: editing the
/// applied theme in the Designer and re-applying it rewrite the file
/// without changing the theme's name.
fn current_colors_modified() -> Option<std::time::SystemTime> {
    let path = get_colors_toml_path()?;
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

// Check the omarchy current theme every second: a switch (or a re-apply
// of the same theme) reloads the app's own look and moves the "Applied"
// marker; a change to the themes folder refreshes the Themes page.
pub fn spawn_ui_theme_watcher(cx: &mut App) {
    cx.spawn(async move |cx| {
        let mut last_theme_name: Option<String> = get_active_omarchy_theme_name();
        let mut last_dir_modified = user_themes_dir_modified();
        let mut last_colors_modified = current_colors_modified();

        loop {
            Timer::after(POLL_INTERVAL).await;

            let current_theme_name = get_active_omarchy_theme_name();
            let dir_modified = user_themes_dir_modified();
            let colors_modified = current_colors_modified();

            if current_theme_name != last_theme_name || colors_modified != last_colors_modified {
                last_theme_name = current_theme_name;
                last_colors_modified = colors_modified;
                crate::ui::app_events::emit_async(
                    cx,
                    crate::ui::app_events::AppEvent::ReloadUiTheme,
                );
                crate::ui::app_events::emit_async(
                    cx,
                    crate::ui::app_events::AppEvent::RefreshThemes,
                );
            } else if dir_modified != last_dir_modified {
                last_dir_modified = dir_modified;
                crate::ui::app_events::emit_async(
                    cx,
                    crate::ui::app_events::AppEvent::RefreshThemes,
                );
            }
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::{build_theme_config, resolved_colors};

    /// A Quattro theme names its colours; none of the numbered keys appear.
    const MATTE: &str = r##"mode = "dark"
accent = "#e68e0d"
selection = "#2a2a2a"
background = "#121212"
foreground = "#bebebe"
red = "#D35F5F"
yellow = "#b91c1c"
green = "#FFC107"
cyan = "#bebebe"
blue = "#e68e0d"
magenta = "#D35F5F"
bright_blue = "#f59e0b"
"##;

    #[test]
    fn named_colours_are_the_palette_the_ui_follows() {
        let colors = resolved_colors(MATTE);
        assert_eq!(colors["color4"], "#e68e0d");
        assert_eq!(colors["color3"], "#b91c1c");
        assert_eq!(colors["color12"], "#f59e0b");
        assert_eq!(colors["selection_background"], "#2a2a2a");
        assert!(
            !colors.contains_key("color9"),
            "an unset key is derived later"
        );
        let config = build_theme_config(&colors, "matte-black");
        assert!(config.mode.is_dark());
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["colors"]["base.blue"], "#e68e0d");
        assert_eq!(json["colors"]["base.green"], "#FFC107");
        assert_eq!(json["colors"]["base.blue.light"], "#f59e0b");
        assert_eq!(json["colors"]["selection.background"], "#2a2a2a");
    }
}
