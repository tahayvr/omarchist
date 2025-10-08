use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use log::warn;

use crate::types::{
    HyprlandConfigError, HyprlandResult, KeyboardCatalog, KeyboardLayout, KeyboardModel,
    KeyboardOption, KeyboardOptionGroup, KeyboardVariant,
};

const DEFAULT_RULE_FILE_NAMES: &[&str] = &[
    "evdev.lst",
    "base.lst",
    "xorg.lst",
    "evdev.extras",
    "base.extras",
    "xorg.extras",
];

fn default_keyboard_rule_paths() -> Vec<PathBuf> {
    let mut roots = discover_rule_roots_from_environment();
    roots.extend(default_rule_roots());
    // Always ensure the canonical base.lst is included even if earlier roots cover it.
    roots.push(PathBuf::from("/usr/share/X11/xkb/rules/base.lst"));

    build_rule_paths(roots)
}

fn discover_rule_roots_from_environment() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(value) = env::var("XKB_CONFIG_ROOT") {
        roots.extend(split_path_list(&value));
    }

    if let Ok(value) = env::var("XKB_RULES_ROOT") {
        roots.extend(split_path_list(&value));
    }

    if let Ok(value) = env::var("XDG_DATA_HOME") {
        if let Some(home) = normalize_env_path(&value) {
            roots.push(home.join("X11/xkb"));
        }
    }

    if let Ok(value) = env::var("XDG_DATA_DIRS") {
        for entry in value.split(':') {
            if let Some(dir) = normalize_env_path(entry) {
                roots.push(dir.join("X11/xkb"));
            }
        }
    }

    roots
}

fn default_rule_roots() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/share/X11/xkb"),
        PathBuf::from("/usr/share/X11/xkb/rules"),
        PathBuf::from("/usr/local/share/X11/xkb"),
        PathBuf::from("/usr/local/share/X11/xkb/rules"),
        PathBuf::from("/etc/X11/xkb"),
        PathBuf::from("/etc/X11/xkb/rules"),
        PathBuf::from("/run/current-system/sw/share/X11/xkb"),
        PathBuf::from("/run/current-system/sw/share/X11/xkb/rules"),
        PathBuf::from("/nix/var/nix/profiles/default/share/X11/xkb"),
        PathBuf::from("/nix/var/nix/profiles/default/share/X11/xkb/rules"),
    ]
}

fn split_path_list(value: &str) -> Vec<PathBuf> {
    value.split(':').filter_map(normalize_env_path).collect()
}

fn normalize_env_path(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

fn build_rule_paths<I>(roots: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = PathBuf>,
{
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut candidates = Vec::new();

    for root in roots {
        if root.as_os_str().is_empty() {
            continue;
        }

        if root
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("lst"))
            .unwrap_or(false)
        {
            push_candidate(&mut candidates, &mut seen, root);
            continue;
        }

        if root
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case("rules"))
            .unwrap_or(false)
        {
            append_rule_files_from_directory(&mut candidates, &mut seen, &root);
            continue;
        }

        // Treat as the XKB root directory.
        append_rule_files_from_directory(&mut candidates, &mut seen, &root.join("rules"));
    }

    candidates
}

fn append_rule_files_from_directory(
    candidates: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
    directory: &Path,
) {
    for name in DEFAULT_RULE_FILE_NAMES {
        let path = directory.join(name);
        push_candidate(candidates, seen, path);
    }
}

fn push_candidate(candidates: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>, candidate: PathBuf) {
    if seen.insert(candidate.clone()) {
        candidates.push(candidate);
    }
}

const FALLBACK_RULES: &str = r"#! fallback XKB rules used when system definitions are unavailable
! model
  pc101           Generic 101-key PC
  pc104           Generic 104-key PC
  pc105           Generic 105-key PC
  apple_laptop    Apple laptop
  thinkpad        Lenovo ThinkPad

! layout
  us              English (US)
  gb              English (UK)
  de              German
  fr              French
  es              Spanish

! variant
  intl            us: English (US, intl., with dead keys)
  altgr           gb: English (UK, intl., with AltGr)
  nodeadkeys      de: German (eliminate dead keys)
  oss             fr: French (alternative)
  deadtilde       es: Spanish (dead tilde)

