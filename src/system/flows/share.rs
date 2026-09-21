//! Moving flows between machines: an export strips what is local to this
//! one, and an import reads a file or URL into a flow that has no id yet.
//! Importing never writes or runs anything; the caller shows the flow and
//! saves it when the user agrees.
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use isahc::{RequestExt, config::Configurable};

use crate::error::{Error, Result};

use super::{Flow, Triggers, parse_flow};

/// The suffix shared files use, so they can be told apart from the store's
/// `<id>.toml` and associated with Omarchist later.
pub const SHARED_SUFFIX: &str = ".flow.toml";
/// A flow file is text; anything bigger than this is not one.
const MAX_BYTES: u64 = 64 * 1024;
const FETCH_TIMEOUT: Duration = Duration::from_secs(15);

/// The file name an export suggests, `<id>.flow.toml`.
pub fn export_file_name(flow: &Flow) -> String {
    format!("{}{SHARED_SUFFIX}", flow.id)
}

/// The flow as a shared file: no id and no triggers, since both belong to
/// this machine; everything else, metadata included, as saved.
pub fn export_toml(flow: &Flow) -> Result<String> {
    let mut shared = flow.clone();
    shared.id.clear();
    shared.triggers = Triggers::default();
    shared.to_toml()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportSource {
    File(PathBuf),
    Url(String),
}

impl ImportSource {
    /// `https://…` is fetched; anything else is a path. Plain `http://` is
    /// refused because a flow is a list of commands to run.
    pub fn parse(arg: &str) -> Result<Self> {
        let arg = arg.trim();
        if arg.starts_with("https://") {
            Ok(Self::Url(arg.to_string()))
        } else if arg.starts_with("http://") {
            Err(Error::Invalid(
                "Only https:// URLs can be imported".to_string(),
            ))
        } else {
            Ok(Self::File(PathBuf::from(arg)))
        }
    }

    /// What to tell the user the flow came from.
    pub fn label(&self) -> String {
        match self {
            Self::File(path) => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
            Self::Url(url) => url.clone(),
        }
    }
}

/// A flow read from outside the store, not yet saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// Has no id; `meta.source` is set when it came from a URL.
    pub flow: Flow,
    pub origin: String,
}

pub fn read_import(source: &ImportSource) -> Result<Imported> {
    let content = match source {
        ImportSource::File(path) => read_file(path)?,
        ImportSource::Url(url) => fetch(url)?,
    };
    let mut flow = parse_flow(&content)?;
    flow.id.clear();
    if let ImportSource::Url(url) = source {
        flow.meta.source = url.clone();
    }
    flow.validate_content()?;
    Ok(Imported {
        flow,
        origin: source.label(),
    })
}

fn read_file(path: &Path) -> Result<String> {
    let size = fs::metadata(path)
        .map_err(|e| Error::io("Failed to read the flow file", e))?
        .len();
    if size > MAX_BYTES {
        return Err(Error::Invalid(format!(
            "{} is too large to be a flow file",
            path.display()
        )));
    }
    fs::read_to_string(path).map_err(|e| Error::io("Failed to read the flow file", e))
}

fn fetch(url: &str) -> Result<String> {
    let mut response = isahc::Request::get(url)
        .timeout(FETCH_TIMEOUT)
        .header("User-Agent", "omarchist")
        .body(())
        .map_err(|e| Error::Network(format!("Invalid URL: {e}")))?
        .send()
        .map_err(|e| Error::Network(format!("Failed to fetch {url}: {e}")))?;
    if !response.status().is_success() {
        return Err(Error::Network(format!(
            "{url} answered with status {}",
            response.status()
        )));
    }
    let mut content = String::new();
    response
        .body_mut()
        .take(MAX_BYTES + 1)
        .read_to_string(&mut content)
        .map_err(|e| Error::Network(format!("Failed to read {url}: {e}")))?;
    if content.len() as u64 > MAX_BYTES {
        return Err(Error::Invalid(format!(
            "{url} is too large to be a flow file"
        )));
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::super::{Meta, Step, StepKind};
    use super::*;

    fn flow() -> Flow {
        let mut flow = Flow::new("demo".into(), "Demo".into());
        flow.triggers.startup = true;
        flow.meta = Meta {
            author: "Taha".into(),
            ..Meta::default()
        };
        flow.steps.push(Step::new(StepKind::Wait { ms: 5 }));
        flow
    }

    #[test]
    fn export_drops_the_id_and_triggers_and_keeps_the_rest() {
        let text = export_toml(&flow()).unwrap();
        assert!(!text.contains("id ="), "{text}");
        assert!(!text.contains("[triggers]"), "{text}");
        assert!(text.contains("author = \"Taha\""), "{text}");
        assert!(text.contains("[[step]]"), "{text}");
        assert_eq!(export_file_name(&flow()), "demo.flow.toml");
    }

    #[test]
    fn sources_are_told_apart_by_scheme() {
        assert_eq!(
            ImportSource::parse("https://example.com/a.flow.toml").unwrap(),
            ImportSource::Url("https://example.com/a.flow.toml".into())
        );
        assert_eq!(
            ImportSource::parse("./a.flow.toml").unwrap(),
            ImportSource::File(PathBuf::from("./a.flow.toml"))
        );
        assert!(ImportSource::parse("http://example.com/a.flow.toml").is_err());
    }

    #[test]
    fn a_file_imports_without_an_id_and_names_its_origin() {
        let dir = std::env::temp_dir().join(format!("omarchist-share-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.flow.toml");
        fs::write(&path, export_toml(&flow()).unwrap()).unwrap();

        let imported = read_import(&ImportSource::File(path)).unwrap();
        assert!(imported.flow.id.is_empty());
        assert_eq!(imported.flow.name, "Demo");
        assert_eq!(imported.flow.meta.author, "Taha");
        assert!(imported.flow.meta.source.is_empty());
        assert_eq!(imported.origin, "demo.flow.toml");
    }

    #[test]
    fn an_invalid_file_is_refused_on_import() {
        let dir = std::env::temp_dir().join(format!("omarchist-share-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.flow.toml");
        fs::write(&path, "name = \"\"\n").unwrap();
        assert!(read_import(&ImportSource::File(path)).is_err());
    }
}
