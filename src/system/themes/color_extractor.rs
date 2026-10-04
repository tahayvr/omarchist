use crate::error::{Error, Result};
use std::path::Path;

use image::imageops::FilterType;
use palette::{FromColor, Hsl, Srgb};

use crate::system::themes::color_utils::{darken_color, lighten_color, mix_hex};
use crate::types::themes::TerminalPalette;

// How the image's color distribution shapes the ANSI palette: a monochrome
// image gets all six chromatic slots synthesized, anything else gets image
// hues where they exist and synthesized fills where they don't.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ImageType {
    Monochrome,
    Chromatic,
}

#[derive(Debug, Clone)]
struct ColorInfo {
    r: u8,
    g: u8,
    b: u8,
    hsl: Hsl,
}

impl ColorInfo {
    fn from_hsl(hsl: Hsl) -> Self {
        let srgb: Srgb = Srgb::from_color(hsl);
        Self {
            r: (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
            g: (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
            b: (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
            hsl,
        }
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        Self {
            r,
            g,
            b,
            hsl: Hsl::from_color(srgb),
        }
    }

    // Hue in [0, 360). `palette` reports hues in (-180, 180], which breaks
    // bucket arithmetic and hue-distance comparisons if used directly.
    fn hue(&self) -> f32 {
        self.hsl.hue.into_positive_degrees()
    }

    fn lightness(&self) -> f32 {
        self.hsl.lightness
    }

    fn saturation(&self) -> f32 {
        self.hsl.saturation
    }

    fn with_lightness(&self, lightness: f32) -> Self {
        let mut hsl = self.hsl;
        hsl.lightness = lightness;
        Self::from_hsl(hsl)
    }

    fn with_max_saturation(&self, max_saturation: f32) -> Self {
        let mut hsl = self.hsl;
        hsl.saturation = hsl.saturation.min(max_saturation);
        Self::from_hsl(hsl)
    }

    fn hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

// What the theme generator consumes. `black`/`white` in both palettes are
// derived from background/foreground rather than the image extremes: Omarchy's
// resolver overwrites color0/color7 with background/foreground anyway, and
// maps color8 to `muted` and color15 to `bright_foreground`, so those two are
// the only slots whose values actually reach the desktop.
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub terminal: TerminalPalette,
    pub bright: TerminalPalette,
    pub is_light_theme: bool,
}

fn synthesize(hue_deg: f32, saturation: f32, lightness: f32) -> ColorInfo {
    ColorInfo::from_hsl(Hsl::new(hue_deg, saturation, lightness))
}

fn hue_distance(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs() % 360.0;
    diff.min(360.0 - diff)
}

fn hue_bucket(color: &ColorInfo) -> usize {
    ((color.hue() / 30.0) as usize) % 12
}

// Center of the most-populated 30° hue bucket among chromatic colors,
// falling back to a neutral blue when nothing is chromatic.
fn dominant_hue(colors: &[ColorInfo]) -> f32 {
    let mut counts = [0u32; 12];
    for c in colors.iter().filter(|c| c.saturation() > 0.2) {
        counts[hue_bucket(c)] += 1;
    }
    let best = counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, &n)| n)
        .filter(|&(_, &n)| n > 0)
        .map(|(i, _)| i)
        .unwrap_or(7);
    best as f32 * 30.0 + 15.0
}

fn analyze_image_type(colors: &[ColorInfo]) -> ImageType {
    let total = colors.len().max(1) as f32;
    let low_sat = colors.iter().filter(|c| c.saturation() < 0.3).count() as f32;
    let chromatic = colors.iter().filter(|c| c.saturation() > 0.2).count();

    if low_sat / total > 0.7 || chromatic < 3 {
        ImageType::Monochrome
    } else {
        ImageType::Chromatic
    }
}

// Pixel-weighted mean luminance (BT.709) of the RGB buffer. Deciding light
// vs dark from an unweighted mean of the quantized palette let a bright sky
// behind a dark subject flip the whole theme.
fn mean_luminance(rgb: &[u8]) -> f32 {
    if rgb.len() < 3 {
        return 0.0;
    }
    let (pixels, _) = rgb.as_chunks::<3>();
    let sum: f64 = pixels
        .iter()
        .map(|p| {
            0.2126 * p[0] as f64 / 255.0
                + 0.7152 * p[1] as f64 / 255.0
                + 0.0722 * p[2] as f64 / 255.0
        })
        .sum();
    (sum / pixels.len() as f64) as f32
}

pub fn extract_palette(image_path: &Path) -> Result<ColorPalette> {
    let img = image::open(image_path)
        .map_err(|e| Error::Invalid(format!("Failed to open image: {}", e)))?;
    let resized = img.resize(800, 600, FilterType::Triangle);
    // Composite any alpha onto white first so transparent regions don't read
    // as black and drag the theme dark.
    let rgb_img = if resized.color().has_alpha() {
        let rgba = resized.to_rgba8();
        let mut out = image::RgbImage::new(rgba.width(), rgba.height());
        for (o, p) in out.pixels_mut().zip(rgba.pixels()) {
            let a = p[3] as f32 / 255.0;
            let blend = |c: u8| (c as f32 * a + 255.0 * (1.0 - a)).round() as u8;
            *o = image::Rgb([blend(p[0]), blend(p[1]), blend(p[2])]);
        }
        out
    } else {
        resized.to_rgb8()
    };
    let buffer = rgb_img.into_raw();

    let quantized = color_thief::get_palette(&buffer, color_thief::ColorFormat::Rgb, 10, 32)
        .map_err(|e| Error::Invalid(format!("Failed to extract colors: {:?}", e)))?;

    let colors: Vec<ColorInfo> = quantized
        .iter()
        .map(|c| ColorInfo::from_rgb(c.r, c.g, c.b))
        .collect();

    if colors.is_empty() {
        return Err(Error::Invalid(
            "No colors could be extracted from the image".into(),
        ));
    }

    let is_light_theme = mean_luminance(&buffer) > 0.55;
    let image_type = analyze_image_type(&colors);

    let mut by_lightness = colors.clone();
    by_lightness.sort_by(|a, b| a.lightness().total_cmp(&b.lightness()));

    let (background, foreground) = if is_light_theme {
        assign_light_base(&by_lightness)
    } else {
        assign_dark_base(&by_lightness)
    };
    let accent = pick_or_synthesize_accent(&by_lightness, is_light_theme);

    let normalized: Vec<ColorInfo> = colors
        .iter()
        .map(|c| normalize_for_readability(c, is_light_theme))
        .collect();
    let chroma = chromatic_slots(&normalized, image_type, is_light_theme);

    // color8 -> `muted`, color15 -> `bright_foreground` in Omarchy's resolver.
    // The ratios match the stock themes (tokyo-night, flexoki-light).
    let muted = mix_hex(&background, &foreground, 0.27);
    let bright_foreground = if is_light_theme {
        foreground.clone()
    } else {
        mix_hex(&foreground, "#ffffff", 0.25)
    };
    let boost = if is_light_theme {
        darken_color
    } else {
        lighten_color
    };

    let terminal = TerminalPalette {
        black: background.clone(),
        red: chroma[0].clone(),
        yellow: chroma[1].clone(),
        green: chroma[2].clone(),
        cyan: chroma[3].clone(),
        blue: chroma[4].clone(),
        magenta: chroma[5].clone(),
        white: foreground.clone(),
    };
    let bright = TerminalPalette {
        black: muted,
        red: boost(&chroma[0], 0.18),
        yellow: boost(&chroma[1], 0.18),
        green: boost(&chroma[2], 0.18),
        cyan: boost(&chroma[3], 0.18),
        blue: boost(&chroma[4], 0.18),
        magenta: boost(&chroma[5], 0.18),
        white: bright_foreground,
    };

    Ok(ColorPalette {
        background,
        foreground,
        accent,
        terminal,
        bright,
        is_light_theme,
    })
}

// Keep chromatic colors in a lightness band that reads against the theme
// background, and lift muddy saturation. Near-greys pass through untouched.
fn normalize_for_readability(color: &ColorInfo, is_light_theme: bool) -> ColorInfo {
    if color.saturation() < 0.12 {
        return color.clone();
    }
    let mut hsl = color.hsl;
    hsl.lightness = if is_light_theme {
        hsl.lightness.clamp(0.28, 0.55)
    } else {
        hsl.lightness.clamp(0.45, 0.75)
    };
    if hsl.saturation < 0.45 {
        hsl.saturation = (hsl.saturation + 0.25).min(0.80);
    }
    ColorInfo::from_hsl(hsl)
}

// Dark theme: background from the darkest image color, foreground from the
// lightest, both nearly neutral and pinned to a contrast-safe lightness.
fn assign_dark_base(by_lightness: &[ColorInfo]) -> (String, String) {
    let (Some(darkest), Some(lightest)) = (by_lightness.first(), by_lightness.last()) else {
        return ("#1e1e2e".to_string(), "#cdd6f4".to_string());
    };
    let mut bg = darkest.with_max_saturation(0.15);
    if bg.lightness() > 0.16 {
        bg = bg.with_lightness(0.12);
    }
    let mut fg = lightest.with_max_saturation(0.15);
    if fg.lightness() < 0.78 {
        fg = fg.with_lightness(0.82);
    }
    (bg.hex(), fg.hex())
}

fn assign_light_base(by_lightness: &[ColorInfo]) -> (String, String) {
    let (Some(darkest), Some(lightest)) = (by_lightness.first(), by_lightness.last()) else {
        return ("#eff1f5".to_string(), "#4c4f69".to_string());
    };
    let mut bg = lightest.with_max_saturation(0.12);
    if bg.lightness() < 0.92 {
        bg = bg.with_lightness(0.95);
    }
    let mut fg = darkest.with_max_saturation(0.15);
    if fg.lightness() > 0.35 {
        fg = fg.with_lightness(0.30);
    }
    (bg.hex(), fg.hex())
}

fn accent_lightness_range(is_light_theme: bool) -> (f32, f32) {
    if is_light_theme {
        (0.30, 0.55)
    } else {
        (0.45, 0.75)
    }
}

fn pick_or_synthesize_accent(colors: &[ColorInfo], is_light_theme: bool) -> String {
    let (lo, hi) = accent_lightness_range(is_light_theme);
    let target = (lo + hi) / 2.0;

    // Most saturated color that already sits in a readable band.
    if let Some(c) = colors
        .iter()
        .filter(|c| c.lightness() > 0.25 && c.lightness() < 0.75)
        .max_by(|a, b| a.saturation().total_cmp(&b.saturation()))
        .filter(|c| c.saturation() > 0.20)
    {
        return c.with_lightness(c.lightness().clamp(lo, hi)).hex();
    }

    // Otherwise the most saturated color anywhere, re-pinned to the band.
    if let Some(c) = colors
        .iter()
        .max_by(|a, b| a.saturation().total_cmp(&b.saturation()))
        .filter(|c| c.saturation() > 0.15)
    {
        return synthesize(c.hue(), c.saturation(), target).hex();
    }

    // Truly monochrome: synthesize from the dominant hue.
    synthesize(dominant_hue(colors), 0.60, target).hex()
}

// ANSI order: red, yellow, green, cyan, blue, magenta. The lightness offsets
// keep yellow/red bright and cyan darker the way hand-made themes do instead
// of pinning all six to one value.
const ANSI_SLOTS: [(f32, f32); 6] = [
    (0.0, 0.04),
    (60.0, 0.06),
    (120.0, 0.00),
    (180.0, -0.05),
    (240.0, 0.04),
    (300.0, 0.04),
];

// Beyond this hue distance an image color no longer reads as that ANSI
// color, so the slot is synthesized at its canonical hue instead of reusing
// the same image color for several slots.
const MAX_SLOT_HUE_DISTANCE: f32 = 40.0;

fn chromatic_slots(
    colors: &[ColorInfo],
    image_type: ImageType,
    is_light_theme: bool,
) -> [String; 6] {
    let base_lightness = if is_light_theme { 0.42 } else { 0.58 };

    let mut chromatic: Vec<&ColorInfo> = colors.iter().filter(|c| c.saturation() > 0.15).collect();
    if image_type == ImageType::Monochrome {
        chromatic.clear();
    }

    // Saturation for synthesized slots follows the image so fills blend in;
    // a monochrome image gets a moderate, non-neon default.
    let fill_saturation = if chromatic.is_empty() {
        0.55
    } else {
        let mut sats: Vec<f32> = chromatic.iter().map(|c| c.saturation()).collect();
        sats.sort_by(|a, b| a.total_cmp(b));
        sats[sats.len() / 2].clamp(0.45, 0.75)
    };

    ANSI_SLOTS.map(|(target_hue, offset)| {
        let lightness = base_lightness + offset;
        let nearest = chromatic
            .iter()
            .min_by(|a, b| {
                hue_distance(a.hue(), target_hue).total_cmp(&hue_distance(b.hue(), target_hue))
            })
            .filter(|c| hue_distance(c.hue(), target_hue) <= MAX_SLOT_HUE_DISTANCE);

        match nearest {
            Some(c) => synthesize(c.hue(), c.saturation().clamp(0.35, 0.80), lightness).hex(),
            None => synthesize(target_hue, fill_saturation, lightness).hex(),
        }
    })
}

pub fn copy_image_to_backgrounds(source_path: &Path, theme_name: &str) -> Result<String> {
    use crate::system::themes::theme_file_ops::{add_background_image, clear_background_images};

    clear_background_images(theme_name, false)?;

    let dest_path = add_background_image(theme_name, false, source_path)?;
    Ok(dest_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::themes::color_utils::hex_to_rgb;

    fn lightness_of(hex: &str) -> f32 {
        let (r, g, b) = hex_to_rgb(hex).unwrap();
        ColorInfo::from_rgb(r, g, b).lightness()
    }

    fn saturation_of(hex: &str) -> f32 {
        let (r, g, b) = hex_to_rgb(hex).unwrap();
        ColorInfo::from_rgb(r, g, b).saturation()
    }

    fn write_test_image(name: &str, paint: impl Fn(u32, u32) -> [u8; 3]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("omarchist-extract-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let img = image::RgbImage::from_fn(96, 96, |x, y| image::Rgb(paint(x, y)));
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn hues_are_bucketed_on_the_positive_circle() {
        let blue = ColorInfo::from_rgb(0, 0, 255);
        assert!((blue.hue() - 240.0).abs() < 0.5);
        assert_eq!(hue_bucket(&blue), 8);
        let magenta = ColorInfo::from_rgb(255, 0, 255);
        assert_eq!(hue_bucket(&magenta), 10);
        assert_eq!(dominant_hue(&[blue.clone(), blue, magenta]), 255.0);
        assert_eq!(hue_distance(350.0, 10.0), 20.0);
    }

    #[test]
    fn two_hue_image_does_not_collapse_ansi_slots() {
        // Dark navy ground with orange and teal blobs.
        let path = write_test_image("two-hue.png", |x, y| match (x / 16 + y / 16) % 3 {
            0 => [230, 120, 40],
            1 => [40, 170, 170],
            _ => [12, 14, 30],
        });
        let p = extract_palette(&path).unwrap();
        assert!(!p.is_light_theme);
        let t = &p.terminal;
        let slots = [&t.red, &t.yellow, &t.green, &t.cyan, &t.blue, &t.magenta];
        for (i, a) in slots.iter().enumerate() {
            for b in &slots[i + 1..] {
                assert_ne!(a, b, "ANSI slots must not share one image color");
            }
        }
        assert!(
            lightness_of(&p.background) <= 0.16,
            "dark bg must stay dark"
        );
        assert!(
            lightness_of(&p.foreground) >= 0.78,
            "dark fg must stay light"
        );
        assert!(
            saturation_of(&p.bright.black) < 0.2,
            "muted must be near-neutral"
        );
        assert_eq!(t.black, p.background);
        assert_eq!(t.white, p.foreground);
    }

    #[test]
    fn light_image_with_dark_subject_stays_light() {
        // Mostly bright cream with a dark stripe: pixel-weighted luminance
        // must win over the palette's unweighted mean.
        let path = write_test_image("light.png", |_, y| {
            if y % 8 == 0 {
                [20, 20, 30]
            } else {
                [250, 246, 236]
            }
        });
        let p = extract_palette(&path).unwrap();
        assert!(p.is_light_theme);
        assert!(lightness_of(&p.background) >= 0.92);
        assert!(lightness_of(&p.foreground) <= 0.35);
        assert_eq!(p.bright.white, p.foreground);
    }

    #[test]
    fn monochrome_image_synthesizes_moderate_colors() {
        let path = write_test_image("grey.png", |x, _| {
            let v = 40 + (x * 2) as u8;
            [v, v, v]
        });
        let p = extract_palette(&path).unwrap();
        for hex in [&p.terminal.red, &p.terminal.green, &p.terminal.blue] {
            let s = saturation_of(hex);
            assert!(
                (0.4..=0.7).contains(&s),
                "synthesized saturation {s} should not be neon"
            );
        }
        assert_ne!(
            lightness_of(&p.terminal.yellow),
            lightness_of(&p.terminal.cyan)
        );
    }
}
