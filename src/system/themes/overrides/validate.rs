use std::io::{ErrorKind, Write};
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

use super::registry::{Format, OverrideSpec};

/// Checks that Omarchy and the consuming app can use `content`. File-specific
/// rules mirror what the matching `omarchy-theme-set-*` script accepts, since
/// it silently skips anything else.
pub fn validate(spec: &OverrideSpec, content: &str) -> Result<()> {
    match spec.file {
        "vscode.json" => vscode_descriptor(content)?,
        "hermes.yaml" => hermes_skin(content)?,
        "chromium.theme" => rgb_triplet(content)?,
        "keyboard.rgb" => keyboard_hex(content)?,
        "icons.theme" => icon_theme(content)?,
        "t3code.json" if content.contains("{{") => {
            return Err(invalid(spec, "it still contains an unresolved {{ token }}"));
        }
        _ => {}
    }

    match spec.format {
        Format::Json => serde_json::from_str::<serde_json::Value>(content)
            .map(|_| ())
            .map_err(|e| invalid(spec, e)),
        Format::Toml => toml::from_str::<toml::Table>(content)
            .map(|_| ())
            .map_err(|e| invalid(spec, e.message())),
        Format::Lua => lua_syntax(content).map_err(|e| invalid(spec, e)),
        Format::Yaml | Format::Css | Format::Ini | Format::Plain => Ok(()),
    }
}

fn invalid(spec: &OverrideSpec, reason: impl std::fmt::Display) -> Error {
    Error::Invalid(format!("{} is not valid: {reason}", spec.file))
}

// `luac -p` parses without running. Without `luac` there is nothing to check
// against, so the content is accepted.
fn lua_syntax(content: &str) -> std::result::Result<(), String> {
    let mut child = match Command::new("luac")
        .args(["-p", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.to_string()),
    };
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = stderr.trim();
    Err(message
        .strip_prefix("luac: ")
        .unwrap_or(message)
        .replacen("stdin:", "line ", 1))
}

fn vscode_descriptor(content: &str) -> Result<()> {
    let value: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| Error::Invalid(format!("vscode.json is not valid: {e}")))?;
    let field = |key: &str| value.get(key).and_then(|v| v.as_str()).unwrap_or_default();

    let name = field("name");
    if name.is_empty()
        || name
            .chars()
            .any(|c| c.is_control() || c == '"' || c == '\\')
    {
        return Err(Error::Invalid(
            "The VS Code theme name must be set and cannot contain quotes or backslashes".into(),
        ));
    }
    let extension = field("extension");
    if extension.is_empty()
        || !extension
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(Error::Invalid(
            "The extension must be a Marketplace id such as publisher.theme-name".into(),
        ));
    }
    Ok(())
}

// `omarchy-theme-set-hermes` publishes only a skin of this exact shape: the
// name `omarchy`, an optional plain description, then `colors:` followed by
// `  key: "#rrggbb"` lines, all printable ASCII.
fn hermes_skin(content: &str) -> Result<()> {
    let fail = |reason: String| {
        Err(Error::Invalid(format!(
            "hermes.yaml is not valid: {reason}"
        )))
    };

    if let Some(c) = content
        .chars()
        .find(|&c| c != '\n' && !(' '..='~').contains(&c))
    {
        return fail(format!("it contains the character {c:?}"));
    }

    let mut seen_name = false;
    let mut seen_description = false;
    let mut seen_colors = false;
    let mut colors = 0;

    for (index, line) in content.lines().enumerate() {
        let number = index + 1;
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !seen_name {
            if line != "name: omarchy" {
                return fail(format!("line {number} must be `name: omarchy`"));
            }
            seen_name = true;
        } else if !seen_colors {
            if line == "colors:" {
                seen_colors = true;
            } else if !seen_description && is_plain_description(line) {
                seen_description = true;
            } else {
                return fail(format!(
                    "line {number} must be `description: ...` or `colors:`"
                ));
            }
        } else if is_skin_color(line) {
            colors += 1;
        } else {
            return fail(format!(
                "line {number} must look like `  ui_text: \"#rrggbb\"`"
            ));
        }
    }

    if colors == 0 {
        return fail("it needs a `colors:` block with at least one color".into());
    }
    Ok(())
}

fn is_plain_description(line: &str) -> bool {
    line.strip_prefix("description: ").is_some_and(|text| {
        text.len() <= 200
            && text
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || " ,.()-".contains(c))
    })
}

