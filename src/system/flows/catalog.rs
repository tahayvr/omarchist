//! The catalog: flows people shared, reviewed in a public repository and
//! listed in a signed index. This module reads that index for the app
//! (`load`, `fetch_flow`), builds it for the repository's CI (`build`,
//! `sign`), and checks a flow file the same way on both sides
//! (`check_file`). Nothing from the catalog is saved or run by this code:
//! a flow comes back as an [`Imported`] for the editor's review screen.
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use isahc::config::{Configurable, RedirectPolicy};
use isahc::{ReadResponseExt, RequestExt};
use ring::signature::{ED25519, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::system::fs::write_atomic;

use super::risks::{self, Risk};
use super::share::{Imported, SHARED_SUFFIX};
use super::{Flow, ICONS, Step, StepKind, Triggers, is_slug, parse_flow, requirements, slug};

/// Where the app reads the catalog from. `OMARCHIST_CATALOG_URL` names
/// another one (a `file://` folder works too), for a catalog of your own
/// or a test.
pub const DEFAULT_URL: &str = "https://flows.omarchist.com";
/// The repository flows are published to and reviewed in.
pub const REPO: &str = "tahayvr/omarchist-flows";
/// The one license every flow in the catalog carries.
pub const LICENSE: &str = "CC0-1.0";
/// What `meta.source` of a flow installed from the catalog starts with.
pub const SOURCE_PREFIX: &str = "catalog:";

pub const CATEGORIES: &[&str] = &[
    "Productivity",
    "Focus",
    "Media",
    "Capture",
    "Text",
    "Web",
    "Files",
    "Windows",
    "System",
];

/// The Ed25519 keys an index may be signed with, base64. More than one
/// only while the key is being changed: a release ships the new key
/// before the catalog starts signing with it.
const PUBLIC_KEYS: &[&str] = &["T/g9V9DY1EaynU9Nxja67Ve5g6+esdBeCt/FBBMMuOk="];

const INDEX_PATH: &str = "v1/index.json";
const INSTALLS_PATH: &str = "v1/installs.json";
const MAX_INDEX_BYTES: u64 = 4 * 1024 * 1024;
const MAX_FLOW_BYTES: u64 = 64 * 1024;
const TIMEOUT: Duration = Duration::from_secs(15);

const MAX_STEPS: usize = 100;
const MAX_TAGS: usize = 5;

/// The list of everything in the catalog, as the catalog's CI writes it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Index {
    pub version: u32,
    /// Grows with every build; an index older than one already seen is
    /// refused, so a yanked flow cannot be brought back by replaying one.
    pub serial: u64,
    pub generated: String,
    #[serde(default)]
    pub featured: Vec<String>,
    /// Authors with a history of reviewed flows.
    #[serde(default)]
    pub verified: Vec<String>,
    pub flows: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Entry {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub author: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub version: u32,
    /// Of the flow file's bytes, in hex.
    pub sha256: String,
    pub size: u64,
    pub format: u32,
    pub steps: usize,
    /// The `type_tag` of each top-level step, for the card's strip.
    #[serde(default)]
    pub kinds: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    /// What the checks flagged, the dangerous ones first.
    #[serde(default)]
    pub risks: Vec<String>,
    pub added: String,
    pub updated: String,
    /// Why the flow was pulled, when it was. A pulled flow is not offered
    /// and people who installed it are told.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yanked: Option<String>,
}

impl Index {
    pub fn entry(&self, slug: &str) -> Option<&Entry> {
        self.flows.iter().find(|entry| entry.slug == slug)
    }

    pub fn is_verified(&self, author: &str) -> bool {
        self.verified
            .iter()
            .any(|name| name.eq_ignore_ascii_case(author))
    }

    pub fn is_featured(&self, slug: &str) -> bool {
        self.featured.iter().any(|s| s == slug)
    }
}

// MARK: Checking a flow file

/// What `omarchist flow check` says about one file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Report {
    pub file: String,
    pub ok: bool,
    pub errors: Vec<String>,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: Option<u32>,
    pub category: String,
    pub tags: Vec<String>,
    pub format: u32,
    pub steps: usize,
    /// One line per step, numbered and indented as the editor shows them.
    pub summary: Vec<String>,
    pub requires: Vec<String>,
    pub risks: Vec<Risk>,
}

/// The flow's steps as numbered lines, indented by how deep they sit.
pub fn step_lines(flow: &Flow) -> Vec<String> {
    flow.walk()
        .into_iter()
        .enumerate()
        .map(|(index, (path, step))| {
            format!(
                "{}{}. {}{}",
                "  ".repeat(path.len() / 2),
                index + 1,
                step.kind.text(),
                if step.enabled { "" } else { " (off)" }
            )
        })
        .collect()
}

/// Every program the flow's steps start, and what `meta.requires` adds,
/// each once, in order. Paths are left out: they name one machine.
pub fn required_programs(flow: &Flow) -> Vec<String> {
    let mut programs: Vec<String> = Vec::new();
    let from_steps = flow
        .walk()
        .into_iter()
        .flat_map(|(_, step)| requirements::programs_of(&step.kind));
    for program in from_steps.chain(flow.meta.requires.iter().cloned()) {
        if !program.contains('/') && !programs.contains(&program) {
            programs.push(program);
        }
    }
    programs
}

/// `1`, `2`, `3`: the catalog's versions are whole numbers that only grow.
pub fn version_of(flow: &Flow) -> Option<u32> {
    flow.meta
        .version
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|v| *v > 0)
}

