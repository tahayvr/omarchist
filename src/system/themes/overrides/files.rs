use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::shell::theme_sh_commands::theme_color_all;
use crate::system::omarchy_paths::{
    themed_templates_dir, user_themed_templates_dir, user_themes_dir,
};

use super::registry::{OverrideSpec, Seed};
use super::template::{Palette, render};
use super::validate::validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverrideStatus {
    /// No file in the theme; Omarchy generates it from the palette.
    Generated,
    /// The theme ships its own file.
    Custom,
}

fn theme_dir(theme: &str) -> Result<PathBuf> {
    let dir = user_themes_dir()
        .ok_or(Error::UnknownDirectory("custom themes"))?
        .join(theme);
    if dir.is_dir() {
        Ok(dir)
    } else {
        Err(Error::ThemeNotFound(theme.to_string()))
    }
}

pub fn status(theme: &str, spec: &OverrideSpec) -> OverrideStatus {
    match theme_dir(theme) {
        Ok(dir) if dir.join(spec.file).is_file() => OverrideStatus::Custom,
        _ => OverrideStatus::Generated,
    }
}

/// The theme's own copy of the file, or `None` when Omarchy generates it.
pub fn read(theme: &str, spec: &OverrideSpec) -> Result<Option<String>> {
    let path = theme_dir(theme)?.join(spec.file);
    if !path.is_file() {
        return Ok(None);
    }
    fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| Error::io(format!("Failed to read {}", spec.file), e))
}

/// Validates `content` and writes it atomically. Invalid content is never
/// written, so the file on disk is always one Omarchy can use.
pub fn write(theme: &str, spec: &OverrideSpec, content: &str) -> Result<()> {
    validate(spec, content)?;

    let dir = theme_dir(theme)?;
    // A dotfile: `omarchy-theme-set` copies the folder with `*`, which skips it.
    let tmp = dir.join(format!(".{}.tmp", spec.file));
    fs::write(&tmp, content).map_err(|e| Error::io(format!("Failed to write {}", spec.file), e))?;
    fs::rename(&tmp, dir.join(spec.file))
        .map_err(|e| Error::io(format!("Failed to write {}", spec.file), e))
}

/// Deletes the theme's copy so Omarchy generates the file again.
pub fn remove(theme: &str, spec: &OverrideSpec) -> Result<()> {
    let path = theme_dir(theme)?.join(spec.file);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| Error::io(format!("Failed to remove {}", spec.file), e))?;
    }
    Ok(())
}

/// The theme's palette as Omarchy resolves it.
pub fn palette(theme: &str) -> Result<Palette> {
    let colors = theme_dir(theme)?.join("colors.toml");
    Ok(Palette::parse(&theme_color_all(&colors)?))
}

/// What Omarchy would generate for this file from the theme's current palette.
pub fn generated(theme: &str, spec: &OverrideSpec) -> Result<String> {
    match spec.seed {
        Seed::Fixed(text) => Ok(text.to_string()),
        Seed::Template(template) => Ok(render(&read_template(template)?, &palette(theme)?)),
        Seed::ShellSection(section) => {
            let shell = render(&read_template("shell.toml.tpl")?, &palette(theme)?);
            shell_section(&shell, section)
                .ok_or_else(|| Error::Invalid(format!("shell.toml.tpl has no [{section}] section")))
        }
    }
}

/// The user's template of this name if there is one, else Omarchy's.
fn read_template(name: &str) -> Result<String> {
    let path = user_themed_templates_dir()
        .map(|dir| dir.join(name))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| themed_templates_dir().join(name));
    fs::read_to_string(&path).map_err(|e| Error::io(format!("Failed to read template {name}"), e))
}

// The section name of a `[section]` header line, matching the header pattern
// `omarchy-theme-set-templates` uses (a trailing comment is allowed).
fn header_name(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix('[')?;
    let (name, after) = rest.split_once(']')?;
    let after = after.trim_start();
    (!name.is_empty() && (after.is_empty() || after.starts_with('#'))).then_some(name)
}

/// Cuts one section, header included, out of a rendered `shell.toml`.
pub fn shell_section(shell: &str, section: &str) -> Option<String> {
    let mut lines = shell
        .lines()
        .skip_while(|line| header_name(line) != Some(section));
    let header = lines.next()?;
    let mut body: Vec<&str> = lines
        .take_while(|line| header_name(line).is_none())
        .collect();
    while body.last().is_some_and(|line| line.trim().is_empty()) {
        body.pop();
    }
    let mut out = format!("{}\n", header.trim());
    for line in body {
        out.push_str(line);
        out.push('\n');
    }
    Some(out)
}

/// Every `[section]` of a `shell.toml`, in order.
pub fn shell_sections(shell: &str) -> Vec<&str> {
    shell.lines().filter_map(header_name).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{header_name, shell_section, shell_sections};
    use crate::system::omarchy_paths::themed_templates_dir;
    use crate::system::themes::overrides::registry::OVERRIDES;

    const SHELL: &str = "# comment\n\n[bar]\n# help\nbackground = \"#000000\"\n\n[lock] # trailing\ntext = \"#ffffff\"\n\n\n";

    #[test]
    fn headers() {
        assert_eq!(header_name("[bar]"), Some("bar"));
        assert_eq!(header_name("  [image-picker]  # c"), Some("image-picker"));
        assert_eq!(header_name("colors = [1, 2]"), None);
        assert_eq!(header_name("[bar] x"), None);
    }

    #[test]
    fn cuts_a_section_with_its_header_and_without_trailing_blanks() {
        assert_eq!(
            shell_section(SHELL, "bar").unwrap(),
            "[bar]\n# help\nbackground = \"#000000\"\n"
        );
        assert_eq!(
            shell_section(SHELL, "lock").unwrap(),
            "[lock] # trailing\ntext = \"#ffffff\"\n"
        );
        assert!(shell_section(SHELL, "menu").is_none());
    }

    // Compares the renderer with what Omarchy staged for the active theme, when
    // that theme is one of the user's. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn generated_matches_the_active_theme() {
        use crate::system::omarchy_paths::{current_theme_dir, current_theme_name_file};
        use crate::system::themes::overrides::registry::Seed;

        let name = fs::read_to_string(current_theme_name_file().unwrap()).unwrap();
        let name = name.trim();
        let current = current_theme_dir().unwrap();
        for spec in OVERRIDES {
            if super::status(name, spec) == super::OverrideStatus::Custom {
                continue;
            }
            let staged = match spec.seed {
                Seed::Template(_) => fs::read_to_string(current.join(spec.file)).ok(),
                Seed::ShellSection(section) => fs::read_to_string(current.join("shell.toml"))
                    .ok()
                    .and_then(|shell| shell_section(&shell, section)),
                Seed::Fixed(_) => None,
            };
            let Some(staged) = staged else {
                continue;
            };
            assert_eq!(
                super::generated(name, spec).unwrap(),
                staged,
                "{}",
                spec.file
            );
        }
    }

    #[test]
    fn every_installed_shell_section_has_an_override() {
        let Ok(shell) = fs::read_to_string(themed_templates_dir().join("shell.toml.tpl")) else {
            return;
        };
        for section in shell_sections(&shell) {
            assert!(
                OVERRIDES
                    .iter()
                    .any(|spec| spec.shell_section() == Some(section)),
                "no override for [{section}]"
            );
        }
    }
}
