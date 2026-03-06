use std::path::Path;

use crate::system::themes::color_extractor::{
    ColorPalette, copy_image_to_backgrounds, extract_palette,
};
use crate::system::themes::theme_management::{
    create_theme_from_defaults, save_theme_data, update_icons_theme,
};
use crate::types::themes::{
    BrowserConfig, BtopConfig, EditingTheme, HyprlandConfig, HyprlockConfig, MakoConfig,
    SwayosdConfig, TerminalConfig, TerminalCursor, TerminalPrimary, TerminalSelection,
    WalkerConfig, WaybarConfig,
};

// Available icon themes mapped to their representative colors (RGB)
const ICON_THEMES: &[(&str, (u8, u8, u8))] = &[
    ("Yaru-red", (233, 32, 32)),     // Red (#e92020)
    ("Yaru-blue", (32, 143, 233)),   // Blue (#208fe9)
    ("Yaru-olive", (99, 107, 47)),   // Olive (#636B2F)
    ("Yaru-yellow", (233, 186, 32)), // Yellow (#e9ba20)
    ("Yaru-purple", (94, 39, 80)),   // Purple (#5e2750)
    ("Yaru-magenta", (255, 0, 255)), // Magenta (#FF00FF)
    ("Yaru-sage", (18, 61, 24)),     // Sage (#123d18)
];

// Calculate Euclidean distance between two RGB colors
fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f32 {
    let dr = (c1.0 as f32 - c2.0 as f32).powi(2);
    let dg = (c1.1 as f32 - c2.1 as f32).powi(2);
    let db = (c1.2 as f32 - c2.2 as f32).powi(2);
    (dr + dg + db).sqrt()
}

// Convert hex color to RGB tuple
fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

// Select the best matching icon theme based on the accent color
fn select_icon_theme(accent_hex: &str, _is_light_theme: bool) -> &'static str {
    let accent_rgb = match hex_to_rgb(accent_hex) {
        Some(rgb) => rgb,
        None => return "Yaru-blue", // Default fallback
    };

    // Find the closest color match among all available themes
    ICON_THEMES
        .iter()
        .min_by(|(_, c1), (_, c2)| {
            let d1 = color_distance(accent_rgb, *c1);
            let d2 = color_distance(accent_rgb, *c2);
            d1.partial_cmp(&d2).unwrap()
        })
        .map(|(name, _)| *name)
        .unwrap_or("Yaru-blue")
}

// Progress callback for theme generation
pub type ProgressCallback = Box<dyn Fn(&str) + Send>;

// Create a complete theme from an image
pub fn create_theme_from_image(
    image_path: &Path,
    theme_name: &str,
    progress: Option<ProgressCallback>,
) -> Result<String, String> {
    let report = |msg: &str| {
        if let Some(ref cb) = progress {
            cb(msg);
        }
    };

    report("Analyzing image...");

    // Extract color palette
    let palette = extract_palette(image_path)?;

    report("Creating theme structure...");

    // Create base theme from defaults
    create_theme_from_defaults(theme_name)?;

    report("Applying colors...");

    // Build complete theme with extracted colors
    let editing_theme = build_theme_from_palette(&palette, theme_name)?;

    // Save all configs
    save_theme_data(theme_name, &editing_theme)?;

    report("Copying background image...");

    // Copy image to backgrounds folder
    copy_image_to_backgrounds(image_path, theme_name)?;

    report("Done!");

    Ok(theme_name.to_string())
}

