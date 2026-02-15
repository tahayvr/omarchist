use gpui::*;

/// Convert RGB values to GPUI Hsla color
pub fn rgb_to_hsla(r: u8, g: u8, b: u8) -> Hsla {
    let hex = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    rgb(hex).into()
}
