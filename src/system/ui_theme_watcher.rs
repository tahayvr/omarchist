use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui::App;
use gpui_component::{Theme, ThemeConfig};
use smol::Timer;

use crate::system::themes::color_utils::{
    adjust_lightness, darken, is_dark_color, lighten, with_alpha,
};

const POLL_INTERVAL: Duration = Duration::from_secs(1);

thread_local! {
    pub static PENDING_UI_THEME_RELOAD: RefCell<bool> = const { RefCell::new(false) };
}

// the omarchy current theme directory:
// `~/.config/omarchy/current/theme/`
fn get_omarchy_current_theme_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(
        home.join(".config")
            .join("omarchy")
            .join("current")
            .join("theme"),
    )
}

// `~/.config/omarchy/current/theme.name`
fn get_active_omarchy_theme_name() -> Option<String> {
    let home = dirs::home_dir()?;
    let name_file = home
        .join(".config")
        .join("omarchy")
        .join("current")
        .join("theme.name");
    let name = std::fs::read_to_string(&name_file).ok()?;
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

// `~/.config/omarchy/current/theme/colors.toml`
fn get_colors_toml_path() -> Option<PathBuf> {
    Some(get_omarchy_current_theme_dir()?.join("colors.toml"))
}

fn parse_colors_toml(path: &PathBuf) -> Option<HashMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            map.insert(key, value);
        }
    }

    Some(map)
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

    let is_dark = is_dark_color(&bg);
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

// Loads the omarchy current theme from `colors.toml` and applies it to the UI.
pub fn load_and_apply_omarchy_theme(cx: &mut App) {
    let theme_name = get_active_omarchy_theme_name().unwrap_or_else(|| "omarchy".to_string());
    if let Some(colors_path) = get_colors_toml_path()
        && let Some(colors) = parse_colors_toml(&colors_path)
    {
        let config = build_theme_config(&colors, &theme_name);
        let mode = config.mode;
        let config_rc = Rc::new(config);
        if mode.is_dark() {
            Theme::global_mut(cx).dark_theme = config_rc;
        } else {
            Theme::global_mut(cx).light_theme = config_rc;
        }
        Theme::change(mode, None, cx);
    }
    // If omarchy theme is unavailable, the embedded theme stays in effect.
}

// Check the omarchy current theme directory every second.
pub fn spawn_ui_theme_watcher(cx: &mut App) {
    cx.spawn(async move |cx| {
        let mut last_theme_name: Option<String> = get_active_omarchy_theme_name();

        loop {
            Timer::after(POLL_INTERVAL).await;

            // Stop the loop if the app has shut down.
            if cx.update(|_| {}).is_err() {
                break;
            }

            let current_theme_name = get_active_omarchy_theme_name();

            if current_theme_name != last_theme_name {
                last_theme_name = current_theme_name;
                PENDING_UI_THEME_RELOAD.with(|flag| {
                    *flag.borrow_mut() = true;
                });
                let _ = cx.refresh();
            }
        }
    })
    .detach();
}
