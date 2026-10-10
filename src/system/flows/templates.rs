//! Starter flows: flow files without an id, offered when creating a flow.
//! The built-in ones are embedded from `defaults/flows/` and only use
//! commands every Omarchy install has; the user's own live in
//! `~/.config/omarchist/templates/`.
use std::fs;
use std::path::PathBuf;

use crate::assets::DefaultAssets;
use crate::error::{Error, Result};

use super::share::{SHARED_SUFFIX, export_toml};
use super::{Flow, is_slug, parse_flow, slug};

const BUILT_IN_DIR: &str = "flows/";
/// Keys of user templates start with this, so they never collide with a
/// built-in one.
const USER_KEY_PREFIX: &str = "user:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    /// Shipped inside the binary.
    BuiltIn,
    /// A file in the user's templates directory.
    User,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    /// Built-in: the file stem without its ordering prefix, e.g.
    /// `morning-start`. User: `user:<stem>`.
    pub key: String,
    pub flow: Flow,
    pub source: TemplateSource,
}

impl Template {
    pub fn is_built_in(&self) -> bool {
        self.source == TemplateSource::BuiltIn
    }
}

/// `~/.config/omarchist/templates`
pub fn user_templates_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".config").join("omarchist").join("templates"))
        .ok_or(Error::UnknownDirectory("home"))
}

/// Every template: the built-in ones in their file order, then the user's
/// by file name. A file that does not parse is reported and skipped; the
/// test below keeps a broken built-in from shipping.
pub fn templates() -> Vec<Template> {
    let mut templates = built_in_templates();
    templates.extend(user_templates());
    templates
}

/// The template with this key, reading only what the key names: the
/// embedded files for a built-in, one file for a user template.
pub fn template(key: &str) -> Option<Template> {
    match key.strip_prefix(USER_KEY_PREFIX) {
        Some(stem) => {
            if !is_slug(stem) {
                return None;
            }
            let path = user_templates_dir()
                .ok()?
                .join(format!("{stem}{SHARED_SUFFIX}"));
            user_template(&path).ok()
        }
        None => built_in_templates().into_iter().find(|t| t.key == key),
    }
}

/// Saves `flow` as one of the user's templates, in a file named after
/// it, and says whether a template of that name was there already: saving
/// under the same name again is how a template is updated. Like a shared
/// file, a template carries no id and no triggers.
pub fn save_user_template(flow: &Flow) -> Result<bool> {
    save_template_in(&user_templates_dir()?, flow)
}

fn save_template_in(dir: &std::path::Path, flow: &Flow) -> Result<bool> {
    flow.validate_content()?;
    let stem = slug(&flow.name);
    if stem.is_empty() {
        return Err(Error::Invalid(
            "A template's name needs a letter or a digit".to_string(),
        ));
    }
    fs::create_dir_all(dir).map_err(|e| Error::io("Failed to create the templates folder", e))?;
    let path = dir.join(format!("{stem}{SHARED_SUFFIX}"));
    let replaced = path.exists();
    crate::system::fs::write_atomic(&path, export_toml(flow)?, "the template")?;
    Ok(replaced)
}

/// Deletes one of the user's templates by its key. A built-in one cannot
/// be deleted: it is part of the program.
pub fn delete_user_template(key: &str) -> Result<()> {
    delete_template_in(&user_templates_dir()?, key)
}

fn delete_template_in(dir: &std::path::Path, key: &str) -> Result<()> {
    let stem = key
        .strip_prefix(USER_KEY_PREFIX)
        .filter(|stem| is_slug(stem))
        .ok_or_else(|| Error::Invalid("Only your own templates can be deleted".to_string()))?;
    let path = dir.join(format!("{stem}{SHARED_SUFFIX}"));
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io("Failed to delete the template", e)),
    }
}

