use crate::error::Result;

use crate::types::themes::ColorsConfig;

use crate::system::fs::write_atomic;
use crate::system::themes::theme_file_ops::omarchist_theme_dir;

// Serializes the palette in the exact shape Quattro's `omarchy-theme-color`
// parses: one `key = "value"` per line, no TOML tables.
pub fn render_colors_toml(colors: &ColorsConfig) -> String {
    let mut out = format!(
        r#"mode = "{}"

accent = "{}"
foreground = "{}"
background = "{}"
selection_foreground = "{}"
selection_background = "{}"

color0 = "{}"
color1 = "{}"
color2 = "{}"
color3 = "{}"
color4 = "{}"
color5 = "{}"
color6 = "{}"
color7 = "{}"
color8 = "{}"
color9 = "{}"
color10 = "{}"
color11 = "{}"
color12 = "{}"
color13 = "{}"
color14 = "{}"
color15 = "{}"
"#,
        colors.mode,
        colors.accent,
        colors.foreground,
        colors.background,
        colors.selection_foreground,
        colors.selection_background,
        colors.color0,
        colors.color1,
        colors.color2,
        colors.color3,
        colors.color4,
        colors.color5,
        colors.color6,
        colors.color7,
        colors.color8,
        colors.color9,
        colors.color10,
        colors.color11,
        colors.color12,
        colors.color13,
        colors.color14,
        colors.color15,
    );

    // Optional Hyprland border overrides, read by `hyprland.lua.tpl`.
    let active = colors.hyprland_active_border.as_deref().map(str::trim);
    let inactive = colors.hyprland_inactive_border.as_deref().map(str::trim);
    if active.is_some_and(|v| !v.is_empty()) || inactive.is_some_and(|v| !v.is_empty()) {
        out.push('\n');
        if let Some(v) = active.filter(|v| !v.is_empty()) {
            out.push_str(&format!("hyprland_active_border = \"{v}\"\n"));
        }
        if let Some(v) = inactive.filter(|v| !v.is_empty()) {
            out.push_str(&format!("hyprland_inactive_border = \"{v}\"\n"));
        }
    }

    for (key, value) in &colors.extra {
        out.push_str(&format!("{key} = \"{value}\"\n"));
    }

    out
}

/// The palette as `colors.toml` holds it, over `fallback` for any key the
/// file does not set. `colors.toml` is the theme's source of truth (Omarchy
/// reads it, and users are told they can edit it), so the designer starts
/// from it rather than from the manifest's copy. Semantic aliases
/// (`red` for `color1`, ...) are honoured the way `omarchy-theme-color`
/// honours them; keys Omarchist does not know are kept as `extra`.
pub fn read_colors_toml(content: &str, fallback: &ColorsConfig) -> ColorsConfig {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            pairs.push((key.trim().to_string(), value.to_string()));
        }
    }
    let get = |key: &str| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
    let pick = |key: &str, alias: &str, current: &str| {
        get(key)
            .or_else(|| get(alias))
            .unwrap_or_else(|| current.to_string())
    };
    let mut colors = fallback.clone();
    colors.mode = get("mode").unwrap_or_else(|| fallback.mode.clone());
    colors.accent = pick("accent", "accent", &fallback.accent);
    colors.foreground = pick("foreground", "foreground", &fallback.foreground);
    colors.background = pick("background", "background", &fallback.background);
    colors.selection_foreground = pick(
        "selection_foreground",
        "selection_foreground",
        &fallback.selection_foreground,
    );
    colors.selection_background = pick(
        "selection_background",
        "selection_background",
        &fallback.selection_background,
    );
    colors.color0 = pick("color0", "black", &fallback.color0);
    colors.color1 = pick("color1", "red", &fallback.color1);
    colors.color2 = pick("color2", "green", &fallback.color2);
    colors.color3 = pick("color3", "yellow", &fallback.color3);
    colors.color4 = pick("color4", "blue", &fallback.color4);
    colors.color5 = get("color5")
        .or_else(|| get("magenta"))
        .or_else(|| get("purple"))
        .unwrap_or_else(|| fallback.color5.clone());
    colors.color6 = pick("color6", "cyan", &fallback.color6);
    colors.color7 = pick("color7", "white", &fallback.color7);
    colors.color8 = pick("color8", "bright_black", &fallback.color8);
    colors.color9 = pick("color9", "bright_red", &fallback.color9);
    colors.color10 = pick("color10", "bright_green", &fallback.color10);
    colors.color11 = pick("color11", "bright_yellow", &fallback.color11);
    colors.color12 = pick("color12", "bright_blue", &fallback.color12);
    colors.color13 = get("color13")
        .or_else(|| get("bright_magenta"))
        .or_else(|| get("bright_purple"))
        .unwrap_or_else(|| fallback.color13.clone());
    colors.color14 = pick("color14", "bright_cyan", &fallback.color14);
    colors.color15 = pick("color15", "bright_white", &fallback.color15);
    colors.hyprland_active_border = get("hyprland_active_border")
        .filter(|v| !v.trim().is_empty())
        .or_else(|| fallback.hyprland_active_border.clone());
    colors.hyprland_inactive_border = get("hyprland_inactive_border")
        .filter(|v| !v.trim().is_empty())
        .or_else(|| fallback.hyprland_inactive_border.clone());
    colors.extra = pairs
        .into_iter()
        .filter(|(k, _)| !KNOWN_KEYS.contains(&k.as_str()))
        .collect();
    colors
}