! option
  grp             Switching to another layout
  grp:toggle      Right Alt toggles layout
  grp:ctrl_shift_toggle Left Ctrl+Left Shift switches layout
  grp:alt_shift_toggle Left Alt+Left Shift switches layout
  grp:win_space_toggle Super+Space toggles layout
layout switching continuation line
  caps            Caps Lock behavior
  caps:escape     Caps Lock becomes Escape
  caps:super      Caps Lock becomes Super
  altwin          Alt/Win key behavior
  altwin:swap_lalt_lwin Swap Left Alt with Left Win
  altwin:meta_alt Left Alt is Meta key
";

pub fn load_keyboard_catalog() -> HyprlandResult<KeyboardCatalog> {
    let candidate_paths = default_keyboard_rule_paths();
    load_keyboard_catalog_from_candidates(candidate_paths)
}

fn load_keyboard_catalog_from_candidates<I, P>(candidates: I) -> HyprlandResult<KeyboardCatalog>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut last_error: Option<HyprlandConfigError> = None;

    for candidate in candidates {
        let path = candidate.as_ref();
        let path_display = path.display().to_string();

        match parse_keyboard_catalog_from_path(path) {
            Ok(catalog)
                if catalog.models.is_empty()
                    && catalog.layouts.is_empty()
                    && catalog.option_groups.is_empty() =>
            {
                last_error = Some(HyprlandConfigError::Parse {
                    field: "keyboard catalog".to_string(),
                    message: format!("No keyboard definitions found in {path_display}"),
                });
            },
            Ok(catalog) => return Ok(catalog),
            Err(HyprlandConfigError::FileNotFound { .. }) => {
                last_error = Some(HyprlandConfigError::FileNotFound { path: path_display });
            },
            Err(err) => {
                last_error = Some(err);
            },
        }
    }

    if let Some(err) = &last_error {
        warn!(
            "Falling back to bundled keyboard catalog because system catalogs failed: {}",
            err
        );
    } else {
        warn!(
            "No XKB keyboard rule files found; using bundled fallback catalog so controls remain usable."
        );
    }

    fallback_keyboard_catalog()
}

fn fallback_keyboard_catalog() -> HyprlandResult<KeyboardCatalog> {
    parse_keyboard_catalog(FALLBACK_RULES).map_err(|err| {
        warn!(
            "Bundled fallback keyboard catalog failed to parse (this should not happen): {}",
            err
        );
        err
    })
}

fn parse_keyboard_catalog_from_path(path: &Path) -> HyprlandResult<KeyboardCatalog> {
    if !path.exists() {
        return Err(HyprlandConfigError::FileNotFound {
            path: path.display().to_string(),
        });
    }

    let contents = fs::read_to_string(path)?;
    parse_keyboard_catalog(&contents)
}