fn is_github_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 39
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// What keeps a flow out of the catalog, beyond being a valid flow.
pub fn catalog_errors(flow: &Flow) -> Vec<String> {
    let mut errors = Vec::new();
    let mut need = |ok: bool, message: &str| {
        if !ok {
            errors.push(message.to_string());
        }
    };
    let name = flow.name.trim().chars().count();
    need(
        (3..=48).contains(&name),
        "The name needs 3 to 48 characters",
    );
    let description = flow.description.trim().chars().count();
    need(
        (10..=160).contains(&description),
        "The description needs 10 to 160 characters",
    );
    need(
        !slug(&flow.name).is_empty(),
        "The name needs a letter or a digit",
    );
    need(
        is_github_name(&flow.meta.author),
        "meta.author must be your GitHub user name",
    );
    need(
        version_of(flow).is_some(),
        "meta.version must be a whole number, starting at 1",
    );
    need(
        CATEGORIES.contains(&flow.meta.category.as_str()),
        &format!("meta.category must be one of: {}", CATEGORIES.join(", ")),
    );
    need(
        flow.meta.license == LICENSE,
        &format!("meta.license must be \"{LICENSE}\""),
    );
    need(
        flow.meta.tags.len() <= MAX_TAGS
            && flow
                .meta
                .tags
                .iter()
                .all(|tag| is_slug(tag) && tag.len() <= 20),
        "At most 5 tags, each lowercase letters, digits and hyphens",
    );
    need(
        ICONS.contains(&flow.icon.as_str()),
        "The icon must be one the editor offers",
    );
    need(
        flow.id.is_empty(),
        "A shared flow has no id; export it first",
    );
    need(
        flow.triggers.is_empty(),
        "A shared flow has no triggers; export it first",
    );
    need(
        flow.meta.source.is_empty(),
        "meta.source is set by the app, not by a shared flow",
    );
    let steps = flow.walk();
    need(
        steps.iter().any(|(_, step)| step.enabled),
        "The flow has no step that is switched on",
    );
    need(
        steps.len() <= MAX_STEPS,
        &format!("At most {MAX_STEPS} steps"),
    );
    need(
        !steps
            .iter()
            .any(|(_, step)| matches!(step.kind, StepKind::Flow { .. })),
        "A shared flow cannot run another flow: that flow is not shared with it",
    );
    errors
}

/// Checks a flow's text: that it parses and validates, and, with
/// `for_catalog`, that it meets the catalog's rules. `file` only names it.
pub fn check_text(file: &str, content: &str, for_catalog: bool) -> Report {
    let mut report = Report {
        file: file.to_string(),
        ..Report::default()
    };
    if content.len() as u64 > MAX_FLOW_BYTES {
        report.errors.push(format!(
            "Larger than {} KB, which no flow should be",
            MAX_FLOW_BYTES / 1024
        ));
        return report;
    }
    let flow = match parse_flow(content) {
        Ok(flow) => flow,
        Err(e) => {
            report.errors.push(e.to_string());
            return report;
        }
    };
    if let Err(e) = flow.validate_content() {
        report.errors.push(e.to_string());
    }
    if for_catalog {
        report.errors.extend(catalog_errors(&flow));
    }
    report.name = flow.name.clone();
    report.description = flow.description.clone();
    report.author = flow.meta.author.clone();
    report.version = version_of(&flow);
    report.category = flow.meta.category.clone();
    report.tags = flow.meta.tags.clone();
    report.format = flow.required_format();
    report.steps = flow.walk().len();
    report.summary = step_lines(&flow);
    report.requires = required_programs(&flow);
    report.risks = risks::risks(&flow);
    report.ok = report.errors.is_empty();
    report
}

/// The slug a catalog file is published under: its name without the
/// `.flow.toml`.
fn file_slug(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    let stem = name.strip_suffix(SHARED_SUFFIX)?;
    (is_slug(stem) && stem.len() <= 48).then(|| stem.to_string())
}

pub fn check_file(path: &Path, for_catalog: bool) -> Report {
    let file = path.display().to_string();
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            return Report {
                file,
                errors: vec![format!("Could not read the file: {e}")],
                ..Report::default()
            };
        }
    };
    let mut report = check_text(&file, &content, for_catalog);
    if for_catalog && file_slug(path).is_none() {
        report.errors.push(format!(
            "The file must be named <slug>{SHARED_SUFFIX}: lowercase letters, digits and hyphens"
        ));
        report.ok = false;
    }
    report
}

// MARK: Building the index (the catalog's CI)

fn lines_of(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

#[derive(Deserialize)]
struct Yank {
    #[serde(default)]
    reason: String,
}

fn entry_of(slug: &str, flow: &Flow, bytes: &[u8]) -> Entry {
    Entry {
        slug: slug.to_string(),
        name: flow.name.trim().to_string(),
        description: flow.description.trim().to_string(),
        icon: flow.icon.clone(),
        author: flow.meta.author.clone(),
        category: flow.meta.category.clone(),
        tags: flow.meta.tags.clone(),
        version: version_of(flow).unwrap_or(1),
        sha256: hex::encode(Sha256::digest(bytes)),
        size: bytes.len() as u64,
        format: flow.required_format(),
        steps: flow.walk().len(),
        kinds: flow.steps.iter().map(|step| step.kind.type_tag()).collect(),
        requires: required_programs(flow),
        risks: risks::summary(flow)
            .into_iter()
            .map(|(_, what)| what.to_string())
            .collect(),
        added: String::new(),
        updated: String::new(),
        yanked: None,
    }
}

/// Builds the catalog from a checkout of its repository into `out`:
/// `v1/index.json` and one `v1/flows/<slug>/<version>.flow.toml` per flow.
/// `previous` is the index being replaced; it carries the dates forward
/// and lets the build refuse a flow that changed without a new version,
/// went back a version, or changed hands. Every problem is reported, not
/// only the first. `today` is `YYYY-MM-DD` and `now` the build's time in
/// seconds since the Unix epoch.
pub fn build(
    repo: &Path,
    out: &Path,
    previous: Option<&Index>,
    today: &str,
    now: u64,
) -> Result<Index> {
    let dir = repo.join("flows");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| Error::io(format!("Failed to read {}", dir.display()), e))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.to_string_lossy().ends_with(SHARED_SUFFIX))
        .collect();
    files.sort();

    let yanked: HashMap<String, Yank> = fs::read_to_string(repo.join("yanked.toml"))
        .ok()
        .map(|text| {
            toml::from_str(&text)
                .map_err(|e| Error::Invalid(format!("Failed to parse yanked.toml: {e}")))
        })
        .transpose()?
        .unwrap_or_default();

    let mut problems: Vec<String> = Vec::new();
    let mut entries: Vec<Entry> = Vec::new();
    for path in &files {
        let report = check_file(path, true);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !report.ok {
            problems.extend(report.errors.iter().map(|e| format!("{name}: {e}")));
            continue;
        }
        let (Some(slug), Ok(bytes)) = (file_slug(path), fs::read(path)) else {
            problems.push(format!("{name}: could not be read"));
            continue;
        };
        // Checked above, so it parses.
        let Ok(flow) = parse_flow(&String::from_utf8_lossy(&bytes)) else {
            continue;
        };
        let mut entry = entry_of(&slug, &flow, &bytes);
        let before = previous.and_then(|index| index.entry(&slug));
        match before {
            Some(old) if !old.author.eq_ignore_ascii_case(&entry.author) => {
                problems.push(format!(
                    "{name}: '{slug}' belongs to {}; pick another name for yours",
                    old.author
                ));
                continue;
            }
            Some(old) if old.version > entry.version => {
                problems.push(format!(
                    "{name}: version {} is older than the published {}",
                    entry.version, old.version
                ));
                continue;
            }
            Some(old) if old.version == entry.version && old.sha256 != entry.sha256 => {
                problems.push(format!(
                    "{name}: changed without a new version; set meta.version to \"{}\"",
                    old.version + 1
                ));
                continue;
            }
            _ => {}
        }
        (entry.added, entry.updated) = match before {
            Some(old) if old.version == entry.version => (old.added.clone(), old.updated.clone()),
            Some(old) => (old.added.clone(), today.to_string()),
            None => (today.to_string(), today.to_string()),
        };
        entry.yanked = yanked.get(&slug).map(|yank| yank.reason.trim().to_string());

        let target = out
            .join("v1/flows")
            .join(&slug)
            .join(format!("{}{SHARED_SUFFIX}", entry.version));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::io("Failed to create the output folder", e))?;
        }
        fs::write(&target, &bytes).map_err(|e| Error::io("Failed to write a flow file", e))?;
        entries.push(entry);
    }
    if !problems.is_empty() {
        return Err(Error::Invalid(problems.join("\n")));
    }

    let known = |slug: &String| entries.iter().any(|entry| &entry.slug == slug);
    let index = Index {
        version: 1,
        serial: now.max(previous.map_or(0, |index| index.serial + 1)),
        generated: today.to_string(),
        featured: lines_of(&repo.join("featured.txt"))
            .into_iter()
            .filter(known)
            .collect(),
        verified: lines_of(&repo.join("verified.txt")),
        flows: entries,
    };
    Ok(index)
}

