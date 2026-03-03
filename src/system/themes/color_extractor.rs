use std::path::Path;

use image::imageops::FilterType;
use palette::{FromColor, Hsl, Hsv, Srgb};

use crate::types::themes::TerminalPalette;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageType {
    Monochrome,
    LowDiversity,
    Chromatic,
}

#[derive(Debug, Clone)]
pub struct ColorInfo {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub hsl: Hsl,
    pub hsv: Hsv,
}

#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub terminal: TerminalPalette,
    pub bright: TerminalPalette,
    pub is_light_theme: bool,
    pub image_type: ImageType,
}

/// Synthesize a ColorInfo directly from HSL values.
fn synthesize_color(hue_deg: f32, saturation: f32, lightness: f32) -> ColorInfo {
    let hsl: Hsl = Hsl::new(hue_deg, saturation, lightness);
    let srgb: Srgb = Srgb::from_color(hsl);
    let hsv: Hsv = Hsv::from_color(hsl);
    ColorInfo {
        r: (srgb.red.clamp(0.0, 1.0) * 255.0) as u8,
        g: (srgb.green.clamp(0.0, 1.0) * 255.0) as u8,
        b: (srgb.blue.clamp(0.0, 1.0) * 255.0) as u8,
        hsl,
        hsv,
    }
}

// Return the center degree of the most-populated 30° hue bucket among chromatic colors.
// Falls back to 210° (a neutral blue) if no chromatic colors are found.
fn dominant_hue(colors: &[ColorInfo]) -> f32 {
    let mut hue_counts = [0u32; 12];
    for c in colors {
        if c.hsl.saturation > 0.2 {
            let bucket = ((c.hsl.hue.into_degrees() / 30.0) as usize) % 12;
            hue_counts[bucket] += 1;
        }
    }
    let best_bucket = hue_counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, &count)| count)
        .map(|(i, _)| i)
        .unwrap_or(7); // bucket 7 = 210° (blue-ish)
    (best_bucket as f32 * 30.0) + 15.0
}

// Classify the image based on color diversity of the extracted palette.
fn analyze_image_type(colors: &[ColorInfo]) -> ImageType {
    let total = colors.len() as f32;
    let low_sat_count = colors.iter().filter(|c| c.hsl.saturation < 0.3).count() as f32;
    let low_sat_ratio = low_sat_count / total;

    if low_sat_ratio > 0.7 {
        return ImageType::Monochrome;
    }

    let chromatic: Vec<&ColorInfo> = colors.iter().filter(|c| c.hsl.saturation > 0.2).collect();

    // Not enough distinct chromatic colors → treat as monochrome
    if chromatic.len() < 3 {
        return ImageType::Monochrome;
    }

    let chromatic_total = chromatic.len() as f32;
    let mut hue_counts = [0u32; 12];
    for c in &chromatic {
        let hue_bucket = ((c.hsl.hue.into_degrees() / 30.0) as usize) % 12;
        hue_counts[hue_bucket] += 1;
    }

    let max_hue_count = *hue_counts.iter().max().unwrap_or(&0);
    // Use chromatic-only count as denominator to avoid dilution by grays
    let max_hue_ratio = max_hue_count as f32 / chromatic_total;

    if max_hue_ratio > 0.6 {
        return ImageType::LowDiversity;
    }

    ImageType::Chromatic
}

