use gpui::*;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "**/*"]
pub struct OmarchistAssets;

impl AssetSource for OmarchistAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Ok(Self::get(path).map(|f| Some(f.data)).unwrap_or(None))
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
