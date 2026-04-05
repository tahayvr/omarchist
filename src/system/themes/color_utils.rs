// Shared hex color manipulation utilities used across the theme system.

// Parse a #RRGGBB hex string into an (r, g, b) tuple.
pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

// Lighten by interpolating each channel toward white. amount in [0.0, 1.0].
pub fn lighten_color(hex: &str, amount: f32) -> String {
    let (r, g, b) = hex_to_rgb(hex).unwrap_or((0, 0, 0));
    let new_r = ((r as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;
    let new_g = ((g as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;
    let new_b = ((b as f32 * (1.0 - amount)) + (255.0 * amount)) as u8;
    format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b)
}

// Darken by scaling each channel toward black. amount in [0.0, 1.0].
pub fn darken_color(hex: &str, amount: f32) -> String {
    let (r, g, b) = hex_to_rgb(hex).unwrap_or((0, 0, 0));
    let new_r = (r as f32 * (1.0 - amount)) as u8;
    let new_g = (g as f32 * (1.0 - amount)) as u8;
    let new_b = (b as f32 * (1.0 - amount)) as u8;
    format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b)
}

// Positive amount = lighten (interpolation), negative = darken (scaling).
pub fn adjust_brightness(hex: &str, amount: f32) -> String {
    if amount >= 0.0 {
        lighten_color(hex, amount)
    } else {
        darken_color(hex, amount.abs())
    }
}

// Shift all channels additively. Positive = lighter, negative = darker.
pub fn adjust_lightness(hex: &str, amount: f32) -> String {
    let hex_body = hex.trim_start_matches('#');
    if hex_body.len() < 6 {
        return format!("#{}", hex_body);
    }
    let r = u8::from_str_radix(&hex_body[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex_body[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex_body[4..6], 16).unwrap_or(0) as f32 / 255.0;
    let new_r = ((r + amount).clamp(0.0, 1.0) * 255.0) as u8;
    let new_g = ((g + amount).clamp(0.0, 1.0) * 255.0) as u8;
    let new_b = ((b + amount).clamp(0.0, 1.0) * 255.0) as u8;
    format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b)
}

pub fn lighten(hex: &str, amount: f32) -> String {
    adjust_lightness(hex, amount)
}

pub fn darken(hex: &str, amount: f32) -> String {
    adjust_lightness(hex, -amount)
}

// Append a two-character hex alpha suffix, e.g. "#RRGGBB" + "80" -> "#RRGGBB80".
pub fn with_alpha(hex: &str, alpha_hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        format!("#{}{}", &hex[..6], alpha_hex)
    } else {
        format!("#{}", hex)
    }
}

// Returns true if the color's perceived luminance (ITU-R BT.709) is below 0.5.
pub fn is_dark_color(hex: &str) -> bool {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return true;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    luminance < 0.5
}
