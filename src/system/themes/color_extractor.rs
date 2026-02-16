use std::path::Path;

use color_thief::ColorFormat;
use image::imageops::FilterType;
use palette::{FromColor, Hsl, Srgb};

use crate::types::themes::TerminalPalette;

/// Extracted color palette from an image
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub terminal: TerminalPalette,
    pub is_light_theme: bool,
}

/// Extract a smart color palette from an image
pub fn extract_palette(image_path: &Path) -> Result<ColorPalette, String> {
    let img = image::open(image_path).map_err(|e| format!("Failed to open image: {}", e))?;

    // Resize for faster processing while keeping color accuracy
    let resized = img.resize(150, 150, FilterType::Triangle);

    // Convert to RGB8 buffer for color-thief
    let rgb_img = resized.to_rgb8();
    let buffer = rgb_img.into_raw();

    // Extract 8 dominant colors
    let colors = color_thief::get_palette(&buffer, ColorFormat::Rgb, 10, 8)
        .map_err(|e| format!("Failed to extract colors: {:?}", e))?;

    // Convert to HSL for analysis
    let mut analyzed: Vec<(color_thief::Color, Hsl, f32, f32)> = colors
        .iter()
        .map(|c| {
            let srgb = Srgb::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0);
            let hsl: Hsl = Hsl::from_color(srgb);
            let luminance = hsl.lightness;
            let saturation = hsl.saturation;
            (*c, hsl, luminance, saturation)
        })
        .collect();

    // Sort by luminance (darkest first)
    analyzed.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    // Determine if this is a light theme (>50% of image is bright)
    let avg_luminance: f32 =
        analyzed.iter().map(|(_, _, lum, _)| lum).sum::<f32>() / analyzed.len() as f32;
    let is_light_theme = avg_luminance > 0.55;

    // Assign semantic colors
    let (background, foreground, accent, terminal) = if is_light_theme {
        assign_light_theme_colors(&analyzed)
    } else {
        assign_dark_theme_colors(&analyzed)
    };

    Ok(ColorPalette {
        background,
        foreground,
        accent,
        terminal,
        is_light_theme,
    })
}

fn assign_dark_theme_colors(
    colors: &[(color_thief::Color, Hsl, f32, f32)],
) -> (String, String, String, TerminalPalette) {
    // For dark themes:
    // - background: darkest color
    // - foreground: lightest color
    // - accent: most saturated mid-tone

    let background = rgb_to_hex(&colors[0].0);
    let foreground = rgb_to_hex(&colors.last().unwrap().0);

    // Find accent: most saturated color in mid-luminance range
    let accent_color = colors
        .iter()
        .filter(|(_, _, lum, _)| *lum > 0.3 && *lum < 0.7)
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap())
        .map(|(c, _, _, _)| *c)
        .unwrap_or(colors[colors.len() / 2].0);
    let accent = rgb_to_hex(&accent_color);

    // Generate terminal colors
    let terminal = generate_terminal_palette(colors);

    (background, foreground, accent, terminal)
}

fn assign_light_theme_colors(
    colors: &[(color_thief::Color, Hsl, f32, f32)],
) -> (String, String, String, TerminalPalette) {
    // For light themes:
    // - background: lightest color
    // - foreground: darkest color
    // - accent: most saturated mid-tone

    let background = rgb_to_hex(&colors.last().unwrap().0);
    let foreground = rgb_to_hex(&colors[0].0);

    // Find accent: most saturated color
    let accent_color = colors
        .iter()
        .filter(|(_, _, lum, _)| *lum > 0.3 && *lum < 0.7)
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap())
        .map(|(c, _, _, _)| *c)
        .unwrap_or(colors[colors.len() / 2].0);
    let accent = rgb_to_hex(&accent_color);

    let terminal = generate_terminal_palette(colors);

    (background, foreground, accent, terminal)
}

fn generate_terminal_palette(colors: &[(color_thief::Color, Hsl, f32, f32)]) -> TerminalPalette {
    // We need to assign 8 colors: black, red, green, yellow, blue, magenta, cyan, white

    let mut assigned = vec![false; colors.len()];
    let mut terminal: Vec<String> = vec!["".to_string(); 8];

    // Step 1: Assign black and white - always use darkest and lightest
    terminal[0] = rgb_to_hex(&colors[0].0); // black = darkest
    terminal[7] = rgb_to_hex(&colors.last().unwrap().0); // white = lightest
    assigned[0] = true;
    assigned[colors.len() - 1] = true;

    // Step 2: Find colors closest to pure hues for accent colors
    let target_hues = [
        (0.0, 1),   // Red (hue ~0)
        (120.0, 2), // Green (hue ~120)
        (60.0, 3),  // Yellow (hue ~60)
        (240.0, 4), // Blue (hue ~240)
        (300.0, 5), // Magenta (hue ~300)
        (180.0, 6), // Cyan (hue ~180)
    ];

    for (target_hue, color_idx) in target_hues.iter() {
        let mut best_match: Option<(usize, f32)> = None;

        for (i, (_, hsl, _, _)) in colors.iter().enumerate() {
            if assigned[i] {
                continue;
            }

            // Calculate hue distance
            let hue_deg = hsl.hue.into_degrees();
            let distance = hue_distance(hue_deg, *target_hue);

            // Prefer saturated, mid-brightness colors
            let quality = (1.0 - distance / 180.0) * hsl.saturation;

            if best_match.is_none() || quality > best_match.unwrap().1 {
                best_match = Some((i, quality));
            }
        }

        if let Some((idx, _)) = best_match {
            terminal[*color_idx] = rgb_to_hex(&colors[idx].0);
            assigned[idx] = true;
        }
    }

    // Step 3: Fill any unassigned slots with derived colors
    for term_color in terminal.iter_mut() {
        if term_color.is_empty() {
            // Use the accent color with some transformation
            let accent_idx = colors.len() / 2;
            let base = &colors[accent_idx].0;
            *term_color = rgb_to_hex(base);
        }
    }

    TerminalPalette {
        black: terminal[0].clone(),
        red: terminal[1].clone(),
        green: terminal[2].clone(),
        yellow: terminal[3].clone(),
        blue: terminal[4].clone(),
        magenta: terminal[5].clone(),
        cyan: terminal[6].clone(),
        white: terminal[7].clone(),
    }
}

fn hue_distance(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

fn rgb_to_hex(color: &color_thief::Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

/// Copy image to theme backgrounds folder
pub fn copy_image_to_backgrounds(source_path: &Path, theme_name: &str) -> Result<String, String> {
    use crate::system::themes::theme_file_ops::add_background_image;

    let dest_path = add_background_image(theme_name, false, source_path)?;
    Ok(dest_path.to_string_lossy().to_string())
}