fn built_in_templates() -> Vec<Template> {
    let mut paths: Vec<String> = DefaultAssets::iter()
        .map(|p| p.to_string())
        .filter(|p| p.starts_with(BUILT_IN_DIR) && p.ends_with(SHARED_SUFFIX))
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

fn built_in(path: &str) -> Result<Template> {
    let content = crate::assets::read_default_str(path)?;
    let mut flow = parse_flow(&content)?;
    // Triggers belong to the machine, never to a template: nobody expects
    // "start from a template" to install a startup hook.
    flow.triggers = Default::default();
    Ok(Template {
        key: built_in_key(path),
        flow,
        source: TemplateSource::BuiltIn,
    })
}

/// `flows/01-morning-start.flow.toml` → `morning-start`.
fn built_in_key(path: &str) -> String {
    let stem = path
        .trim_start_matches(BUILT_IN_DIR)
        .trim_end_matches(SHARED_SUFFIX);
    let unprefixed = stem.trim_start_matches(|c: char| c.is_ascii_digit());
    unprefixed
        .strip_prefix('-')
        .unwrap_or(unprefixed)
        .to_string()
}

fn user_templates() -> Vec<Template> {
    let Ok(dir) = user_templates_dir() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().ends_with(SHARED_SUFFIX))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| match user_template(&path) {
            Ok(template) => Some(template),
            Err(e) => {
                eprintln!("Skipping template {}: {e}", path.display());
                None
            }
        })
        .collect()
}

fn user_template(path: &PathBuf) -> Result<Template> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stem = name.trim_end_matches(SHARED_SUFFIX);
    if !is_slug(stem) {
        return Err(Error::Invalid(format!(
            "'{stem}' is not a valid template name; use lowercase letters, digits, and hyphens"
        )));
    }
    let content = fs::read_to_string(path).map_err(|e| Error::io("Failed to read template", e))?;
    let mut flow = parse_flow(&content)?;
    flow.id.clear();
    flow.triggers = Default::default();
    flow.validate_content()?;
    Ok(Template {
        key: format!("{USER_KEY_PREFIX}{stem}"),
        flow,
        source: TemplateSource::User,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_built_in_template_parses_and_validates() {
        let templates = built_in_templates();
        assert!(!templates.is_empty(), "no built-in templates were embedded");
        for template in &templates {
            assert!(
                template.flow.id.is_empty(),
                "template {} carries an id; the editor assigns one on save",
                template.key
            );
            assert_eq!(
                template.flow.triggers,
                Default::default(),
                "template {} carries triggers; those belong to the machine",
                template.key
            );
            assert!(
                is_slug(&template.key),
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
    fn a_flow_saved_as_a_template_loses_its_id_and_triggers() {
        // A folder of its own: the tests run in parallel in one process.
        let dir =
            std::env::temp_dir().join(format!("omarchist-templates-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut flow = Flow::new("morning".into(), "Morning Start!".into());
        flow.steps = vec![super::super::Step::new(super::super::StepKind::Wait {
            ms: 5,
        })];
        flow.triggers.launcher = true;

        assert!(!save_template_in(&dir, &flow).unwrap());
        let path = dir.join("morning-start.flow.toml");
        let template = user_template(&path).unwrap();
        assert_eq!(template.key, "user:morning-start");
        assert!(template.flow.id.is_empty());
        assert_eq!(template.flow.triggers, Default::default());
        assert_eq!(template.flow.steps, flow.steps);
        // Saving under the same name again replaces it.
        assert!(save_template_in(&dir, &flow).unwrap());

        // Only a key of the user's own names a file to delete.
        assert!(delete_template_in(&dir, "morning-start").is_err());
        assert!(delete_template_in(&dir, "user:../morning-start").is_err());
        assert!(path.exists());
        delete_template_in(&dir, "user:morning-start").unwrap();
        assert!(!path.exists());
        delete_template_in(&dir, "user:morning-start").unwrap();

        flow.name = "***".into();
        assert!(save_template_in(&dir, &flow).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn template_keys_are_unique() {
        let mut keys: Vec<_> = templates().into_iter().map(|t| t.key).collect();
        let total = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), total);
    }

    #[test]
    fn keys_drop_the_ordering_prefix() {
        assert_eq!(
            built_in_key("flows/01-morning-start.flow.toml"),
            "morning-start"
        );
        assert_eq!(built_in_key("flows/wrap-up.flow.toml"), "wrap-up");
    }

    #[test]
    fn a_user_template_file_is_read_without_an_id() {
        let dir =
            std::env::temp_dir().join(format!("omarchist-templates-read-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("night-shift.flow.toml");
        fs::write(&path, "name = \"Night shift\"\nid = \"whatever\"\n").unwrap();
        let template = user_template(&path).unwrap();
        assert_eq!(template.key, "user:night-shift");
        assert!(template.flow.id.is_empty());
        assert!(!template.is_built_in());

        let bad = dir.join("Night Shift.flow.toml");
        fs::write(&bad, "name = \"x\"\n").unwrap();
        assert!(user_template(&bad).is_err());
    }
}
