use std::path::Path;

use crate::system::themes::color_extractor::{
    ColorPalette, copy_image_to_backgrounds, extract_palette,
};
use crate::system::themes::theme_management::{create_theme_from_defaults, save_theme_data};
use crate::types::themes::{
    BrowserConfig, BtopConfig, EditingTheme, HyprlandConfig, HyprlockConfig, MakoConfig,
    SwayosdConfig, TerminalConfig, TerminalCursor, TerminalPalette, TerminalPrimary,
    TerminalSelection, WalkerConfig, WaybarConfig,
};

/// Progress callback for theme generation
pub type ProgressCallback = Box<dyn Fn(&str) + Send>;

/// Create a complete theme from an image
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

/// Build a complete EditingTheme from a color palette
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
        inactive_border: strip_hash(&darken_color(&palette.background, 0.2)),
        border_size: 1,
        gaps_in: 5,
        gaps_out: 10,
        rounding: 0,
    };

    let walker_config = WalkerConfig {
        background: palette.background.clone(),
        base: palette.background.clone(),
        border: palette.accent.clone(),
        foreground: palette.foreground.clone(),
        text: palette.foreground.clone(),
        selected_text: palette.terminal.magenta.clone(),
    };

    let browser_config = BrowserConfig {
        theme_color: palette.background.clone(),
    };

    let hyprlock_config = HyprlockConfig {
        color: strip_hash(&palette.background),
        inner_color: strip_hash(&palette.background),
        outer_color: strip_hash(&palette.accent),
        font_color: strip_hash(&palette.foreground),
        check_color: strip_hash(&palette.terminal.yellow),
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

    Ok(EditingTheme {
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
            icons: None,
            ghostty: None,
            kitty: None,
            terminal: Some(terminal_config),
        },
        is_light_theme: palette.is_light_theme,
    })
}

/// Build terminal configuration from palette
fn build_terminal_config(palette: &ColorPalette) -> TerminalConfig {
    // Generate bright variants by lightening normal colors
    let bright = TerminalPalette {
        black: lighten_color(&palette.terminal.black, 0.15),
        red: lighten_color(&palette.terminal.red, 0.15),
        green: lighten_color(&palette.terminal.green, 0.15),
        yellow: lighten_color(&palette.terminal.yellow, 0.15),
        blue: lighten_color(&palette.terminal.blue, 0.15),
        magenta: lighten_color(&palette.terminal.magenta, 0.15),
        cyan: lighten_color(&palette.terminal.cyan, 0.15),
        white: lighten_color(&palette.terminal.white, 0.1),
    };

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
            background: adjust_brightness(&palette.accent, -0.3),
            foreground: palette.foreground.clone(),
        },
        normal: palette.terminal.clone(),
        bright,
    }
}

/// Build btop configuration from palette
fn build_btop_config(palette: &ColorPalette) -> BtopConfig {
    let bg = &palette.background;
    let fg = &palette.foreground;
    let accent = &palette.accent;

    BtopConfig {
        main_bg: bg.clone(),
        main_fg: fg.clone(),
        title: adjust_brightness(fg, -0.2),
        hi_fg: accent.clone(),
        selected_bg: palette.terminal.yellow.clone(),
        selected_fg: bg.clone(),
        inactive_fg: adjust_brightness(fg, -0.5),
        proc_misc: adjust_brightness(fg, -0.3),
        cpu_box: adjust_brightness(fg, -0.4),
        mem_box: adjust_brightness(fg, -0.4),
        net_box: adjust_brightness(fg, -0.4),
        proc_box: adjust_brightness(fg, -0.4),
        div_line: adjust_brightness(fg, -0.4),
        // Gradient from green -> blue-ish accent -> red
        temp_start: palette.terminal.green.clone(),
        temp_mid: accent.clone(),
        temp_end: palette.terminal.red.clone(),
        cpu_start: palette.terminal.green.clone(),
        cpu_mid: accent.clone(),
        cpu_end: palette.terminal.red.clone(),
        free_start: palette.terminal.green.clone(),
        free_mid: accent.clone(),
        free_end: palette.terminal.red.clone(),
        cached_start: palette.terminal.green.clone(),
        cached_mid: accent.clone(),
        cached_end: palette.terminal.red.clone(),
        available_start: palette.terminal.green.clone(),
        available_mid: accent.clone(),
        available_end: palette.terminal.red.clone(),
        used_start: palette.terminal.green.clone(),
        used_mid: accent.clone(),
        used_end: palette.terminal.red.clone(),
        download_start: palette.terminal.green.clone(),
        download_mid: accent.clone(),
        download_end: palette.terminal.red.clone(),
        upload_start: palette.terminal.green.clone(),
        upload_mid: accent.clone(),
        upload_end: palette.terminal.red.clone(),
    }
}

/// Strip # from hex color
fn strip_hash(hex: &str) -> String {
    hex.trim_start_matches('#').to_string()
}

/// Lighten a hex color by a percentage (0.0 - 1.0)
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

/// Darken a hex color by a percentage
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

/// Adjust brightness: positive = lighten, negative = darken
fn adjust_brightness(hex: &str, amount: f32) -> String {
    if amount >= 0.0 {
        lighten_color(hex, amount)
    } else {
        darken_color(hex, amount.abs())
    }
}
