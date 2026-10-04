//! `icons.theme`: the Yaru icon color a theme gives GTK apps and the file
//! manager. `omarchy-theme-set-gnome` reads it from the current theme; a
//! theme without one keeps whatever icon theme is set.
use std::fs;

use crate::error::{Error, Result};
use crate::system::fs::write_atomic;
use crate::system::themes::color_utils::hex_to_rgb;
use crate::system::themes::theme_file_ops::omarchist_theme_dir;

pub const ICONS_FILE: &str = "icons.theme";

/// One Yaru color variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconTheme {
    /// The icon theme's folder name, what `icons.theme` holds.
    pub name: &'static str,
    pub label: &'static str,
    pub color: (u8, u8, u8),
}

pub const ICON_THEMES: &[IconTheme] = &[
    IconTheme {
        name: "Yaru-red",
        label: "Red",
        color: (0xe9, 0x20, 0x20),
    },
    IconTheme {
        name: "Yaru-blue",
        label: "Blue",
        color: (0x20, 0x8f, 0xe9),
    },
    IconTheme {
        name: "Yaru-olive",
        label: "Olive",
        color: (0x63, 0x6b, 0x2f),
    },
    IconTheme {
        name: "Yaru-yellow",
        label: "Yellow",
        color: (0xe9, 0xba, 0x20),
    },
    IconTheme {
        name: "Yaru-purple",
        label: "Purple",
        color: (0x5e, 0x27, 0x50),
    },
    IconTheme {
        name: "Yaru-magenta",
        label: "Magenta",
        color: (0xff, 0x00, 0xff),
    },
    IconTheme {
        name: "Yaru-sage",
        label: "Sage",
        color: (0x12, 0x3d, 0x18),
    },
];

/// The icon theme the theme names, or `None` when it ships no
/// `icons.theme`.
pub fn read(theme_name: &str) -> Result<Option<String>> {
    let path = omarchist_theme_dir(theme_name)?.join(ICONS_FILE);
    match fs::read_to_string(&path) {
        Ok(content) => Ok(Some(content.trim().to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::io("Failed to read icons.theme", e)),
    }
}

/// Writes `icons.theme`. Only the variants in [`ICON_THEMES`] are accepted:
/// they are the ones Omarchy installs.
pub fn write(theme_name: &str, icon_theme: &str) -> Result<()> {
    if !ICON_THEMES.iter().any(|t| t.name == icon_theme) {
        return Err(Error::Invalid(format!(
            "{icon_theme} is not one of the Yaru icon colors"
        )));
    }
    let path = omarchist_theme_dir(theme_name)?.join(ICONS_FILE);
    write_atomic(&path, format!("{icon_theme}\n"), ICONS_FILE)
}

/// The variant closest to `accent` (`#rrggbb`), for a theme made from an
/// image; Blue when the accent cannot be read.
pub fn closest_to(accent: &str) -> &'static str {
    let Some(accent) = hex_to_rgb(accent) else {
        return "Yaru-blue";
    };
    let distance = |c: (u8, u8, u8)| {
        let d = |a: u8, b: u8| (a as f32 - b as f32).powi(2);
        d(accent.0, c.0) + d(accent.1, c.1) + d(accent.2, c.2)
    };
    ICON_THEMES
        .iter()
        .min_by(|a, b| distance(a.color).total_cmp(&distance(b.color)))
        .map(|t| t.name)
        .unwrap_or("Yaru-blue")
}

#[cfg(test)]
mod tests {
    use super::closest_to;

    #[test]
    fn the_closest_yaru_color_is_picked_for_an_accent() {
        assert_eq!(closest_to("#ff1010"), "Yaru-red");
        assert_eq!(closest_to("#2090f0"), "Yaru-blue");
        assert_eq!(closest_to("not a color"), "Yaru-blue");
    }
}