pub fn extract_palette(image_path: &Path) -> Result<ColorPalette, String> {
    let img = image::open(image_path).map_err(|e| format!("Failed to open image: {}", e))?;

    let resized = img.resize(800, 600, FilterType::Triangle);

    let rgb_img = resized.to_rgb8();
    let buffer = rgb_img.into_raw();

    let colors = color_thief::get_palette(&buffer, color_thief::ColorFormat::Rgb, 10, 32)
        .map_err(|e| format!("Failed to extract colors: {:?}", e))?;

    let analyzed: Vec<ColorInfo> = colors
        .iter()
        .map(|c| {
            let srgb = Srgb::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0);
            let hsl: Hsl = Hsl::from_color(srgb);
            let hsv: Hsv = Hsv::from_color(srgb);
            ColorInfo {
                r: c.r,
                g: c.g,
                b: c.b,
                hsl,
                hsv,
            }
        })
        .collect();

    let image_type = analyze_image_type(&analyzed);

    let avg_luminance: f32 =
        analyzed.iter().map(|c| c.hsl.lightness).sum::<f32>() / analyzed.len() as f32;
    let is_light_theme = avg_luminance > 0.55;

    let mut sorted_by_lightness = analyzed.clone();
    sorted_by_lightness.sort_by(|a, b| a.hsl.lightness.partial_cmp(&b.hsl.lightness).unwrap());

    let bg_lightness = if is_light_theme {
        sorted_by_lightness
            .last()
            .map(|c| c.hsl.lightness)
            .unwrap_or(0.9)
    } else {
        sorted_by_lightness
            .first()
            .map(|c| c.hsl.lightness)
            .unwrap_or(0.1)
    };

    let normalized: Vec<ColorInfo> = analyzed
        .iter()
        .map(|c| normalize_color_for_readability(c, bg_lightness))
        .collect();

    let (background, foreground, accent) = if is_light_theme {
        assign_light_theme_colors(&sorted_by_lightness)
    } else {
        assign_dark_theme_colors(&sorted_by_lightness)
    };

    let terminal = generate_terminal_palette(&normalized, image_type, false, is_light_theme);
    let bright = generate_terminal_palette(&normalized, image_type, true, is_light_theme);

    Ok(ColorPalette {
        background,
        foreground,
        accent,
        terminal,
        bright,
        is_light_theme,
        image_type,
    })
}

// Boost lightness and saturation so terminal colors remain vivid and readable.
fn normalize_color_for_readability(color: &ColorInfo, bg_lightness: f32) -> ColorInfo {
    let mut hsl = color.hsl;

    if bg_lightness < 0.5 {
        // Dark theme: push lightness up and boost saturation
        if hsl.lightness < 0.45 {
            hsl.lightness = 0.50;
        }
        // Boost under-saturated colors (skip near-grays and already-vivid ones)
        if hsl.saturation > 0.05 && hsl.saturation < 0.55 {
            hsl.saturation = 0.65;
        }
    } else {
        // Light theme: push lightness down and boost saturation
        if hsl.lightness > 0.55 {
            hsl.lightness = 0.45;
        }
        if hsl.saturation > 0.05 && hsl.saturation < 0.55 {
            hsl.saturation = 0.65;
        }
    }

    let srgb: Srgb = Srgb::from_color(hsl);
    let hsv: Hsv = Hsv::from_color(hsl);

    ColorInfo {
        r: (srgb.red.clamp(0.0, 1.0) * 255.0) as u8,
        g: (srgb.green.clamp(0.0, 1.0) * 255.0) as u8,
        b: (srgb.blue.clamp(0.0, 1.0) * 255.0) as u8,
        hsl,
        hsv,
    }
}

// Desaturate a color toward neutral while preserving its hue tint.
fn desaturate_toward_neutral(color: &ColorInfo, max_saturation: f32) -> ColorInfo {
    let mut hsl = color.hsl;
    if hsl.saturation > max_saturation {
        hsl.saturation = max_saturation;
    }
    let srgb: Srgb = Srgb::from_color(hsl);
    let hsv: Hsv = Hsv::from_color(hsl);
    ColorInfo {
        r: (srgb.red.clamp(0.0, 1.0) * 255.0) as u8,
        g: (srgb.green.clamp(0.0, 1.0) * 255.0) as u8,
        b: (srgb.blue.clamp(0.0, 1.0) * 255.0) as u8,
        hsl,
        hsv,
    }
}

fn assign_dark_theme_colors(colors: &[ColorInfo]) -> (String, String, String) {
    // Colors are pre-sorted by lightness (darkest first)
    let bg_base = colors.first().unwrap();
    let fg_base = colors.last().unwrap();

    // Desaturate bg/fg to avoid heavy color casts (keep a subtle hue tint)
    let background = color_to_hex(&desaturate_toward_neutral(bg_base, 0.15));
    let foreground = color_to_hex(&desaturate_toward_neutral(fg_base, 0.15));

    // Accent: most saturated mid-lightness color; synthesize if palette is too gray
    let accent = pick_or_synthesize_accent(colors, false);

    (background, foreground, accent)
}

