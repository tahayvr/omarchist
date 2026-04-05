use gpui::{Hsla, rgb};

// Parse a #RRGGBB hex string into GPUI's Hsla type.
pub fn hex_to_hsla(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(rgb(u32::from_be_bytes([0, r, g, b])).into())
}
