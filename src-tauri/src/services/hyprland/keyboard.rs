use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use log::warn;

use crate::types::{
    HyprlandConfigError, HyprlandResult, KeyboardCatalog, KeyboardLayout, KeyboardModel,
    KeyboardOption, KeyboardOptionGroup, KeyboardVariant,
};

/// The canonical system XKB rules file containing keyboard definitions.
const SYSTEM_BASE_LST: &str = "/usr/share/X11/xkb/rules/base.lst";

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

/// Load the keyboard catalog from the system XKB rules file, falling back to bundled definitions.
pub fn load_keyboard_catalog() -> HyprlandResult<KeyboardCatalog> {
    match parse_keyboard_catalog_from_path(Path::new(SYSTEM_BASE_LST)) {
        Ok(catalog)
            if !catalog.models.is_empty()
                || !catalog.layouts.is_empty()
                || !catalog.option_groups.is_empty() =>
        {
            Ok(catalog)
        }
        Ok(_) => {
            warn!(
                "System XKB rules file at {} contained no definitions; using bundled fallback",
                SYSTEM_BASE_LST
            );
            fallback_keyboard_catalog()
        }
        Err(err) => {
            warn!(
                "Failed to load system XKB rules from {}: {}; using bundled fallback",
                SYSTEM_BASE_LST, err
            );
            fallback_keyboard_catalog()
        }
    }
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