fn parse_keyboard_catalog(contents: &str) -> HyprlandResult<KeyboardCatalog> {
    enum Section {
        None,
        Models,
        Layouts,
        Variants,
        Options,
    }

    let mut section = Section::None;
    let mut catalog = KeyboardCatalog::default();
    let mut layout_indices: BTreeMap<String, usize> = BTreeMap::new();
    let mut option_group_indices: BTreeMap<String, usize> = BTreeMap::new();
    let mut last_variant: Option<(usize, usize)> = None;
    let mut last_option: Option<(usize, usize)> = None;
    let mut last_group: Option<usize> = None;

    for raw_line in contents.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }

        if raw_line.trim_start().starts_with('!') {
            let header = raw_line.trim().to_lowercase();
            section = if header.starts_with("! model") {
                Section::Models
            } else if header.starts_with("! layout") {
                Section::Layouts
            } else if header.starts_with("! variant") {
                Section::Variants
            } else if header.starts_with("! option") {
                Section::Options
            } else {
                Section::None
            };
            last_variant = None;
            last_option = None;
            last_group = None;
            continue;
        }

        if raw_line.trim_start().starts_with('#') {
            continue;
        }

        match section {
            Section::Models => {
                if let Some((name, description)) = split_code_description(raw_line) {
                    catalog.models.push(KeyboardModel { name, description });
                }
            },
            Section::Layouts => {
                if let Some((name, description)) = split_code_description(raw_line) {
                    let index = catalog.layouts.len();
                    layout_indices.insert(name.clone(), index);
                    catalog.layouts.push(KeyboardLayout {
                        name,
                        description,
                        variants: Vec::new(),
                    });
                }
            },
            Section::Variants => {
                if let Some((variant_name, rest)) = split_code_description(raw_line) {
                    if let Some((layout_code, description)) = rest.split_once(':') {
                        let layout_code = layout_code.trim();
                        if let Some(&layout_index) = layout_indices.get(layout_code) {
                            let description = description.trim().to_string();
                            let variants = &mut catalog.layouts[layout_index].variants;
                            variants.push(KeyboardVariant {
                                name: variant_name,
                                description,
                            });
                            last_variant = Some((layout_index, variants.len() - 1));
                        }
                    }
                } else if let Some((layout_index, variant_index)) = last_variant {
                    let continuation = raw_line.trim();
                    if !continuation.is_empty() {
                        let variant = &mut catalog.layouts[layout_index].variants[variant_index];
                        variant.description.push(' ');
                        variant.description.push_str(continuation);
                    }
                }
            },
            Section::Options => {
                let starts_with_whitespace = raw_line
                    .chars()
                    .next()
                    .map(|ch| ch.is_whitespace())
                    .unwrap_or(false);

                if !starts_with_whitespace {
                    let continuation = raw_line.trim();
                    if continuation.is_empty() {
                        continue;
                    }

                    if let Some((group_index, option_index)) = last_option {
                        let option = &mut catalog.option_groups[group_index].options[option_index];
                        if !option.description.is_empty() {
                            option.description.push(' ');
                        }
                        option.description.push_str(continuation);
                        continue;
                    } else if let Some(group_index) = last_group {
                        let group = &mut catalog.option_groups[group_index];
                        if !group.description.is_empty() {
                            group.description.push(' ');
                        }
                        group.description.push_str(continuation);
                        continue;
                    }
                }

                if let Some((identifier, description)) = split_code_description(raw_line) {
                    if let Some((group_name, option_name)) = identifier.split_once(':') {
                        let group_index = ensure_option_group(
                            &mut catalog,
                            &mut option_group_indices,
                            group_name.trim(),
                            None,
                        );
                        let options = &mut catalog.option_groups[group_index].options;
                        options.push(KeyboardOption {
                            name: option_name.trim().to_string(),
                            description: description.to_string(),
                        });
                        last_option = Some((group_index, options.len() - 1));
                        last_group = Some(group_index);
                    } else {
                        let group_index = ensure_option_group(
                            &mut catalog,
                            &mut option_group_indices,
                            identifier.as_str(),
                            Some(description.to_string()),
                        );
                        catalog.option_groups[group_index].description = description;
                        last_group = Some(group_index);
                        last_option = None;
                    }
                } else {
                    let continuation = raw_line.trim();
                    if continuation.is_empty() {
                        continue;
                    }

                    if let Some((group_index, option_index)) = last_option {
                        let option = &mut catalog.option_groups[group_index].options[option_index];
                        option.description.push(' ');
                        option.description.push_str(continuation);
                    } else if let Some(group_index) = last_group {
                        let group = &mut catalog.option_groups[group_index];
                        if !group.description.is_empty() {
                            group.description.push(' ');
                        }
                        group.description.push_str(continuation);
                    }
                }
            },
            Section::None => {
                // Ignore unrecognised sections
            },
        }
    }

    Ok(catalog)
}

fn ensure_option_group(
    catalog: &mut KeyboardCatalog,
    group_indices: &mut BTreeMap<String, usize>,
    name: &str,
    description: Option<String>,
) -> usize {
    if let Some(&index) = group_indices.get(name) {
        return index;
    }

    let index = catalog.option_groups.len();
    catalog.option_groups.push(KeyboardOptionGroup {
        name: name.to_string(),
        description: description.unwrap_or_default(),
        options: Vec::new(),
    });
    group_indices.insert(name.to_string(), index);
    index
}