/// The index as it is published: its text and the signature of exactly
/// that text, in one file, so the two can never be out of step while a
/// new one is being uploaded.
#[derive(Debug, Serialize, Deserialize)]
struct SignedIndex {
    signed: String,
    #[serde(default)]
    signature: String,
}

/// Writes `v1/index.json` under `out`, signed with `private_pem` when
/// there is a key (a pull request's check has none and needs none).
pub fn write_index(out: &Path, index: &Index, private_pem: Option<&str>) -> Result<()> {
    let signed = serde_json::to_string_pretty(index)
        .map_err(|e| Error::json("Failed to write the index", e))?;
    let signature = match private_pem {
        Some(pem) => sign(signed.as_bytes(), pem)?,
        None => String::new(),
    };
    let text = serde_json::to_string(&SignedIndex { signed, signature })
        .map_err(|e| Error::json("Failed to write the index", e))?;
    let target = out.join(INDEX_PATH);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::io("Failed to create the output folder", e))?;
    }
    fs::write(&target, text).map_err(|e| Error::io("Failed to write the index", e))
}

/// The index in a published file, without checking who signed it: what
/// the catalog's CI reads back as the index it is about to replace.
pub fn read_published(bytes: &[u8]) -> Result<Index> {
    let published: SignedIndex = serde_json::from_slice(bytes)
        .map_err(|e| Error::json("Failed to read the published index", e))?;
    serde_json::from_str(&published.signed)
        .map_err(|e| Error::json("Failed to read the published index", e))
}

// MARK: Signing

fn pem_body(pem: &str) -> Result<Vec<u8>> {
    let body: String = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .flat_map(|line| line.chars())
        .filter(|c| !c.is_whitespace())
        .collect();
    BASE64
        .decode(body)
        .map_err(|_| Error::Invalid("The signing key is not a PEM file".to_string()))
}

fn key_pair(private_pem: &str) -> Result<Ed25519KeyPair> {
    Ed25519KeyPair::from_pkcs8_maybe_unchecked(&pem_body(private_pem)?)
        .map_err(|_| Error::Invalid("The signing key is not an Ed25519 private key".to_string()))
}

/// A new signing key: the private key as PEM, for the catalog's CI secret,
/// and the public key as base64, for [`PUBLIC_KEYS`].
pub fn new_key() -> Result<(String, String)> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|_| Error::Invalid("Could not generate a key".to_string()))?;
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())
        .map_err(|_| Error::Invalid("Could not generate a key".to_string()))?;
    let pem = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        BASE64.encode(pkcs8.as_ref())
    );
    Ok((pem, BASE64.encode(pair.public_key().as_ref())))
}

/// The public key of a private key, base64.
pub fn public_key_of(private_pem: &str) -> Result<String> {
    Ok(BASE64.encode(key_pair(private_pem)?.public_key().as_ref()))
}

/// The signature of `bytes` (an index file) under the private key, base64.
pub fn sign(bytes: &[u8], private_pem: &str) -> Result<String> {
    Ok(BASE64.encode(key_pair(private_pem)?.sign(bytes).as_ref()))
}

fn verifies(bytes: &[u8], signature: &str, public_key: &str) -> bool {
    let (Ok(signature), Ok(key)) = (
        BASE64.decode(signature.trim()),
        BASE64.decode(public_key.trim()),
    ) else {
        return false;
    };
    UnparsedPublicKey::new(&ED25519, key)
        .verify(bytes, &signature)
        .is_ok()
}

/// The keys this run trusts: `OMARCHIST_CATALOG_KEY` when a catalog of
/// one's own is in use, the built-in ones otherwise.
fn trusted_keys() -> Vec<String> {
    match std::env::var("OMARCHIST_CATALOG_KEY") {
        Ok(key) if !key.trim().is_empty() => vec![key.trim().to_string()],
        _ => PUBLIC_KEYS.iter().map(|key| key.to_string()).collect(),
    }
}

/// Parses a published index only after its signature checks out against
/// one of `keys`.
fn verified_index(bytes: &[u8], keys: &[String]) -> Result<Index> {
    let unsigned = || {
        Error::Invalid(
            "The catalog's list is not signed with a key this Omarchist trusts".to_string(),
        )
    };
    let published: SignedIndex = serde_json::from_slice(bytes).map_err(|_| unsigned())?;
    if !keys
        .iter()
        .any(|key| verifies(published.signed.as_bytes(), &published.signature, key))
    {
        return Err(unsigned());
    }
    serde_json::from_str(&published.signed)
        .map_err(|e| Error::json("Failed to read the catalog's list", e))
}

// MARK: Reading the catalog (the app)

pub fn base_url() -> String {
    match std::env::var("OMARCHIST_CATALOG_URL") {
        Ok(url) if !url.trim().is_empty() => url.trim().trim_end_matches('/').to_string(),
        _ => DEFAULT_URL.to_string(),
    }
}