fn is_skin_color(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("  ") else {
        return false;
    };
    let Some((key, value)) = rest.split_once(": ") else {
        return false;
    };
    (1..=64).contains(&key.len())
        && key.chars().all(|c| c.is_ascii_lowercase() || c == '_')
        && value.len() == 9
        && value.starts_with("\"#")
        && value.ends_with('"')
        && value[2..8].chars().all(|c| c.is_ascii_hexdigit())
}

fn rgb_triplet(content: &str) -> Result<()> {
    let parts: Vec<&str> = content.trim().split(',').map(str::trim).collect();
    let valid = parts.len() == 3
        && parts
            .iter()
            .all(|p| (1..=3).contains(&p.len()) && p.parse::<u16>().is_ok_and(|n| n < 256));
    if valid {
        Ok(())
    } else {
        Err(Error::Invalid(
            "chromium.theme must be a color as R,G,B, for example 26,27,38".into(),
        ))
    }
}

fn keyboard_hex(content: &str) -> Result<()> {
    let hex = content.trim();
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(Error::Invalid(
            "keyboard.rgb must be a hex color such as #7aa2f7".into(),
        ))
    }
}

fn icon_theme(content: &str) -> Result<()> {
    let name = content.trim();
    if name.is_empty() || name.contains(['\n', '/']) {
        Err(Error::Invalid(
            "icons.theme must be a single icon theme name such as Yaru-blue".into(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::validate;
    use crate::system::themes::overrides::registry::find;

    fn check(file: &str, content: &str) -> bool {
        validate(find(file).unwrap(), content).is_ok()
    }

    #[test]
    fn json_and_toml() {
        assert!(check("claude.json", "{\"name\": \"Omarchy\"}"));
        assert!(!check("claude.json", "{\"name\": }"));
        assert!(check("helix.toml", "[palette]\nred = \"#ff0000\"\n"));
        assert!(!check("helix.toml", "[palette\n"));
        assert!(check("shell.lock.toml", "[lock]\ntext = \"#ffffff\"\n"));
    }

    #[test]
    fn lua_is_parsed_when_luac_exists() {
        assert!(check(
            "neovim.lua",
            "return { { \"folke/tokyonight.nvim\" } }\n"
        ));
        if std::process::Command::new("luac")
            .arg("-v")
            .output()
            .is_ok()
        {
            assert!(!check("neovim.lua", "return {\n"));
        }
    }

    #[test]
    fn vscode_descriptor() {
        assert!(check(
            "vscode.json",
            r#"{"name": "Lumon", "extension": "oldjobobo.lumon-theme"}"#
        ));
        assert!(!check(
            "vscode.json",
            r#"{"name": "Lu\"mon", "extension": "a.b"}"#
        ));
        assert!(!check(
            "vscode.json",
            r#"{"name": "Lumon", "extension": "a b; rm"}"#
        ));
        assert!(!check("vscode.json", r#"{"name": "", "extension": "a.b"}"#));
    }

    #[test]
    fn hermes_skin() {
        let good = "name: omarchy\ndescription: Omarchy system theme\ncolors:\n  background: \"#1a1b26\"\n";
        assert!(check("hermes.yaml", good));
        assert!(!check(
            "hermes.yaml",
            "name: other\ncolors:\n  a: \"#000000\"\n"
        ));
        assert!(!check("hermes.yaml", "name: omarchy\ncolors:\n"));
        assert!(!check("hermes.yaml", "name: omarchy\ncolors:\n  a: red\n"));
        assert!(!check(
            "hermes.yaml",
            "name: omarchy\ncolors:\n  a: \"#00000é\"\n"
        ));
    }

    #[test]
    fn single_value_files() {
        assert!(check("chromium.theme", "26,27,38\n"));
        assert!(!check("chromium.theme", "256,0,0"));
        assert!(!check("chromium.theme", "#1a1b26"));
        assert!(check("keyboard.rgb", "#7aa2f7\n"));
        assert!(check("keyboard.rgb", "7aa2f7"));
        assert!(!check("keyboard.rgb", "blue"));
        assert!(check("icons.theme", "Yaru-blue\n"));
        assert!(!check("icons.theme", "\n"));
        assert!(!check("t3code.json", "{\"accent\": \"{{ accent }}\"}"));
    }
}