/// Keys Omarchist owns in `colors.toml`, including the aliases Omarchy's
/// resolver maps onto them.
const KNOWN_KEYS: &[&str] = &[
    "mode",
    "accent",
    "foreground",
    "background",
    "selection_foreground",
    "selection_background",
    "color0",
    "color1",
    "color2",
    "color3",
    "color4",
    "color5",
    "color6",
    "color7",
    "color8",
    "color9",
    "color10",
    "color11",
    "color12",
    "color13",
    "color14",
    "color15",
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "purple",
    "cyan",
    "white",
    "bright_black",
    "bright_red",
    "bright_green",
    "bright_yellow",
    "bright_blue",
    "bright_magenta",
    "bright_purple",
    "bright_cyan",
    "bright_white",
    "hyprland_active_border",
    "hyprland_inactive_border",
];

pub fn update_colors_toml(theme_name: &str, colors: &ColorsConfig) -> Result<()> {
    let theme_dir = omarchist_theme_dir(theme_name)?;

    let toml_path = theme_dir.join("colors.toml");
    write_atomic(&toml_path, render_colors_toml(colors), "colors.toml")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{read_colors_toml, render_colors_toml};
    use crate::types::themes::ColorsConfig;

    #[test]
    fn a_hand_edited_colors_toml_wins_over_the_manifest_and_keeps_extra_keys() {
        let manifest = ColorsConfig::default();
        let toml = "mode = \"light\"\naccent = \"#ff0000\"\nred = \"#aa0000\"\norange = \"#ffa500\"\n# a comment\n";
        let colors = read_colors_toml(toml, &manifest);
        assert_eq!(colors.mode, "light");
        assert_eq!(colors.accent, "#ff0000");
        assert_eq!(colors.color1, "#aa0000", "the semantic alias fills color1");
        assert_eq!(colors.color2, manifest.color2, "missing keys fall back");
        assert_eq!(
            colors.extra,
            vec![("orange".to_string(), "#ffa500".to_string())]
        );

        let rendered = render_colors_toml(&colors);
        assert!(rendered.contains("accent = \"#ff0000\"\n"));
        assert!(rendered.ends_with("orange = \"#ffa500\"\n"));
        assert!(
            !rendered.contains("\nred = "),
            "aliases are written as colorN"
        );
        let again = read_colors_toml(&rendered, &manifest);
        assert_eq!(again.extra, colors.extra, "round trip keeps extra keys");
    }

    #[test]
    fn render_omits_border_keys_when_unset() {
        let toml = render_colors_toml(&ColorsConfig::default());
        assert!(toml.starts_with("mode = \"dark\"\n"));
        assert!(toml.contains("color15 = \"#F8F8FF\"\n"));
        assert!(!toml.contains("hyprland_active_border"));
        assert!(!toml.contains("hyprland_inactive_border"));
    }

    #[test]
    fn render_writes_border_keys_including_gradients() {
        let colors = ColorsConfig {
            hyprland_active_border: Some("rgba(26a269ee) rgba(2ec27eee) 45deg".to_string()),
            hyprland_inactive_border: Some("  ".to_string()),
            ..ColorsConfig::default()
        };
        let toml = render_colors_toml(&colors);
        assert!(
            toml.contains("hyprland_active_border = \"rgba(26a269ee) rgba(2ec27eee) 45deg\"\n")
        );
        assert!(
            !toml.contains("hyprland_inactive_border"),
            "blank values must not be written"
        );
    }
}