/// `$XDG_CACHE_HOME/omarchist/catalog` (`~/.cache/omarchist/catalog`).
pub fn cache_dir() -> Result<PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .or_else(|| dirs::home_dir().map(|home| home.join(".cache")))
        .map(|dir| dir.join("omarchist").join("catalog"))
        .ok_or(Error::UnknownDirectory("cache"))
}

/// Reads `path` under the catalog's address, refusing more than `limit`
/// bytes. A `file://` address is read from disk.
fn get(base: &str, path: &str, limit: u64) -> Result<Vec<u8>> {
    if let Some(dir) = base.strip_prefix("file://") {
        let file = Path::new(dir).join(path);
        let size = fs::metadata(&file)
            .map_err(|e| Error::io(format!("Failed to read {}", file.display()), e))?
            .len();
        if size > limit {
            return Err(Error::Invalid(format!("{path} is too large")));
        }
        return fs::read(&file).map_err(|e| Error::io("Failed to read the catalog", e));
    }
    let url = format!("{base}/{path}");
    let mut response = isahc::Request::get(&url)
        .timeout(TIMEOUT)
        .redirect_policy(RedirectPolicy::Limit(3))
        .header(
            "User-Agent",
            concat!("omarchist/", env!("CARGO_PKG_VERSION")),
        )
        .body(())
        .map_err(|e| Error::Network(format!("Invalid catalog address: {e}")))?
        .send()
        .map_err(|e| Error::Network(format!("Could not reach the catalog: {e}")))?;
    if !response.status().is_success() {
        return Err(Error::Network(format!(
            "The catalog answered with status {}",
            response.status()
        )));
    }
    let mut bytes = Vec::new();
    response
        .body_mut()
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| Error::Network(format!("Could not read from the catalog: {e}")))?;
    if bytes.len() as u64 > limit {
        return Err(Error::Invalid(format!("{path} is too large")));
    }
    Ok(bytes)
}

/// The catalog as the app shows it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Catalog {
    pub index: Index,
    /// How often each flow was installed, by slug. Unsigned and only
    /// shown, never trusted.
    pub installs: HashMap<String, u64>,
    /// Why this is the copy kept from an earlier visit, when it is.
    pub notice: Option<String>,
}

fn read_cache(dir: &Path, keys: &[String]) -> Option<Catalog> {
    let bytes = fs::read(dir.join("index.json")).ok()?;
    // Checked again: the cache is a file anyone on the machine could edit.
    let index = verified_index(&bytes, keys).ok()?;
    let installs = fs::read(dir.join("installs.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    Some(Catalog {
        index,
        installs,
        notice: None,
    })
}

/// Whether the kept copy is missing or older than a day: time to look
/// for updates to the flows installed from the catalog.
pub fn is_stale() -> bool {
    let Ok(dir) = cache_dir() else {
        return false;
    };
    fs::metadata(dir.join("index.json"))
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .is_none_or(|age| age > Duration::from_secs(24 * 60 * 60))
}

/// The copy of the catalog kept from the last visit, if there is one.
pub fn cached() -> Option<Catalog> {
    read_cache(&cache_dir().ok()?, &trusted_keys())
}

/// Fetches the catalog, checks its signature, and keeps a copy. When it
/// cannot be reached, or what came back does not check out, the copy from
/// the last visit is returned with a notice saying so.
pub fn load() -> Result<Catalog> {
    load_from(&base_url(), &cache_dir()?, &trusted_keys())
}

fn load_from(base: &str, cache: &Path, keys: &[String]) -> Result<Catalog> {
    let kept = read_cache(cache, keys);
    let fetched = get(base, INDEX_PATH, MAX_INDEX_BYTES).and_then(|bytes| {
        let index = verified_index(&bytes, keys)?;
        Ok((bytes, index))
    });
    let fall_back = |why: String| match kept.clone() {
        Some(mut catalog) => {
            catalog.notice = Some(format!(
                "{why} Showing the catalog as of {}.",
                catalog.index.generated
            ));
            Ok(catalog)
        }
        None => Err(Error::Network(why)),
    };
    let (bytes, index) = match fetched {
        Ok(fetched) => fetched,
        Err(e) => return fall_back(e.to_string()),
    };
    // An older list than the one already seen is a cache somewhere
    // catching up, or somebody replaying a list with a flow that was
    // pulled since. Either way the newer one stands, quietly.
    if let Some(kept) = &kept
        && index.serial < kept.index.serial
    {
        return Ok(kept.clone());
    }
    // Counts are a nicety: without them the catalog still opens.
    let installs_bytes = get(base, INSTALLS_PATH, MAX_INDEX_BYTES).ok();
    let installs: HashMap<String, u64> = installs_bytes
        .as_deref()
        .and_then(|bytes| serde_json::from_slice(bytes).ok())
        .or_else(|| kept.map(|kept| kept.installs))
        .unwrap_or_default();

    if fs::create_dir_all(cache).is_ok() {
        let _ = write_atomic(&cache.join("index.json"), &bytes, "the catalog cache");
        if let Ok(text) = serde_json::to_string(&installs) {
            let _ = write_atomic(&cache.join("installs.json"), text, "the catalog cache");
        }
    }
    Ok(Catalog {
        index,
        installs,
        notice: None,
    })
}

/// A flow's file from the catalog, checked against the hash its entry
/// promises, as a flow ready for the review screen: no id, no triggers,
/// and `meta.source` saying where it came from. A copy is kept, so a
/// second look needs no network.
pub fn fetch_flow(entry: &Entry) -> Result<Imported> {
    fetch_flow_from(&base_url(), &cache_dir()?, entry)
}

fn fetch_flow_from(base: &str, cache: &Path, entry: &Entry) -> Result<Imported> {
    if !is_slug(&entry.slug) {
        return Err(Error::Invalid("The catalog named a flow oddly".to_string()));
    }
    let matches = |bytes: &[u8]| hex::encode(Sha256::digest(bytes)) == entry.sha256;
    let kept = cache
        .join("flows")
        .join(format!("{}-{}{SHARED_SUFFIX}", entry.slug, entry.version));
    let bytes = match fs::read(&kept) {
        Ok(bytes) if matches(&bytes) => bytes,
        _ => {
            let path = format!("v1/flows/{}/{}{SHARED_SUFFIX}", entry.slug, entry.version);
            let bytes = get(base, &path, MAX_FLOW_BYTES)?;
            if !matches(&bytes) {
                return Err(Error::Invalid(format!(
                    "'{}' is not the file the catalog lists; it was not opened",
                    entry.name
                )));
            }
            if let Some(dir) = kept.parent()
                && fs::create_dir_all(dir).is_ok()
            {
                let _ = write_atomic(&kept, &bytes, "the catalog cache");
            }
            bytes
        }
    };
    let mut flow = parse_flow(&String::from_utf8_lossy(&bytes))?;
    flow.id.clear();
    flow.triggers = Triggers::default();
    flow.meta.source = format!("{SOURCE_PREFIX}{}@{}", entry.slug, entry.version);
    flow.validate_content()?;
    Ok(Imported {
        flow,
        origin: format!("the catalog, by {}", entry.author),
    })
}