// Build a complete EditingTheme from a color palette
fn build_theme_from_palette(
    palette: &ColorPalette,
    theme_name: &str,
) -> Result<EditingTheme, String> {
    use chrono::Utc;

    let now = Utc::now().to_rfc3339();

    // Build terminal config
    let terminal_config = build_terminal_config(palette);

    // Build all app configs
    let waybar_config = WaybarConfig {
        background: palette.background.clone(),
        foreground: palette.foreground.clone(),
    };

    let hyprland_config = HyprlandConfig {
        active_border: strip_hash(&palette.accent),
        // Use a very subtle darkening so inactive borders remain visible
        inactive_border: strip_hash(&darken_color(&palette.background, 0.08)),
    };

    let walker_config = WalkerConfig {
        background: palette.background.clone(),
        base: palette.background.clone(),
        border: palette.accent.clone(),
        foreground: palette.foreground.clone(),
        text: palette.foreground.clone(),
        selected_text: most_distinct_from_accent(&palette.terminal, &palette.foreground),
    };

    let browser_config = BrowserConfig {
        theme_color: palette.background.clone(),
    };

    let hyprlock_config = HyprlockConfig {
        color: strip_hash(&palette.background),
        inner_color: strip_hash(&palette.background),
        outer_color: strip_hash(&palette.accent),
        font_color: strip_hash(&palette.foreground),
        check_color: strip_hash(&most_distinct_from_accent(
            &palette.terminal,
            &palette.accent,
        )),
    };

    let mako_config = MakoConfig {
        text_color: palette.foreground.clone(),
        border_color: palette.accent.clone(),
        background_color: palette.background.clone(),
    };

    let btop_config = build_btop_config(palette);

    let swayosd_config = SwayosdConfig {
        background_color: palette.background.clone(),
        border_color: palette.accent.clone(),
        label: adjust_brightness(&palette.foreground, -0.3),
        image: adjust_brightness(&palette.foreground, -0.3),
        progress: adjust_brightness(&palette.foreground, -0.3),
    };

    // Select the best matching icon theme based on accent color
    let icon_theme_name = select_icon_theme(&palette.accent, palette.is_light_theme);

    // Save the icons.theme file directly
    let _ = update_icons_theme(theme_name, icon_theme_name);

    // Create icons config for the theme data
    let icons_config = serde_json::json!({
        "theme_name": icon_theme_name
    });

    // Build colors config from terminal config and palette accent
    let colors_config = crate::types::themes::ColorsConfig {
        accent: palette.accent.clone(),
        cursor: terminal_config.cursor.cursor.clone(),
        foreground: terminal_config.primary.foreground.clone(),
        background: terminal_config.primary.background.clone(),
        selection_foreground: terminal_config.selection.foreground.clone(),
        selection_background: terminal_config.selection.background.clone(),
        color0: terminal_config.normal.black.clone(),
        color1: terminal_config.normal.red.clone(),
        color2: terminal_config.normal.green.clone(),
        color3: terminal_config.normal.yellow.clone(),
        color4: terminal_config.normal.blue.clone(),
        color5: terminal_config.normal.magenta.clone(),
        color6: terminal_config.normal.cyan.clone(),
        color7: terminal_config.normal.white.clone(),
        color8: terminal_config.bright.black.clone(),
        color9: terminal_config.bright.red.clone(),
        color10: terminal_config.bright.green.clone(),
        color11: terminal_config.bright.yellow.clone(),
        color12: terminal_config.bright.blue.clone(),
        color13: terminal_config.bright.magenta.clone(),
        color14: terminal_config.bright.cyan.clone(),
        color15: terminal_config.bright.white.clone(),
    };

    Ok(EditingTheme {
        version: "1.0.0".to_string(),
        name: theme_name.to_string(),
        created_at: now.clone(),
        modified_at: now,
        author: None,
        apps: crate::types::themes::AppConfigs {
            alacritty: None,
            waybar: Some(waybar_config),
            chromium: Some(browser_config),
            btop: Some(btop_config),
            hyprland: Some(hyprland_config),
            hyprlock: Some(hyprlock_config),
            mako: Some(mako_config),
            walker: Some(walker_config),
            swayosd: Some(swayosd_config),
            neovim: None,
            vscode: None,
            icons: Some(icons_config),
            ghostty: None,
            kitty: None,
            terminal: Some(terminal_config),
        },
        colors: colors_config,
        is_light_theme: palette.is_light_theme,
    })
}

fn build_terminal_config(palette: &ColorPalette) -> TerminalConfig {
    TerminalConfig {
        primary: TerminalPrimary {
            background: palette.background.clone(),
            foreground: palette.foreground.clone(),
        },
        cursor: TerminalCursor {
            cursor: palette.accent.clone(),
            text: palette.background.clone(),
        },
        selection: TerminalSelection {
            background: adjust_brightness(
                &palette.accent,
                if palette.is_light_theme { 0.25 } else { -0.25 },
            ),
            foreground: palette.foreground.clone(),
        },
        normal: palette.terminal.clone(),
        bright: palette.bright.clone(),
    }
}