fn split_code_description(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let mut first_whitespace_index = None;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_whitespace() {
            first_whitespace_index = Some(idx);
            break;
        }
    }

    let idx = first_whitespace_index.unwrap_or(trimmed.len());
    if idx == trimmed.len() {
        return None;
    }

    let code = trimmed[..idx].to_string();
    let description = trimmed[idx..].trim().to_string();
    if description.is_empty() {
        None
    } else {
        Some((code, description))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::NamedTempFile;

    const SAMPLE: &str = r"! model
  pc105           Generic 105-key PC

! layout
  us              English (US)

! variant
  intl            us: English (US, intl., with dead keys)

! option
  grp             Switching to another layout
  grp:toggle      Right Alt
  grp:ctrl_shift_toggle Left Ctrl+Left Shift chooses previous layout
layout continuation
  lv3             Key to choose the 3rd level
  lv3:ralt_switch Right Alt";

    #[test]
    fn parses_sample_catalog() {
        let catalog = parse_keyboard_catalog(SAMPLE).expect("parse sample catalog");
        assert_eq!(catalog.models.len(), 1);
        assert_eq!(catalog.layouts.len(), 1);
        assert_eq!(catalog.layouts[0].name, "us");
        assert_eq!(catalog.layouts[0].variants.len(), 1);
        assert_eq!(catalog.layouts[0].variants[0].name, "intl");
        assert_eq!(
            catalog.layouts[0].variants[0].description,
            "English (US, intl., with dead keys)"
        );
        assert_eq!(catalog.option_groups.len(), 2);
        assert_eq!(catalog.option_groups[0].name, "grp");
        assert_eq!(
            catalog.option_groups[0].description,
            "Switching to another layout"
        );
        assert_eq!(catalog.option_groups[0].options.len(), 2);
        assert_eq!(
            catalog.option_groups[0].options[1].name,
            "ctrl_shift_toggle"
        );
        assert!(catalog.option_groups[0].options[1]
            .description
            .contains("previous layout layout continuation"));
        assert_eq!(catalog.option_groups[1].name, "lv3");
        assert_eq!(catalog.option_groups[1].options.len(), 1);
    }

    #[test]
    fn falls_back_to_next_rules_file_when_first_missing() {
        let temp = NamedTempFile::new().expect("create temp XKB rules file");
        std::fs::write(temp.path(), SAMPLE).expect("write sample data");

        let missing = Path::new("/__does_not_exist__/xkb/rules.lst");
        let catalog = load_keyboard_catalog_from_candidates([missing, temp.path()])
            .expect("load catalog from fallback file");

        assert_eq!(catalog.models.len(), 1);
        assert_eq!(catalog.layouts.len(), 1);
        assert_eq!(catalog.option_groups.len(), 2);
    }

    #[test]
    fn uses_bundled_catalog_when_all_candidates_fail() {
        let missing_one = Path::new("/__does_not_exist__/xkb/rules.lst");
        let missing_two = Path::new("/__does_not_exist__/xkb/evdev.lst");
        let catalog = load_keyboard_catalog_from_candidates([missing_one, missing_two])
            .expect("load bundled fallback catalog");

        assert!(!catalog.models.is_empty());
        assert!(catalog.layouts.iter().any(|layout| layout.name == "us"));
        assert!(catalog
            .option_groups
            .iter()
            .any(|group| group.name == "grp"));
    }

    #[test]
    fn fallback_constant_matches_parser() {
        let catalog = fallback_keyboard_catalog().expect("fallback parses");
        assert!(!catalog.models.is_empty());
        assert!(catalog.layouts.iter().any(|layout| layout.name == "us"));
        assert!(catalog
            .option_groups
            .iter()
            .any(|group| group.name == "grp"));
    }

    #[test]
    fn build_rule_paths_accepts_direct_lst_files() {
        let custom = PathBuf::from("/tmp/custom_rules.lst");
        let candidates = build_rule_paths(vec![custom.clone()]);
        assert_eq!(candidates, vec![custom]);
    }

    #[test]
    fn build_rule_paths_extends_rule_directories() {
        let root = PathBuf::from("/opt/xkb/rules");
        let candidates = build_rule_paths(vec![root.clone()]);
        assert!(candidates.contains(&root.join("evdev.lst")));
        assert!(candidates.contains(&root.join("base.lst")));
    }

    #[test]
    fn default_paths_always_include_system_base_lst() {
        let candidates = default_keyboard_rule_paths();
        assert!(candidates.contains(&PathBuf::from("/usr/share/X11/xkb/rules/base.lst")));
    }
}
