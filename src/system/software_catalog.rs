//! The software Omarchy can install and remove, taken from Omarchy's own
//! menu definition (`default/omarchy/omarchy-menu.jsonc`, plus the user's
//! extensions): every `install.*` entry with an action, paired with its
//! `remove.*` counterpart. Each entry's `when` condition says whether it
//! applies right now, and its `action` is run exactly as the menu runs
//! it, so this page can never drift from what the menu offers.
use std::collections::BTreeMap;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::error::{Error, Result};
use crate::system::omarchy_paths::{menu_file, user_menu_extensions_file};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    pub label: String,
    pub action: Option<String>,
    pub when: Option<String>,
}

/// One thing that can be installed or removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareItem {
    /// The menu key without its `install.` prefix (`browser.chrome`).
    pub id: String,
    pub label: String,
    pub install: MenuAction,
    pub remove: Option<MenuAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuAction {
    pub action: String,
    pub when: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareGroup {
    pub title: String,
    pub items: Vec<SoftwareItem>,
}

/// Whether an item can be installed or removed right now, from its
/// conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Availability {
    pub install: bool,
    pub remove: bool,
}

/// Parses the menu's JSONC: `//` line comments and trailing commas are
/// allowed, as Omarchy's file uses both.
pub fn parse_menu(jsonc: &str) -> Result<BTreeMap<String, MenuEntry>> {
    let json = strip_jsonc(jsonc);
    let value: Value = serde_json::from_str(&json)
        .map_err(|e| Error::json("Failed to parse omarchy-menu.jsonc", e))?;
    let Some(object) = value.as_object() else {
        return Err(Error::Invalid("omarchy-menu.jsonc is not an object".into()));
    };
    Ok(object
        .iter()
        .filter_map(|(key, entry)| {
            let label = entry.get("label")?.as_str()?.to_string();
            Some((
                key.clone(),
                MenuEntry {
                    label,
                    action: entry
                        .get("action")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    when: entry
                        .get("when")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                },
            ))
        })
        .collect())
}

fn strip_jsonc(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        // A comment is a line starting with `//`; the entries' strings hold
        // `//` inside URLs, so only whole-line comments are dropped.
        if line.trim_start().starts_with("//") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    // Trailing commas before a closing brace or bracket.
    let mut result = String::with_capacity(out.len());
    let chars: Vec<char> = out.chars().collect();
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            result.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_string = true;
            result.push(c);
        } else if c == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                // drop the comma
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
        i += 1;
    }
    result
}

/// Omarchy's menu with the user's extensions laid over it.
pub fn load_menu() -> Result<BTreeMap<String, MenuEntry>> {
    let content = std::fs::read_to_string(menu_file())
        .map_err(|e| Error::io("Failed to read omarchy-menu.jsonc", e))?;
    let mut menu = parse_menu(&content)?;
    if let Some(path) = user_menu_extensions_file()
        && let Ok(content) = std::fs::read_to_string(path)
        && let Ok(extensions) = parse_menu(&content)
    {
        menu.extend(extensions);
    }
    Ok(menu)
}

/// The catalog: every installable entry, grouped by its top-level menu
/// section, with its remove counterpart when the menu has one.
pub fn catalog(menu: &BTreeMap<String, MenuEntry>) -> Vec<SoftwareGroup> {
    let mut groups: Vec<SoftwareGroup> = Vec::new();
    for (key, entry) in menu {
        let Some(path) = key.strip_prefix("install.") else {
            continue;
        };
        let Some(action) = &entry.action else {
            continue;
        };
        let segments: Vec<&str> = path.split('.').collect();
        let title = if segments.len() > 1 {
            menu.get(&format!("install.{}", segments[0]))
                .map(|g| g.label.clone())
                .unwrap_or_else(|| segments[0].to_string())
        } else {
            "Other".to_string()
        };
        let leaf = segments.last().copied().unwrap_or(path);
        let remove = [
            format!("remove.{path}"),
            format!("remove.{leaf}"),
            segments
                .first()
                .map(|g| format!("remove.{g}.{leaf}"))
                .unwrap_or_default(),
        ]
        .iter()
        .filter_map(|k| menu.get(k))
        .find_map(|e| {
            e.action.as_ref().map(|action| MenuAction {
                action: action.clone(),
                when: e.when.clone(),
            })
        });
        let item = SoftwareItem {
            id: path.to_string(),
            label: entry.label.clone(),
            install: MenuAction {
                action: action.clone(),
                when: entry.when.clone(),
            },
            remove,
        };
        match groups.iter_mut().find(|g| g.title == title) {
            Some(group) => group.items.push(item),
            None => groups.push(SoftwareGroup {
                title,
                items: vec![item],
            }),
        }
    }
    // Named sections first, the loose tools last.
    groups.sort_by_key(|g| g.title == "Other");
    groups
}

