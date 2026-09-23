//! What the Theme Designer lists on its Optional tabs: one entry per file,
//! except that the files of a palette bundle are one entry, and hidden files
//! are not listed.

use std::collections::BTreeMap;

use super::files::{OverrideStatus, status};
use super::palette::{PaletteBundle, bundle_of};
use super::registry::{Category, EditorKind, OVERRIDES, OverrideSpec};

#[derive(Debug, Clone, Copy)]
pub enum Entry {
    File(&'static OverrideSpec),
    Bundle(&'static PaletteBundle),
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Entry {
    /// The file name, or the bundle id.
    pub fn id(&self) -> &'static str {
        match self {
            Entry::File(spec) => spec.file,
            Entry::Bundle(bundle) => bundle.id,
        }
    }

    pub fn app(&self) -> &'static str {
        match self {
            Entry::File(spec) => spec.app,
            Entry::Bundle(bundle) => bundle.app,
        }
    }

    pub fn category(&self) -> Category {
        match self {
            Entry::File(spec) => spec.category,
            Entry::Bundle(bundle) => bundle.category,
        }
    }

    pub fn binaries(&self) -> &'static [&'static str] {
        match self {
            Entry::File(spec) => spec.binaries,
            Entry::Bundle(bundle) => bundle.binaries,
        }
    }

    /// Whether the theme customizes it; `palettes` is the theme's
    /// `EditingTheme::palettes`.
    pub fn is_custom(
        &self,
        theme: &str,
        palettes: &BTreeMap<String, BTreeMap<String, String>>,
    ) -> bool {
        match self {
            Entry::File(spec) => status(theme, spec) == OverrideStatus::Custom,
            Entry::Bundle(bundle) => palettes.contains_key(bundle.id),
        }
    }
}

/// The entries of a category, in registry order.
pub fn in_category(category: Category) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    for spec in OVERRIDES.iter().filter(|spec| spec.category == category) {
        let entry = match spec.editor {
            EditorKind::Hidden => continue,
            EditorKind::Palette => match bundle_of(spec.file) {
                Some(bundle) => Entry::Bundle(bundle),
                None => continue,
            },
            _ => Entry::File(spec),
        };
        if !entries.contains(&entry) {
            entries.push(entry);
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::{Entry, in_category};
    use crate::system::themes::overrides::registry::Category;

    #[test]
    fn terminals_are_one_bundle() {
        let terminals = in_category(Category::Terminals);
        assert_eq!(terminals.len(), 1);
        assert!(matches!(terminals[0], Entry::Bundle(bundle) if bundle.id == "terminals"));
    }

    #[test]
    fn hidden_files_are_not_listed() {
        let desktop: Vec<&str> = in_category(Category::Desktop)
            .iter()
            .map(Entry::id)
            .collect();
        assert!(!desktop.contains(&"hyprland.lua"));
        assert!(!desktop.contains(&"shell.hyprland.toml"));
        assert!(desktop.contains(&"terminal-menus"));
        assert!(desktop.contains(&"shell.bar.toml"));
    }
}