fn build_btop_config(palette: &ColorPalette) -> BtopConfig {
    let bg = &palette.background;
    let fg = &palette.foreground;
    let accent = &palette.accent;
    let (grad_start, grad_end) = widest_hue_pair(&palette.terminal);
    let grad_mid = accent.clone();

    BtopConfig {
        main_bg: bg.clone(),
        main_fg: fg.clone(),
        title: adjust_brightness(fg, -0.2),
        hi_fg: accent.clone(),
        selected_bg: accent.clone(),
        selected_fg: bg.clone(),
        inactive_fg: adjust_brightness(fg, -0.5),
        proc_misc: adjust_brightness(fg, -0.3),
        cpu_box: adjust_brightness(fg, -0.4),
        mem_box: adjust_brightness(fg, -0.4),
        net_box: adjust_brightness(fg, -0.4),
        proc_box: adjust_brightness(fg, -0.4),
        div_line: adjust_brightness(fg, -0.4),
        temp_start: grad_start.clone(),
        temp_mid: grad_mid.clone(),
        temp_end: grad_end.clone(),
        cpu_start: grad_start.clone(),
        cpu_mid: grad_mid.clone(),
        cpu_end: grad_end.clone(),
        free_start: grad_start.clone(),
        free_mid: grad_mid.clone(),
        free_end: grad_end.clone(),
        cached_start: grad_start.clone(),
        cached_mid: grad_mid.clone(),
        cached_end: grad_end.clone(),
        available_start: grad_start.clone(),
        available_mid: grad_mid.clone(),
        available_end: grad_end.clone(),
        used_start: grad_start.clone(),
        used_mid: grad_mid.clone(),
        used_end: grad_end.clone(),
        download_start: grad_start.clone(),
        download_mid: grad_mid.clone(),
        download_end: grad_end.clone(),
        upload_start: grad_start,
        upload_mid: grad_mid,
        upload_end: grad_end,
    }
}

fn strip_hash(hex: &str) -> String {
    hex.trim_start_matches('#').to_string()
}

// Return the two terminal palette colors that are farthest apart in hue.
// Used for btop gradients so start and end are maximally distinct on-palette colors.
fn widest_hue_pair(palette: &crate::types::themes::TerminalPalette) -> (String, String) {
    use palette::{FromColor, Hsl, Srgb};

    let slots = [
        &palette.red,
        &palette.yellow,
        &palette.green,
        &palette.cyan,
        &palette.blue,
        &palette.magenta,
    ];

    let hues: Vec<f32> = slots
        .iter()
        .map(|hex| {
            let hex = hex.trim_start_matches('#');
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            let hsl: Hsl = Hsl::from_color(Srgb::new(r, g, b));
            hsl.hue.into_degrees()
        })
        .collect();

    let mut best_dist = -1.0f32;
    let mut best_i = 0;
    let mut best_j = 3; // default: red and cyan (opposite)

    for i in 0..slots.len() {
        for j in (i + 1)..slots.len() {
            let diff = (hues[i] - hues[j]).abs();
            let dist = diff.min(360.0 - diff);
            if dist > best_dist {
                best_dist = dist;
                best_i = i;
                best_j = j;
            }
        }
    }

    (slots[best_i].to_string(), slots[best_j].to_string())
}

// Return the terminal palette color whose hue is most distant from the accent hue.
// Used for hyprlock check_color so it's always visually distinct from the accent.
fn most_distinct_from_accent(
    terminal: &crate::types::themes::TerminalPalette,
    accent_hex: &str,
) -> String {
    use palette::{FromColor, Hsl, Srgb};

    let accent_hue = {
        let hex = accent_hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        let hsl: Hsl = Hsl::from_color(Srgb::new(r, g, b));
        hsl.hue.into_degrees()
    };

    let slots = [
        &terminal.red,
        &terminal.yellow,
        &terminal.green,
        &terminal.cyan,
        &terminal.blue,
        &terminal.magenta,
    ];

    slots
        .iter()
        .max_by(|a, b| {
            let hue_of = |hex: &&&String| {
                let hex = hex.trim_start_matches('#');
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
                let bv = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
                let hsl: Hsl = Hsl::from_color(Srgb::new(r, g, bv));
                let diff = (hsl.hue.into_degrees() - accent_hue).abs();
                diff.min(360.0 - diff)
            };
            hue_of(a).partial_cmp(&hue_of(b)).unwrap()
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| terminal.green.clone())
}

fn lighten_color(hex: &str, amount: f32) -> String {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let new_r = ((r as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;
    let new_g = ((g as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;
    let new_b = ((b as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;

    format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b)
}

fn darken_color(hex: &str, amount: f32) -> String {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let new_r = (r as f32 * (1.0 - amount)) as u8;
    let new_g = (g as f32 * (1.0 - amount)) as u8;
    let new_b = (b as f32 * (1.0 - amount)) as u8;

    format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b)
}

// Adjust brightness: positive = lighten, negative = darken
fn adjust_brightness(hex: &str, amount: f32) -> String {
    if amount >= 0.0 {
        lighten_color(hex, amount)
    } else {
        darken_color(hex, amount.abs())
    }
}