/// Tells the catalog a flow was installed, so its count grows. Nothing
/// about the person or the machine is sent: the flow's slug and version.
/// Failing is fine and silent.
pub fn count_install(slug: &str, version: u32) {
    let base = base_url();
    if base.starts_with("file://") {
        return;
    }
    let body = serde_json::json!({ "slug": slug, "version": version }).to_string();
    let _ = isahc::Request::post(format!("{base}/v1/installs"))
        .timeout(Duration::from_secs(8))
        .header("Content-Type", "application/json")
        .header(
            "User-Agent",
            concat!("omarchist/", env!("CARGO_PKG_VERSION")),
        )
        .body(body)
        .map(|request| request.send().map(|mut response| response.text()));
}

// MARK: Installed flows

/// The catalog flow and version a saved flow was installed from.
pub fn source_of(flow: &Flow) -> Option<(String, u32)> {
    let (slug, version) = flow
        .meta
        .source
        .strip_prefix(SOURCE_PREFIX)?
        .rsplit_once('@')?;
    Some((slug.to_string(), version.parse().ok()?))
}

/// What the catalog says about a flow installed from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// Its newest version is the installed one.
    Current,
    /// A newer version is there.
    Update(u32),
    /// It was pulled, for this reason.
    Pulled(String),
    /// The catalog no longer lists it.
    Gone,
}

pub fn standing(flow: &Flow, index: &Index) -> Option<Standing> {
    let (slug, version) = source_of(flow)?;
    Some(match index.entry(&slug) {
        None => Standing::Gone,
        Some(entry) => match &entry.yanked {
            Some(reason) => Standing::Pulled(reason.clone()),
            None if entry.version > version => Standing::Update(entry.version),
            None => Standing::Current,
        },
    })
}

// MARK: What changed between two versions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Same,
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffRow {
    pub change: Change,
    /// How many blocks the step sits inside.
    pub depth: usize,
    /// The step, without the steps inside it.
    pub step: Step,
}

/// The step without the steps it holds, so a block compares by its own
/// settings and the steps inside compare by theirs.
fn without_children(kind: &StepKind) -> StepKind {
    let mut kind = kind.clone();
    match &mut kind {
        StepKind::If {
            then, otherwise, ..
        } => {
            then.clear();
            otherwise.clear();
        }
        StepKind::Repeat { steps, .. } | StepKind::Each { steps, .. } => steps.clear(),
        StepKind::Menu { choices, .. } => {
            for choice in choices {
                choice.steps.clear();
            }
        }
        _ => {}
    }
    kind
}

/// The steps of `new` against those of `old`, as written: kept, added, or
/// removed. A step that changed shows as removed and added.
pub fn diff_steps(old: &Flow, new: &Flow) -> Vec<DiffRow> {
    let flat = |flow: &Flow| -> Vec<(usize, String, Step)> {
        flow.walk()
            .into_iter()
            .map(|(path, step)| {
                let depth = path.len() / 2;
                // Everything the step's file holds for it: any setting
                // that changed makes it a different step.
                let key = format!(
                    "{depth}\u{0}{}\u{0}{:?}\u{0}{}",
                    serde_json::to_string(&without_children(&step.kind)).unwrap_or_default(),
                    step.output,
                    step.enabled
                );
                (depth, key, step.clone())
            })
            .collect()
    };
    let (a, b) = (flat(old), flat(new));
    // Longest common subsequence, from the ends.
    let mut lcs = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            lcs[i][j] = if a[i].1 == b[j].1 {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut rows = Vec::new();
    let (mut i, mut j) = (0, 0);
    let row = |change, item: &(usize, String, Step)| DiffRow {
        change,
        depth: item.0,
        step: item.2.clone(),
    };
    while i < a.len() && j < b.len() {
        if a[i].1 == b[j].1 {
            rows.push(row(Change::Same, &b[j]));
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            rows.push(row(Change::Removed, &a[i]));
            i += 1;
        } else {
            rows.push(row(Change::Added, &b[j]));
            j += 1;
        }
    }
    rows.extend(a[i..].iter().map(|item| row(Change::Removed, item)));
    rows.extend(b[j..].iter().map(|item| row(Change::Added, item)));
    rows
}

// MARK: Publishing

/// A flow made ready for the catalog, and where to hand it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submission {
    pub slug: String,
    pub file_name: String,
    /// The file's text, as it goes into the repository.
    pub toml: String,
    pub version: u32,
    /// Whether the catalog already has this flow, by the same author.
    pub update: bool,
    /// GitHub's page for proposing the file, in the browser.
    pub url: String,
}

/// A URL longer than this is cut off by browsers or by GitHub; the file's
/// text then travels on the clipboard instead.
const MAX_URL: usize = 6000;

