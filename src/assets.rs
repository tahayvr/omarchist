use gpui::*;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::path::Path;

#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "**/*"]
pub struct OmarchistAssets;

// Embedded copy of the `defaults/` directory tree, compiled into the binary at build time.
// Paths are relative to the `defaults/` folder root (e.g. `"omarchist/settings.json"`).
#[derive(RustEmbed)]
#[folder = "./defaults"]
#[include = "**/*"]
pub struct DefaultAssets;

// Returns the content of an embedded default file as a UTF-8 string.
// `path` is relative to the `defaults/` folder (e.g. `"omarchist/settings.json"`).
pub fn read_default_str(path: &str) -> crate::error::Result<String> {
    let file = DefaultAssets::get(path).ok_or_else(|| {
        crate::error::Error::Invalid(format!("Embedded default not found: '{path}'"))
    })?;
    std::str::from_utf8(&file.data)
        .map(|s| s.to_string())
        .map_err(|e| {
            crate::error::Error::Invalid(format!(
                "Embedded default '{}' is not valid UTF-8: {}",
                path, e
            ))
        })
}

// Extracts all embedded files whose path begins with `prefix/` and writes them under `dest`.
// For example, `extract_default_dir("theme", Path::new("/home/user/.config/omarchy/themes/my-theme"))`
// will recreate the subtree at `dest`, stripping the `theme/` prefix from each file's embedded path.
pub fn extract_default_dir(prefix: &str, dest: &Path) -> crate::error::Result<()> {
    let prefix_slash = format!("{}/", prefix.trim_end_matches('/'));
    let mut extracted = false;

    for cow_path in DefaultAssets::iter() {
        let embedded_path: &str = &cow_path;
        if !embedded_path.starts_with(&prefix_slash) {
            continue;
        }

        let relative = &embedded_path[prefix_slash.len()..];
        let dest_path = dest.join(relative);

        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::error::Error::io(format!("Failed to create directory '{:?}'", parent), e)
            })?;
        }

        let file = DefaultAssets::get(embedded_path).unwrap();
        std::fs::write(&dest_path, file.data).map_err(|e| {
            crate::error::Error::io(format!("Failed to write '{:?}'", dest_path), e)
        })?;

        extracted = true;
    }

    if !extracted {
        return Err(crate::error::Error::Invalid(format!(
            "No embedded defaults found under prefix '{}'",
            prefix
        )));
    }

    Ok(())
}

impl AssetSource for OmarchistAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Ok(Self::get(path).map(|f| f.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

// Combined asset source that tries OmarchistAssets first, then gpui_component_assets
pub struct CombinedAssets {
    omarchist: OmarchistAssets,
    gpui_component: gpui_component_assets::Assets,
}

impl Default for CombinedAssets {
    fn default() -> Self {
        Self::new()
    }
}

impl CombinedAssets {
    pub fn new() -> Self {
        Self {
            omarchist: OmarchistAssets,
            gpui_component: gpui_component_assets::Assets,
        }
    }
}

impl AssetSource for CombinedAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        // Try OmarchistAssets first
        if let Some(data) = self.omarchist.load(path)? {
            return Ok(Some(data));
        }

        // Fall back to gpui_component_assets
        self.gpui_component.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut results = self.omarchist.list(path)?;
        results.extend(self.gpui_component.list(path)?);
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultAssets, read_default_str};
    use crate::types::themes::EditingTheme;

    #[test]
    fn embedded_defaults_match_quattro_theme_layout() {
        let embedded: Vec<String> = DefaultAssets::iter().map(|p| p.to_string()).collect();

        for required in [
            "omarchist/settings.json",
            "theme/omarchist.json",
            "theme/colors.toml",
            "theme/icons.theme",
        ] {
            assert!(
                embedded.iter().any(|p| p == required),
                "expected embedded default '{required}', have: {embedded:?}"
            );
        }

        // Pre-Quattro leftovers must not come back, and override files are
        // opt-in: Omarchy generates all of these from colors.toml.
        for legacy in [
            "light.mode",
            "theme/btop.theme",
            "theme/chromium.theme",
            "theme/shell.lock.toml",
            "omarchist/hyprland/hyprland.conf",
            "theme/neovim.lua",
            "theme/vscode.json",
            "theme/alacritty.toml",
            "theme/hyprland.conf",
            "theme/waybar.css",
        ] {
            assert!(
                !embedded.iter().any(|p| p == legacy),
                "legacy default '{legacy}' should not be embedded"
            );
        }
    }

    #[test]
    fn default_theme_manifest_deserializes_after_placeholder_substitution() {
        let content = read_default_str("theme/omarchist.json").expect("manifest is embedded");
        let substituted = content
            .replace("{{THEME_NAME}}", "test-theme")
            .replace("{{CREATED_AT}}", "2026-01-01T00:00:00Z")
            .replace("{{MODIFIED_AT}}", "2026-01-01T00:00:00Z")
            .replace("{{AUTHOR}}", "");

        let theme: EditingTheme =
            serde_json::from_str(&substituted).expect("default manifest should parse");
        assert_eq!(theme.name, "test-theme");
        assert!(
            theme.apps.neovim.is_none(),
            "new themes ship no neovim override"
        );
        assert!(
            theme.apps.vscode.is_none(),
            "new themes ship no vscode override"
        );
        assert!(theme.apps.btop.is_none(), "btop override is opt-in");
        assert!(theme.apps.chromium.is_none(), "chromium override is opt-in");
        assert!(theme.apps.lock.is_none(), "lock override is opt-in");
        assert_eq!(theme.colors.mode, "dark");
    }
}
