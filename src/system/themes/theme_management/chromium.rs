use crate::error::{Error, Result};
use std::fs;
use std::path::Path;

use crate::types::themes::{BrowserConfig, ColorsConfig};

use super::paths::get_custom_themes_dir;

// What Quattro's `chromium.theme.tpl` would generate (`{{ background_rgb }}`),
// used to seed the override when the user turns it on.
pub fn default_browser_config(colors: &ColorsConfig) -> BrowserConfig {
    BrowserConfig {
        theme_color: colors.background.clone(),
    }
}

pub fn update_chromium_config(theme_name: &str, config: &BrowserConfig) -> Result<()> {
    let themes_dir = get_custom_themes_dir().ok_or(Error::UnknownDirectory("custom themes"))?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(Error::ThemeNotFound(theme_name.to_string()));
    }

    let hex = config.theme_color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(15);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(15);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(25);

    let theme_path = theme_dir.join("chromium.theme");
    fs::write(&theme_path, format!("{},{},{}\n", r, g, b))
        .map_err(|e| Error::io("Failed to write chromium.theme", e))?;

    Ok(())
}

// Reads a `chromium.theme` (`R,G,B` decimal, as Omarchy's browser policy
// expects) back into a hex `BrowserConfig`. Returns None when the file is
// absent or malformed, which the caller treats as "override off".
pub(super) fn parse_chromium_theme_file(path: &Path) -> Option<BrowserConfig> {
    let content = fs::read_to_string(path).ok()?;
    let mut parts = content
        .trim()
        .split(',')
        .map(|p| p.trim().parse::<u8>().ok());
    let (r, g, b) = (parts.next()??, parts.next()??, parts.next()??);
    Some(BrowserConfig {
        theme_color: format!("#{:02X}{:02X}{:02X}", r, g, b),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_browser_config_uses_background() {
        let colors = ColorsConfig::default();
        assert_eq!(
            default_browser_config(&colors).theme_color,
            colors.background
        );
    }

    #[test]
    fn parse_chromium_theme_file_round_trips_rgb() {
        let dir = std::env::temp_dir().join(format!("omarchist-chromium-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("chromium.theme");
        std::fs::write(&path, "15,15,25\n").unwrap();
        let parsed = parse_chromium_theme_file(&path).expect("valid file parses");
        assert_eq!(parsed.theme_color, "#0F0F19");
        std::fs::write(&path, "garbage").unwrap();
        assert!(parse_chromium_theme_file(&path).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