fn assign_light_theme_colors(colors: &[ColorInfo]) -> (String, String, String) {
    // Colors are pre-sorted by lightness (darkest first), so last = lightest
    let bg_base = colors.last().unwrap();
    let fg_base = colors.first().unwrap();

    let background = color_to_hex(&desaturate_toward_neutral(bg_base, 0.12));

    // Ensure fg is dark enough for readability on a light background
    let mut fg_info = desaturate_toward_neutral(fg_base, 0.15);
    if fg_info.hsl.lightness > 0.45 {
        let mut hsl = fg_info.hsl;
        hsl.lightness = 0.35;
        let srgb: Srgb = Srgb::from_color(hsl);
        let hsv: Hsv = Hsv::from_color(hsl);
        fg_info = ColorInfo {
            r: (srgb.red.clamp(0.0, 1.0) * 255.0) as u8,
            g: (srgb.green.clamp(0.0, 1.0) * 255.0) as u8,
            b: (srgb.blue.clamp(0.0, 1.0) * 255.0) as u8,
            hsl,
            hsv,
        };
    }
    let foreground = color_to_hex(&fg_info);

    let accent = pick_or_synthesize_accent(colors, true);

    (background, foreground, accent)
}

// Pick the most saturated mid-lightness palette color as the accent, or synthesize
// from the dominant hue when the palette is too gray.
fn pick_or_synthesize_accent(colors: &[ColorInfo], is_light_theme: bool) -> String {
    let accent_color = colors
        .iter()
        .filter(|c| c.hsl.lightness > 0.3 && c.hsl.lightness < 0.7)
        .max_by(|a, b| a.hsl.saturation.partial_cmp(&b.hsl.saturation).unwrap());

    match accent_color {
        Some(c) if c.hsl.saturation > 0.25 => color_to_hex(c),
        _ => {
            let hue = dominant_hue(colors);
            let lightness = if is_light_theme { 0.40 } else { 0.55 };
            color_to_hex(&synthesize_color(hue, 0.75, lightness))
        }
    }
}