/// Evaluates a menu `when` condition the way the shell does: as a bash
/// expression whose exit status decides.
pub fn condition_holds(when: Option<&str>) -> bool {
    let Some(when) = when else {
        return true;
    };
    Command::new("bash")
        .args(["-c", when])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub fn availability(item: &SoftwareItem) -> Availability {
    Availability {
        install: condition_holds(item.install.when.as_deref()),
        remove: item
            .remove
            .as_ref()
            .is_some_and(|r| condition_holds(r.when.as_deref())),
    }
}

/// Runs a menu action as the shell's menu does: detached, through a
/// login shell so Omarchy's `bin` is on the path.
pub fn run_action(action: &str) -> Result<()> {
    Command::new("bash")
        .args(["-lc", action])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| Error::io("Failed to run the menu action", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
  // A comment
  "install.browser": {"icon":"","label":"Browser"},
  "install.browser.chrome": {"label":"Chrome","when":"! omarchy-pkg-present google-chrome","action":"omarchy-launch-floating-terminal-with-presentation 'omarchy-install-browser chrome'"},
  "install.ai.dictation": {"label":"Dictation","when":"! omarchy-pkg-present voxtype-bin","action":"omarchy-voxtype-install"},
  "install.ai": {"label":"AI"},
  "install.webapp": {"label":"Web App","action":"omarchy-webapp-install"},
  "remove.browser.chrome": {"label":"Chrome","when":"omarchy-pkg-present google-chrome","action":"omarchy-remove-browser chrome"},
  "remove.dictation": {"label":"Dictation","when":"omarchy-pkg-present voxtype-bin","action":"omarchy-voxtype-remove"},
  "trigger.emoji": {"label":"Emoji","action":"omarchy-menu-emoji", "url": "https://x/y"},
}"#;

    #[test]
    fn jsonc_with_comments_and_trailing_commas_parses() {
        let menu = parse_menu(SAMPLE).unwrap();
        assert_eq!(menu.len(), 8);
        assert_eq!(menu["install.browser"].label, "Browser");
        assert_eq!(
            menu["trigger.emoji"].action.as_deref(),
            Some("omarchy-menu-emoji")
        );
    }

    #[test]
    fn catalog_groups_installs_and_pairs_removes() {
        let menu = parse_menu(SAMPLE).unwrap();
        let groups = catalog(&menu);
        let titles: Vec<&str> = groups.iter().map(|g| g.title.as_str()).collect();
        assert_eq!(titles, ["AI", "Browser", "Other"]);
        let chrome = &groups[1].items[0];
        assert_eq!(chrome.id, "browser.chrome");
        assert_eq!(
            chrome.remove.as_ref().unwrap().action,
            "omarchy-remove-browser chrome"
        );
        let dictation = &groups[0].items[0];
        assert_eq!(
            dictation.remove.as_ref().unwrap().action,
            "omarchy-voxtype-remove",
            "a remove entry one level up is matched by its leaf name"
        );
        let webapp = &groups[2].items[0];
        assert!(webapp.remove.is_none());
        assert!(webapp.install.when.is_none());
    }

    #[test]
    fn conditions_are_bash_expressions() {
        assert!(condition_holds(None));
        assert!(condition_holds(Some("[[ 1 == 1 ]]")));
        assert!(!condition_holds(Some("[[ 1 == 2 ]]")));
    }

    /// Omarchy's real menu parses, and every install entry is in a group.
    #[test]
    fn omarchys_menu_parses() {
        let Ok(content) = std::fs::read_to_string(menu_file()) else {
            eprintln!("skipping: no Omarchy menu");
            return;
        };
        let menu = parse_menu(&content).unwrap();
        let groups = catalog(&menu);
        assert!(
            groups.len() >= 5,
            "{:?}",
            groups.iter().map(|g| &g.title).collect::<Vec<_>>()
        );
        assert!(groups.iter().any(|g| g.title == "Browser"));
        assert!(
            groups
                .iter()
                .flat_map(|g| &g.items)
                .any(|i| i.id == "browser.chrome" && i.remove.is_some())
        );
    }
}
