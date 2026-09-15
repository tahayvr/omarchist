use std::fs;

use crate::types::themes::ColorsConfig;

use super::paths::get_custom_themes_dir;

// Serializes the palette in the exact shape Quattro's `omarchy-theme-color`
// parses: one `key = "value"` per line, no TOML tables.
pub fn render_colors_toml(colors: &ColorsConfig) -> String {
    let mut out = format!(
        r#"mode = "{}"

accent = "{}"
cursor = "{}"
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
        colors.cursor,
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

    out
}

pub fn update_colors_toml(theme_name: &str, colors: &ColorsConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let toml_path = theme_dir.join("colors.toml");
    fs::write(&toml_path, render_colors_toml(colors))
        .map_err(|e| format!("Failed to write colors.toml: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::render_colors_toml;
    use crate::types::themes::ColorsConfig;

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
