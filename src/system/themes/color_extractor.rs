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

fn analyze_image_type(colors: &[ColorInfo]) -> ImageType {
    let total = colors.len() as f32;
    let low_sat_count = colors.iter().filter(|c| c.hsl.saturation < 0.3).count() as f32;
    let low_sat_ratio = low_sat_count / total;

    if low_sat_ratio > 0.7 {
        return ImageType::Monochrome;
    }

    let mut hue_counts = [0u32; 12];
    for c in colors {
        if c.hsl.saturation > 0.2 {
            let hue_bucket = ((c.hsl.hue.into_degrees() / 30.0) as usize) % 12;
            hue_counts[hue_bucket] += 1;
        }
    }

    let max_hue_count = *hue_counts.iter().max().unwrap_or(&0);
    let max_hue_ratio = max_hue_count as f32 / total;

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

    let terminal = generate_terminal_palette(&normalized, false, is_light_theme);
    let bright = generate_terminal_palette(&normalized, true, is_light_theme);

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
    let min_l = 0.55;
    let max_l = 0.45;

    if bg_lightness < 0.2 && hsl.lightness < min_l {
        hsl.lightness = min_l;
    } else if bg_lightness > 0.8 && hsl.lightness > max_l {
        hsl.lightness = max_l;
    }

    let srgb: Srgb = Srgb::from_color(hsl);
    let hsv: Hsv = Hsv::from_color(hsl);

    ColorInfo {
        r: (srgb.red * 255.0) as u8,
        g: (srgb.green * 255.0) as u8,
        b: (srgb.blue * 255.0) as u8,
        hsl,
        hsv,
    }
}

fn assign_dark_theme_colors(colors: &[ColorInfo]) -> (String, String, String) {
    let background = color_to_hex(colors.first().unwrap());
    let foreground = color_to_hex(colors.last().unwrap());

    let accent_color = colors
        .iter()
        .filter(|c| c.hsl.lightness > 0.3 && c.hsl.lightness < 0.7)
        .max_by(|a, b| a.hsl.saturation.partial_cmp(&b.hsl.saturation).unwrap())
        .unwrap_or(&colors[colors.len() / 2]);
    let accent = color_to_hex(accent_color);

    (background, foreground, accent)
}

fn assign_light_theme_colors(colors: &[ColorInfo]) -> (String, String, String) {
    let background = color_to_hex(colors.last().unwrap());
    let mut foreground_info = colors.first().unwrap().clone();

    if foreground_info.hsl.lightness > 0.45 {
        foreground_info.hsl.lightness = 0.45;
        let srgb: Srgb = Srgb::from_color(foreground_info.hsl);
        foreground_info.r = (srgb.red * 255.0) as u8;
        foreground_info.g = (srgb.green * 255.0) as u8;
        foreground_info.b = (srgb.blue * 255.0) as u8;
    }
    let foreground = color_to_hex(&foreground_info);

    let accent_color = colors
        .iter()
        .filter(|c| c.hsl.lightness > 0.3 && c.hsl.lightness < 0.7)
        .max_by(|a, b| a.hsl.saturation.partial_cmp(&b.hsl.saturation).unwrap())
        .unwrap_or(&colors[colors.len() / 2]);
    let accent = color_to_hex(accent_color);

    (background, foreground, accent)
}

fn generate_terminal_palette(
    colors: &[ColorInfo],
    boost: bool,
    is_light_theme: bool,
) -> TerminalPalette {
    let mut sorted_by_lightness = colors.to_vec();
    sorted_by_lightness.sort_by(|a, b| a.hsl.lightness.partial_cmp(&b.hsl.lightness).unwrap());

    let black = sorted_by_lightness.first().unwrap();
    let white = sorted_by_lightness.last().unwrap();

    let target_hues = [
        (0.0, "red"),
        (120.0, "green"),
        (60.0, "yellow"),
        (240.0, "blue"),
        (300.0, "magenta"),
        (180.0, "cyan"),
    ];

    let mut palette = TerminalPalette::default();
    let mut used_indices = vec![false; colors.len()];

    for (target_hue, color_name) in target_hues.iter() {
        let mut best_idx = 0;
        let mut best_quality = -1.0;

        for (i, c) in colors.iter().enumerate() {
            if used_indices[i] {
                continue;
            }

            let hue_deg = c.hsl.hue.into_degrees();
            let distance = hue_distance(hue_deg, *target_hue);
            let hue_quality = 1.0 - (distance / 180.0);
            let sat_quality = c.hsl.saturation;
            let light_quality = if c.hsl.lightness > 0.3 && c.hsl.lightness < 0.7 {
                1.0
            } else {
                0.5
            };
            let quality = hue_quality * sat_quality * light_quality;

            if quality > best_quality {
                best_quality = quality;
                best_idx = i;
            }
        }

        used_indices[best_idx] = true;
        let hex = color_to_hex(&colors[best_idx]);

        match *color_name {
            "red" => palette.red = hex,
            "green" => palette.green = hex,
            "yellow" => palette.yellow = hex,
            "blue" => palette.blue = hex,
            "magenta" => palette.magenta = hex,
            "cyan" => palette.cyan = hex,
            _ => {}
        }
    }

    palette.black = if is_light_theme {
        color_to_hex(white)
    } else {
        color_to_hex(black)
    };
    palette.white = if is_light_theme {
        color_to_hex(black)
    } else {
        color_to_hex(white)
    };

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
