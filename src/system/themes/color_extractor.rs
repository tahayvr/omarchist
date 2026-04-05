use std::path::Path;

use image::imageops::FilterType;
use palette::{FromColor, Hsl, Hsv, Srgb};

use crate::system::themes::color_utils::{darken_color, lighten_color};
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

// Synthesize a ColorInfo directly from HSL values.
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
    sorted_by_lightness.sort_by(|a, b| a.hsl.lightness.total_cmp(&b.hsl.lightness));

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

fn normalize_color_for_readability(color: &ColorInfo, bg_lightness: f32) -> ColorInfo {
    let mut hsl = color.hsl;

    if hsl.saturation < 0.12 {
        let srgb: Srgb = Srgb::from_color(hsl);
        let hsv: Hsv = Hsv::from_color(hsl);
        return ColorInfo {
            r: (srgb.red.clamp(0.0, 1.0) * 255.0) as u8,
            g: (srgb.green.clamp(0.0, 1.0) * 255.0) as u8,
            b: (srgb.blue.clamp(0.0, 1.0) * 255.0) as u8,
            hsl,
            hsv,
        };
    }

    if bg_lightness < 0.5 {
        // Dark theme: keep lightness in [0.45, 0.75] to stay vivid against a dark bg
        hsl.lightness = hsl.lightness.clamp(0.45, 0.75);
        // Boost under-saturated chromatic colors so they don't look muddy
        if hsl.saturation < 0.45 {
            hsl.saturation = (hsl.saturation + 0.25).min(0.80);
        }
    } else {
        // Light theme: keep lightness in [0.28, 0.55] to stay vivid against a light bg
        hsl.lightness = hsl.lightness.clamp(0.28, 0.55);
        if hsl.saturation < 0.45 {
            hsl.saturation = (hsl.saturation + 0.25).min(0.80);
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
    let Some(bg_base) = colors.first() else {
        return (
            String::from("#1e1e2e"),
            String::from("#cdd6f4"),
            String::from("#89b4fa"),
        );
    };
    let Some(fg_base) = colors.last() else {
        return (
            String::from("#1e1e2e"),
            String::from("#cdd6f4"),
            String::from("#89b4fa"),
        );
    };

    // Desaturate bg/fg to avoid heavy color casts (keep a subtle hue tint)
    let background = color_to_hex(&desaturate_toward_neutral(bg_base, 0.15));
    let foreground = color_to_hex(&desaturate_toward_neutral(fg_base, 0.15));

    // Accent: most saturated mid-lightness color; synthesize if palette is too gray
    let accent = pick_or_synthesize_accent(colors, false);

    (background, foreground, accent)
}

fn assign_light_theme_colors(colors: &[ColorInfo]) -> (String, String, String) {
    // Colors are pre-sorted by lightness (darkest first), so last = lightest
    let Some(bg_base) = colors.last() else {
        return (
            String::from("#eff1f5"),
            String::from("#4c4f69"),
            String::from("#1e66f5"),
        );
    };
    let Some(fg_base) = colors.first() else {
        return (
            String::from("#eff1f5"),
            String::from("#4c4f69"),
            String::from("#1e66f5"),
        );
    };

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

fn pick_or_synthesize_accent(colors: &[ColorInfo], is_light_theme: bool) -> String {
    let target_lightness = if is_light_theme { 0.40 } else { 0.55 };

    // First preference: saturated color already in a readable lightness range
    let mid_accent = colors
        .iter()
        .filter(|c| c.hsl.lightness > 0.25 && c.hsl.lightness < 0.75)
        .max_by(|a, b| a.hsl.saturation.total_cmp(&b.hsl.saturation));

    if let Some(c) = mid_accent
        && c.hsl.saturation > 0.20
    {
        return color_to_hex(c);
    }

    // Second preference: most saturated color anywhere in the palette, lightness adjusted
    let best = colors
        .iter()
        .max_by(|a, b| a.hsl.saturation.total_cmp(&b.hsl.saturation));

    if let Some(c) = best
        && c.hsl.saturation > 0.15
    {
        // Keep the image hue, re-pin lightness to a readable range
        return color_to_hex(&synthesize_color(
            c.hsl.hue.into_degrees(),
            c.hsl.saturation,
            target_lightness,
        ));
    }

    // Last resort (truly monochrome images): synthesize from the dominant hue
    let hue = dominant_hue(colors);
    color_to_hex(&synthesize_color(hue, 0.72, target_lightness))
}

// Generate the 8-color ANSI terminal palette from image colors.
// Only synthesizes when the image has fewer chromatic colors than slots
// to fill (Monochrome, or near-monochrome images).
fn generate_terminal_palette(
    colors: &[ColorInfo],
    image_type: ImageType,
    boost: bool,
    is_light_theme: bool,
) -> TerminalPalette {
    // ANSI (red=0°, yellow=60°, green=120°, cyan=180°,
    // blue=240°, magenta=300°).
    const ANSI_SLOTS: [(&str, f32); 6] = [
        ("red", 0.0),
        ("yellow", 60.0),
        ("green", 120.0),
        ("cyan", 180.0),
        ("blue", 240.0),
        ("magenta", 300.0),
    ];

    let target_lightness = if is_light_theme { 0.42 } else { 0.58 };

    let mut sorted_by_lightness = colors.to_vec();
    sorted_by_lightness.sort_by(|a, b| a.hsl.lightness.total_cmp(&b.hsl.lightness));

    let Some(black_color) = sorted_by_lightness.first() else {
        return TerminalPalette::default();
    };
    let Some(white_color) = sorted_by_lightness.last() else {
        return TerminalPalette::default();
    };

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

    if image_type == ImageType::Monochrome {
        // Palette is too gray — synthesize all 6 chromatic slots at standard hues
        for (name, hue) in ANSI_SLOTS.iter() {
            let hex = color_to_hex(&synthesize_color(*hue, 0.72, target_lightness));
            assign_ansi_color(&mut palette, name, hex);
        }
    } else {
        // Collect chromatic colors (meaningful saturation), sorted by hue angle
        let mut chromatic: Vec<ColorInfo> = colors
            .iter()
            .filter(|c| c.hsl.saturation > 0.15)
            .cloned()
            .collect();

        chromatic.sort_by(|a, b| {
            a.hsl
                .hue
                .into_degrees()
                .total_cmp(&b.hsl.hue.into_degrees())
        });

        if chromatic.is_empty() {
            // Edge case: nothing usable — synthesize everything
            for (name, hue) in ANSI_SLOTS.iter() {
                let hex = color_to_hex(&synthesize_color(*hue, 0.72, target_lightness));
                assign_ansi_color(&mut palette, name, hex);
            }
        } else {
            // Assign each ANSI slot the image color closest in hue
            for (name, target_hue) in ANSI_SLOTS.iter() {
                let Some(best) = chromatic.iter().min_by(|a, b| {
                    let da = hue_distance(a.hsl.hue.into_degrees(), *target_hue);
                    let db = hue_distance(b.hsl.hue.into_degrees(), *target_hue);
                    da.total_cmp(&db)
                }) else {
                    continue;
                };

                // Borrow only what we need before calling synthesize_color
                let best_hue = best.hsl.hue.into_degrees();
                let best_sat = best.hsl.saturation;

                // Re-pin lightness to a readable range while keeping the image's hue
                let readable = synthesize_color(best_hue, best_sat.max(0.55), target_lightness);
                assign_ansi_color(&mut palette, name, color_to_hex(&readable));
            }
        }
    }

    if boost {
        let boost_fn = if is_light_theme {
            darken_color
        } else {
            lighten_color
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

pub fn copy_image_to_backgrounds(source_path: &Path, theme_name: &str) -> Result<String, String> {
    use crate::system::themes::theme_file_ops::{add_background_image, clear_background_images};

    clear_background_images(theme_name, false)?;

    let dest_path = add_background_image(theme_name, false, source_path)?;
    Ok(dest_path.to_string_lossy().to_string())
}