/// Generate the 8-color ANSI terminal palette with image-type–aware logic:
///
/// - **Monochrome**: synthesize all 6 chromatic slots at standard ANSI hues with good
///   saturation; only black/white come from the palette.
/// - **LowDiversity**: try the palette first, but fall back to synthesis when no palette
///   color is within 40° of the target hue or has enough saturation.
/// - **Chromatic**: same palette-first approach with a slightly wider 45° tolerance.
fn generate_terminal_palette(
    colors: &[ColorInfo],
    image_type: ImageType,
    boost: bool,
    is_light_theme: bool,
) -> TerminalPalette {
    // Standard ANSI hue targets
    const ANSI_HUES: [(f32, &str); 6] = [
        (0.0, "red"),
        (60.0, "yellow"),
        (120.0, "green"),
        (180.0, "cyan"),
        (240.0, "blue"),
        (300.0, "magenta"),
    ];

    let target_lightness = if is_light_theme { 0.40 } else { 0.55 };

    let mut sorted_by_lightness = colors.to_vec();
    sorted_by_lightness.sort_by(|a, b| a.hsl.lightness.partial_cmp(&b.hsl.lightness).unwrap());

    let black_color = sorted_by_lightness.first().unwrap();
    let white_color = sorted_by_lightness.last().unwrap();

    // Black/white always come from the palette extremes
    let mut palette = TerminalPalette {
        black: if is_light_theme {
            color_to_hex(white_color)
        } else {
            color_to_hex(black_color)
        },
        white: if is_light_theme {
            color_to_hex(black_color)
        } else {
            color_to_hex(white_color)
        },
        ..Default::default()
    };

    match image_type {
        ImageType::Monochrome => {
            // Synthesize all 6 chromatic ANSI slots — palette is too gray to be useful
            for (hue, name) in ANSI_HUES.iter() {
                let hex = color_to_hex(&synthesize_color(*hue, 0.75, target_lightness));
                assign_ansi_color(&mut palette, name, hex);
            }
        }
        ImageType::LowDiversity | ImageType::Chromatic => {
            // Try palette first; fall back to synthesis when the match is too poor
            let hue_threshold = if image_type == ImageType::LowDiversity {
                40.0
            } else {
                45.0
            };
            let sat_threshold = if image_type == ImageType::LowDiversity {
                0.30
            } else {
                0.25
            };

            let mut used_indices = vec![false; colors.len()];

            for (target_hue, name) in ANSI_HUES.iter() {
                let mut best_idx = None;
                let mut best_quality = -1.0f32;

                for (i, c) in colors.iter().enumerate() {
                    if used_indices[i] {
                        continue;
                    }
                    let hue_deg = c.hsl.hue.into_degrees();
                    let dist = hue_distance(hue_deg, *target_hue);
                    let hue_quality = 1.0 - (dist / 180.0);
                    let sat_quality = c.hsl.saturation;
                    let light_quality = if c.hsl.lightness > 0.3 && c.hsl.lightness < 0.7 {
                        1.0
                    } else {
                        0.5
                    };
                    let quality = hue_quality * sat_quality * light_quality;

                    if quality > best_quality {
                        best_quality = quality;
                        best_idx = Some(i);
                    }
                }

                let hex = if let Some(idx) = best_idx {
                    let c = &colors[idx];
                    let dist = hue_distance(c.hsl.hue.into_degrees(), *target_hue);
                    if dist > hue_threshold || c.hsl.saturation < sat_threshold {
                        // Poor match — synthesize a vivid color at the target hue
                        color_to_hex(&synthesize_color(*target_hue, 0.75, target_lightness))
                    } else {
                        used_indices[idx] = true;
                        color_to_hex(c)
                    }
                } else {
                    color_to_hex(&synthesize_color(*target_hue, 0.75, target_lightness))
                };

                assign_ansi_color(&mut palette, name, hex);
            }
        }
    }

    if boost {
        let boost_fn = if is_light_theme {
            darken_hex
        } else {
            lighten_hex
        };
        TerminalPalette {
            black: boost_fn(&palette.black, 0.18),
            red: boost_fn(&palette.red, 0.18),
            green: boost_fn(&palette.green, 0.18),
            yellow: boost_fn(&palette.yellow, 0.18),
            blue: boost_fn(&palette.blue, 0.18),
            magenta: boost_fn(&palette.magenta, 0.18),
            cyan: boost_fn(&palette.cyan, 0.18),
            white: boost_fn(&palette.white, 0.12),
        }
    } else {
        palette
    }
}

fn assign_ansi_color(palette: &mut TerminalPalette, name: &str, hex: String) {
    match name {
        "red" => palette.red = hex,
        "yellow" => palette.yellow = hex,
        "green" => palette.green = hex,
        "cyan" => palette.cyan = hex,
        "blue" => palette.blue = hex,
        "magenta" => palette.magenta = hex,
        _ => {}
    }
}

fn hue_distance(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

fn color_to_hex(color: &ColorInfo) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

fn lighten_hex(hex: &str, amount: f32) -> String {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let new_r = ((r as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;
    let new_g = ((g as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;
    let new_b = ((b as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;

    format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b)
}

fn darken_hex(hex: &str, amount: f32) -> String {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    let new_r = (r as f32 * (1.0 - amount)) as u8;
    let new_g = (g as f32 * (1.0 - amount)) as u8;
    let new_b = (b as f32 * (1.0 - amount)) as u8;

    format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b)
}

pub fn copy_image_to_backgrounds(source_path: &Path, theme_name: &str) -> Result<String, String> {
    use crate::system::themes::theme_file_ops::{add_background_image, clear_background_images};

    clear_background_images(theme_name, false)?;

    let dest_path = add_background_image(theme_name, false, source_path)?;
    Ok(dest_path.to_string_lossy().to_string())
}