fn encode(text: &str) -> String {
    percent_encoding::utf8_percent_encode(text, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// Strips what belongs to this machine, adds the catalog's metadata, and
/// checks the result as the catalog's CI will. `index` is the catalog as
/// last seen, to tell a new flow from a new version of one's own.
pub fn prepare_submission(
    flow: &Flow,
    author: &str,
    category: &str,
    tags: &[String],
    index: Option<&Index>,
) -> Result<Submission> {
    let mut shared = flow.clone();
    shared.id.clear();
    shared.triggers = Triggers::default();
    shared.name = shared.name.trim().to_string();
    shared.description = shared.description.trim().to_string();
    shared.meta.source.clear();
    shared.meta.author = author.trim().trim_start_matches('@').to_string();
    shared.meta.category = category.to_string();
    shared.meta.tags = tags.to_vec();
    shared.meta.license = LICENSE.to_string();
    let slug = slug(&shared.name);
    let existing = index.and_then(|index| index.entry(&slug));
    if let Some(entry) = existing
        && !entry.author.eq_ignore_ascii_case(&shared.meta.author)
    {
        return Err(Error::Invalid(format!(
            "The catalog already has '{}', by {}. Give yours another name.",
            entry.name, entry.author
        )));
    }
    let version = existing.map_or(1, |entry| entry.version + 1);
    shared.meta.version = version.to_string();
    shared.validate_content()?;
    if let Some(problem) = catalog_errors(&shared).into_iter().next() {
        return Err(Error::Invalid(problem));
    }
    let toml = shared.to_toml()?;
    let file_name = format!("{slug}{SHARED_SUFFIX}");
    let update = existing.is_some();
    let path = format!("flows/{file_name}");
    let url = if update {
        format!("https://github.com/{REPO}/edit/main/{path}")
    } else {
        // The folder goes in `filename`: GitHub drops a folder in the
        // address itself once `filename` is given.
        let message = encode(&format!("Add {}", shared.name));
        let page = format!(
            "https://github.com/{REPO}/new/main?filename={}&message={message}",
            encode(&path)
        );
        let with_text = format!("{page}&value={}", encode(&toml));
        if with_text.len() <= MAX_URL {
            with_text
        } else {
            page
        }
    };
    Ok(Submission {
        slug,
        file_name,
        toml,
        version,
        update,
        url,
    })
}

/// GitHub's page for telling the reviewers about a flow, in the browser.
pub fn report_url(entry: &Entry) -> String {
    let title = encode(&format!("Report: {}", entry.name));
    let body = encode(&format!(
        "Flow: {}@{}\nAuthor: {}\n\nWhat is wrong with it?\n\n",
        entry.slug, entry.version, entry.author
    ));
    format!("https://github.com/{REPO}/issues/new?labels=report&title={title}&body={body}")
}

/// GitHub's list of the pull requests `author` opened in the catalog.
pub fn submissions_url(author: &str) -> String {
    format!(
        "https://github.com/{REPO}/pulls?q={}",
        encode(&format!("is:pr author:{}", author.trim()))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A key made with `openssl genpkey -algorithm ed25519` for these
    /// tests only, the message it signed, and openssl's signature.
    const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----\n\
        MC4CAQAwBQYDK2VwBCIEIPscK1MVSSz6HWpY5OpocH/dNRNHrxyJ5ZC4Ip7uQRSr\n\
        -----END PRIVATE KEY-----\n";
    const TEST_PUBLIC: &str = "K0NttSSLCqz20brnxyb65tQZqYgyikGooBUKRqoeNxs=";
    const SIGNED: &[u8] = br#"{"version":1,"serial":1,"generated":"2026-10-04","flows":[]}"#;
    const OPENSSL_SIGNATURE: &str =
        "5dug6LaaBQQiFKydFvqva6tDA78s62ZFruzZY3PCwTlAd4LZ24zqVMPDit4ls+AS3VpQ/RdvREqjmC4CIZpFBg==";

    fn keys() -> Vec<String> {
        vec![TEST_PUBLIC.to_string()]
    }

    /// A folder of its own per test, removed when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("omarchist-catalog-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(dir.join("repo/flows")).unwrap();
            Self(dir)
        }

        fn repo(&self) -> PathBuf {
            self.0.join("repo")
        }

        fn out(&self) -> PathBuf {
            self.0.join("out")
        }

        fn cache(&self) -> PathBuf {
            self.0.join("cache")
        }

        fn base(&self) -> String {
            format!("file://{}", self.out().display())
        }

        fn write(&self, slug: &str, text: &str) {
            fs::write(
                self.repo()
                    .join("flows")
                    .join(format!("{slug}{SHARED_SUFFIX}")),
                text,
            )
            .unwrap();
        }

        /// Builds and signs, as the catalog's CI does.
        fn publish(&self, previous: Option<&Index>, now: u64) -> Result<Index> {
            let index = build(&self.repo(), &self.out(), previous, "2026-10-05", now)?;
            write_index(&self.out(), &index, Some(TEST_KEY))?;
            Ok(index)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn shared(name: &str, author: &str, version: u32, wait: u64) -> String {
        let mut flow = Flow::new(String::new(), name.to_string());
        flow.description = "Waits a moment, as a test.".to_string();
        flow.meta.author = author.to_string();
        flow.meta.version = version.to_string();
        flow.meta.category = "Focus".to_string();
        flow.meta.license = LICENSE.to_string();
        flow.meta.tags = vec!["test".to_string()];
        flow.steps = vec![Step::new(StepKind::Wait { ms: wait })];
        flow.to_toml().unwrap()
    }

    #[test]
    fn a_signature_made_by_openssl_verifies_and_ours_is_the_same() {
        assert!(verifies(SIGNED, OPENSSL_SIGNATURE, TEST_PUBLIC));
        assert_eq!(sign(SIGNED, TEST_KEY).unwrap(), OPENSSL_SIGNATURE);
        assert_eq!(public_key_of(TEST_KEY).unwrap(), TEST_PUBLIC);

        let published = |text: &[u8], signature: &str| {
            serde_json::to_vec(&SignedIndex {
                signed: String::from_utf8(text.to_vec()).unwrap(),
                signature: signature.to_string(),
            })
            .unwrap()
        };
        let mut tampered = SIGNED.to_vec();
        tampered[12] ^= 1;
        assert!(!verifies(&tampered, OPENSSL_SIGNATURE, TEST_PUBLIC));
        assert!(verified_index(&published(&tampered, OPENSSL_SIGNATURE), &keys()).is_err());
        assert!(verified_index(&published(SIGNED, "not a signature"), &keys()).is_err());
        assert!(verified_index(&published(SIGNED, ""), &keys()).is_err());
        // The bare index, with no signature around it, is not accepted.
        assert!(verified_index(SIGNED, &keys()).is_err());
        let honest = published(SIGNED, OPENSSL_SIGNATURE);
        assert_eq!(verified_index(&honest, &keys()).unwrap().serial, 1);
        assert_eq!(read_published(&honest).unwrap().serial, 1);

        // A key of one's own signs and verifies, and no other key does.
        let (pem, public) = new_key().unwrap();
        let signature = sign(SIGNED, &pem).unwrap();
        assert!(verifies(SIGNED, &signature, &public));
        assert!(!verifies(SIGNED, &signature, TEST_PUBLIC));
    }

    #[test]
    fn the_built_in_keys_are_ed25519_public_keys() {
        assert!(!PUBLIC_KEYS.is_empty());
        for key in PUBLIC_KEYS {
            assert_eq!(
                BASE64.decode(key).map(|bytes| bytes.len()).ok(),
                Some(32),
                "{key}"
            );
        }
        for category in CATEGORIES {
            assert!(
                category.chars().all(|c| c.is_ascii_alphabetic()),
                "{category}"
            );
        }
    }

    #[test]
    fn a_flow_is_checked_for_the_catalogs_rules() {
        let good = check_text("good.flow.toml", &shared("Pause", "ada", 1, 500), true);
        assert!(good.ok, "{:?}", good.errors);
        assert_eq!((good.steps, good.version), (1, Some(1)));
        assert_eq!(good.summary, vec!["1. wait 500 ms"]);

        // Fine as a flow, not ready for the catalog.
        let mut flow = Flow::new("saved".into(), "Hi".into());
        flow.triggers.launcher = true;
        flow.steps = vec![Step::new(StepKind::flow("other"))];
        let text = flow.to_toml().unwrap();
        assert!(check_text("x", &text, false).ok);
        let errors = check_text("x", &text, true).errors.join("\n");
        for expected in [
            "name needs 3 to 48",
            "description needs",
            "GitHub user name",
            "meta.version",
            "meta.category",
            "meta.license",
            "has no id",
            "has no triggers",
            "cannot run another flow",
        ] {
            assert!(
                errors.contains(expected),
                "missing '{expected}' in:\n{errors}"
            );
        }

        assert!(!check_text("x", "name = 3", false).ok);
        let unreadable = check_file(Path::new("/nonexistent/x.flow.toml"), false);
        assert!(!unreadable.ok && unreadable.errors[0].contains("Could not read"));
    }

    #[test]
    fn the_index_is_built_from_the_repository_and_keeps_its_promises() {
        let scratch = Scratch::new("build");
        scratch.write("pause", &shared("Pause", "ada", 1, 500));
        scratch.write("breathe", &shared("Breathe", "grace", 1, 900));
        fs::write(
            scratch.repo().join("featured.txt"),
            "# picks\npause\nmissing\n",
        )
        .unwrap();
        fs::write(scratch.repo().join("verified.txt"), "Ada\n").unwrap();

        let first = scratch.publish(None, 100).unwrap();
        assert_eq!(
            first
                .flows
                .iter()
                .map(|e| e.slug.as_str())
                .collect::<Vec<_>>(),
            vec!["breathe", "pause"]
        );
        assert_eq!(first.featured, vec!["pause"], "only flows that exist");
        assert!(first.is_verified("ada") && !first.is_verified("grace"));
        let pause = first.entry("pause").unwrap();
        assert_eq!((pause.version, pause.steps), (1, 1));
        assert_eq!(pause.kinds, vec!["wait"]);
        assert_eq!(pause.added, "2026-10-05");
        let file = scratch.out().join("v1/flows/pause/1.flow.toml");
        assert_eq!(
            hex::encode(Sha256::digest(fs::read(file).unwrap())),
            pause.sha256
        );

        // Changed without a new version, and somebody else's name taken.
        scratch.write("pause", &shared("Pause", "ada", 1, 600));
        scratch.write("breathe", &shared("Breathe", "mallory", 2, 900));
        let refused = scratch.publish(Some(&first), 200).unwrap_err().to_string();
        assert!(
            refused.contains("changed without a new version"),
            "{refused}"
        );
        assert!(refused.contains("belongs to grace"), "{refused}");

        // A new version keeps the day it was added; a pulled flow says why.
        scratch.write("pause", &shared("Pause", "ada", 2, 600));
        scratch.write("breathe", &shared("Breathe", "grace", 1, 900));
        fs::write(
            scratch.repo().join("yanked.toml"),
            "[breathe]\nreason = \"Broke\"\n",
        )
        .unwrap();
        let mut earlier = first.clone();
        earlier.flows[1].added = "2026-01-01".to_string();
        let second = scratch.publish(Some(&earlier), 50).unwrap();
        assert!(second.serial > first.serial, "the serial only grows");
        let pause = second.entry("pause").unwrap();
        assert_eq!((pause.version, pause.added.as_str()), (2, "2026-01-01"));
        assert_eq!(
            second.entry("breathe").unwrap().yanked.as_deref(),
            Some("Broke")
        );
        assert!(scratch.out().join("v1/flows/pause/1.flow.toml").exists());
        assert!(scratch.out().join("v1/flows/pause/2.flow.toml").exists());

        // Going back a version is refused too.
        scratch.write("pause", &shared("Pause", "ada", 1, 500));
        assert!(
            scratch
                .publish(Some(&second), 300)
                .unwrap_err()
                .to_string()
                .contains("older than the published")
        );
    }

    #[test]
    fn the_catalog_loads_checks_its_signature_and_survives_being_offline() {
        let scratch = Scratch::new("load");
        scratch.write("pause", &shared("Pause", "ada", 1, 500));
        let first = scratch.publish(None, 100).unwrap();
        fs::write(scratch.out().join(INSTALLS_PATH), r#"{"pause": 7}"#).unwrap();

        let catalog = load_from(&scratch.base(), &scratch.cache(), &keys()).unwrap();
        assert_eq!(catalog.index, first);
        assert_eq!(catalog.installs.get("pause"), Some(&7));
        assert!(catalog.notice.is_none());

        // The file behind an entry is the one its hash promises.
        let entry = catalog.index.entry("pause").unwrap();
        let imported = fetch_flow_from(&scratch.base(), &scratch.cache(), entry).unwrap();
        assert!(imported.flow.id.is_empty() && imported.flow.triggers.is_empty());
        assert_eq!(imported.flow.meta.source, "catalog:pause@1");
        assert_eq!(source_of(&imported.flow), Some(("pause".to_string(), 1)));
        assert!(imported.origin.contains("ada"));

        // A tampered index is not shown; the copy from before is.
        let index_file = scratch.out().join(INDEX_PATH);
        let honest = fs::read_to_string(&index_file).unwrap();
        fs::write(&index_file, honest.replace("Pause", "Pwned")).unwrap();
        let kept = load_from(&scratch.base(), &scratch.cache(), &keys()).unwrap();
        assert_eq!(kept.index, first);
        assert!(kept.notice.unwrap().contains("not signed"));
        // With no copy to fall back on, it is an error.
        assert!(load_from(&scratch.base(), &scratch.0.join("empty"), &keys()).is_err());

        // Offline: the catalog and the flow already looked at still open.
        let nowhere = format!("file://{}", scratch.0.join("gone").display());
        let offline = load_from(&nowhere, &scratch.cache(), &keys()).unwrap();
        assert_eq!(offline.index, first);
        assert!(offline.notice.unwrap().contains("as of 2026-10-05"));
        assert!(fetch_flow_from(&nowhere, &scratch.cache(), entry).is_ok());

        // A swapped flow file is refused, cached copy or not.
        let mut wrong = entry.clone();
        wrong.sha256 = "0".repeat(64);
        let refused = fetch_flow_from(&scratch.base(), &scratch.cache(), &wrong).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("not the file the catalog lists")
        );

        // An older list than the one already seen is a replay.
        fs::write(&index_file, &honest).unwrap();
        scratch.write("pause", &shared("Pause", "ada", 2, 600));
        scratch.publish(Some(&first), 200).unwrap();
        let newer = load_from(&scratch.base(), &scratch.cache(), &keys()).unwrap();
        assert_eq!(newer.index.entry("pause").unwrap().version, 2);
        fs::write(&index_file, &honest).unwrap();
        let replayed = load_from(&scratch.base(), &scratch.cache(), &keys()).unwrap();
        assert_eq!(replayed.index.entry("pause").unwrap().version, 2);
        assert!(replayed.notice.is_none());
    }

    #[test]
    fn an_installed_flow_knows_where_it_stands() {
        let entry = |version: u32, yanked: Option<&str>| Entry {
            slug: "pause".into(),
            version,
            yanked: yanked.map(str::to_string),
            ..Entry::default()
        };
        let index = |entry: Entry| Index {
            flows: vec![entry],
            ..Index::default()
        };
        let mut flow = Flow::new("pause".into(), "Pause".into());
        assert_eq!(
            standing(&flow, &index(entry(1, None))),
            None,
            "not from the catalog"
        );
        flow.meta.source = "https://example.com/pause.flow.toml".into();
        assert_eq!(source_of(&flow), None);

        flow.meta.source = "catalog:pause@2".into();
        assert_eq!(
            standing(&flow, &index(entry(2, None))),
            Some(Standing::Current)
        );
        assert_eq!(
            standing(&flow, &index(entry(3, None))),
            Some(Standing::Update(3))
        );
        assert_eq!(
            standing(&flow, &index(entry(3, Some("Broke")))),
            Some(Standing::Pulled("Broke".into()))
        );
        assert_eq!(standing(&flow, &Index::default()), Some(Standing::Gone));
    }

    #[test]
    fn two_versions_are_compared_step_by_step() {
        let wait = |ms: u64| Step::new(StepKind::Wait { ms });
        let mut old = Flow::new("f".into(), "F".into());
        old.steps = vec![wait(1000), wait(2000), wait(3000)];
        let mut new = old.clone();
        new.steps = vec![
            wait(1000),
            Step::new(StepKind::Repeat {
                times: 2,
                steps: vec![wait(5000)],
            }),
            wait(3000),
        ];
        let rows: Vec<(Change, usize, String)> = diff_steps(&old, &new)
            .into_iter()
            .map(|row| (row.change, row.depth, row.step.kind.type_tag()))
            .collect();
        assert_eq!(
            rows,
            vec![
                (Change::Same, 0, "wait".to_string()),
                (Change::Removed, 0, "wait".to_string()),
                (Change::Added, 0, "repeat".to_string()),
                (Change::Added, 1, "wait".to_string()),
                (Change::Same, 0, "wait".to_string()),
            ]
        );
        assert!(
            diff_steps(&old, &old)
                .iter()
                .all(|row| row.change == Change::Same)
        );

        // A setting that does not show in a step's one-line text still
        // makes it a changed step.
        let mut said = Flow::new("f".into(), "F".into());
        said.steps = vec![Step::new(StepKind::notify("Done", "All good"))];
        let mut says = said.clone();
        says.steps = vec![Step::new(StepKind::notify("Done", "All fine"))];
        assert_eq!(
            diff_steps(&said, &says)
                .iter()
                .map(|row| row.change)
                .collect::<Vec<_>>(),
            vec![Change::Removed, Change::Added]
        );
    }

    #[test]
    fn a_flow_is_made_ready_for_the_catalog() {
        let mut flow = Flow::new("my-pause".into(), "Pause".into());
        flow.description = "Waits a moment, as a test.".into();
        flow.triggers.startup = true;
        flow.meta.source = "catalog:other@1".into();
        flow.steps = vec![Step::new(StepKind::Wait { ms: 500 })];

        let tags = vec!["test".to_string()];
        let new = prepare_submission(&flow, "@ada", "Focus", &tags, None).unwrap();
        assert_eq!(
            (new.slug.as_str(), new.version, new.update),
            ("pause", 1, false)
        );
        assert_eq!(new.file_name, "pause.flow.toml");
        assert!(new.url.starts_with(
            "https://github.com/tahayvr/omarchist-flows/new/main?filename=flows%2Fpause%2Eflow%2Etoml"
        ));
        assert!(new.url.contains("&value="));
        let report = check_text(&new.file_name, &new.toml, true);
        assert!(report.ok, "{:?}", report.errors);
        assert_eq!(report.author, "ada");
        assert!(!new.toml.contains("triggers") && !new.toml.contains("catalog:other"));

        // The catalog has it already: one's own gets a new version, and
        // somebody else's name is taken.
        let index = |author: &str| Index {
            flows: vec![Entry {
                slug: "pause".into(),
                name: "Pause".into(),
                author: author.into(),
                version: 3,
                ..Entry::default()
            }],
            ..Index::default()
        };
        let update = prepare_submission(&flow, "Ada", "Focus", &tags, Some(&index("ada"))).unwrap();
        assert_eq!((update.version, update.update), (4, true));
        assert!(update.url.ends_with("/edit/main/flows/pause.flow.toml"));
        let taken = prepare_submission(&flow, "ada", "Focus", &tags, Some(&index("grace")));
        assert!(taken.unwrap_err().to_string().contains("by grace"));

        // What the catalog would refuse is refused here first.
        assert!(prepare_submission(&flow, "ada", "Nonsense", &tags, None).is_err());
        assert!(prepare_submission(&flow, "not a name!", "Focus", &tags, None).is_err());

        // A long flow travels on the clipboard, not in the address.
        flow.steps = (0..90)
            .map(|n| {
                Step::new(StepKind::Exec {
                    command: format!("echo {}", "x".repeat(60 + n)),
                    wait: true,
                })
            })
            .collect();
        let long = prepare_submission(&flow, "ada", "Focus", &tags, None).unwrap();
        assert!(!long.url.contains("&value=") && long.url.len() < MAX_URL);
    }

    #[test]
    fn links_to_github_carry_what_they_are_about() {
        let entry = Entry {
            slug: "pause".into(),
            name: "Pause & go".into(),
            author: "ada".into(),
            version: 2,
            ..Entry::default()
        };
        let url = report_url(&entry);
        assert!(url.starts_with("https://github.com/tahayvr/omarchist-flows/issues/new?"));
        assert!(url.contains("Report%3A%20Pause%20%26%20go"));
        assert!(url.contains("pause%402"));
        assert!(submissions_url("ada").ends_with("is%3Apr%20author%3Aada"));
    }
}
