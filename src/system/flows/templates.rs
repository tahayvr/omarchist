//! Starter flows: flow files without an id, offered when creating a flow.
//! The built-in ones are embedded from `defaults/flows/` and only use
//! commands every Omarchy install has.
use crate::assets::DefaultAssets;

use super::{Flow, parse_flow};

const BUILT_IN_DIR: &str = "flows/";
const SUFFIX: &str = ".flow.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSource {
    /// Shipped inside the binary.
    BuiltIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    /// The file stem without its ordering prefix, e.g. `morning-start`.
    pub key: String,
    pub flow: Flow,
    pub source: TemplateSource,
}

/// Every template, built-in ones in their file order. A file that does not
/// parse is reported and skipped; the test below keeps that from shipping.
pub fn templates() -> Vec<Template> {
    let mut paths: Vec<String> = DefaultAssets::iter()
        .map(|p| p.to_string())
        .filter(|p| p.starts_with(BUILT_IN_DIR) && p.ends_with(SUFFIX))
        .collect();
    paths.sort();
    paths
        .iter()
        .filter_map(|path| match built_in(path) {
            Ok(template) => Some(template),
            Err(e) => {
                eprintln!("Skipping template {path}: {e}");
                None
            }
        })
        .collect()
}

pub fn template(key: &str) -> Option<Template> {
    templates().into_iter().find(|t| t.key == key)
}

fn built_in(path: &str) -> crate::error::Result<Template> {
    let content = crate::assets::read_default_str(path)?;
    let flow = parse_flow(&content)?;
    Ok(Template {
        key: key_of(path),
        flow,
        source: TemplateSource::BuiltIn,
    })
}

/// `flows/01-morning-start.flow.toml` → `morning-start`.
fn key_of(path: &str) -> String {
    let stem = path
        .trim_start_matches(BUILT_IN_DIR)
        .trim_end_matches(SUFFIX);
    let unprefixed = stem.trim_start_matches(|c: char| c.is_ascii_digit());
    unprefixed
        .strip_prefix('-')
        .unwrap_or(unprefixed)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_built_in_template_parses_and_validates() {
        let templates = templates();
        assert!(!templates.is_empty(), "no built-in templates were embedded");
        for template in &templates {
            assert!(
                template.flow.id.is_empty(),
                "template {} carries an id; the editor assigns one on save",
                template.key
            );
            assert!(
                super::super::is_slug(&template.key),
                "template file name {} is not a slug",
                template.key
            );
            template
                .flow
                .validate_content()
                .unwrap_or_else(|e| panic!("template {} is invalid: {e}", template.key));
        }
    }

    #[test]
    fn template_keys_are_unique() {
        let mut keys: Vec<_> = templates().into_iter().map(|t| t.key).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), templates().len());
    }

    #[test]
    fn keys_drop_the_ordering_prefix() {
        assert_eq!(key_of("flows/01-morning-start.flow.toml"), "morning-start");
        assert_eq!(key_of("flows/wrap-up.flow.toml"), "wrap-up");
    }
}
