use crate::error::{Error, Result};
use std::fs;
use std::path::PathBuf;

use crate::system::fs::write_atomic;
use crate::system::hyprland_config::manager;
use crate::system::omarchy_paths::user_hyprland_config_dir;

const SOURCE_COMMENT: &str = "-- Added by Omarchist";
/// Shown when a save had to put the require line back.
pub const HOOK_RESTORED_MESSAGE: &str =
    "hyprland.lua no longer loaded Omarchist's settings; the require line was added back";
const REQUIRE_DIRECTIVE: &str = "require(\"hypr.omarchist\")";

// Adds `require("hypr.omarchist")` to the user's hyprland.lua, idempotently.
// Confirmed against the real omacom/omarchy@quattro source
// (`config/hypr/hyprland.lua`): Omarchy's own convention for user config
// modules is a `require("hypr.<name>")` line resolving to
// `~/.config/hypr/<name>.lua` (package.path is set up by Omarchy's own
// bootstrap), loaded after `require("hypr.autostart")` — inserting after
// that line keeps Omarchist's settings taking precedence over Omarchy's
// defaults, matching the equivalent behavior under the old hyprlang config.
//
// Unlike hyprlang's `source = ~/.config/omarchist/hyprland/*` (a glob that
// silently matches zero files), Lua's `require()` hard-errors if the target
// module doesn't exist. This runs on every app startup, before the user has
// necessarily ever saved a Hyprland setting (and hence before
// `HyprlandConfigManager` has ever written `omarchist.lua`) — so a stub file
// must always exist here, or the very first launch leaves the user's
// compositor config broken (confirmed via a live smoke test against a real
// Quattro install: `hyprctl configerrors` reported exactly this "module not
// found" error immediately after adding the require line alone).
pub fn ensure_hypr_source() -> Result<bool> {
    // Writing the stub also re-checks the require line.
    ensure_omarchist_lua_stub()
}

/// Adds the `require("hypr.omarchist")` line to `hyprland.lua` when it is
/// missing. `Ok(true)` when it had to be added: at first launch, or after
/// something replaced the user's file (`omarchy-refresh-hyprland`, a reset
/// from the Configuration page, a hand edit), which would otherwise leave
/// every Omarchist setting and keybind silently inert.
pub fn ensure_require_line() -> Result<bool> {
    let hypr_config_path = get_hypr_config_path()?;

    if !hypr_config_path.exists() {
        return Err(Error::Invalid(format!(
            "Hyprland config not found at: {}",
            hypr_config_path.display()
        )));
    }

    let content = fs::read_to_string(&hypr_config_path)
        .map_err(|e| Error::io("Failed to read hyprland.lua", e))?;

    let Some(new_content) = with_require_line(&content) else {
        return Ok(false);
    };

    write_atomic(&hypr_config_path, new_content, "hyprland.lua")?;

    Ok(true)
}

/// Whether `hyprland.lua` loads Omarchist.
pub fn has_require_line(content: &str) -> bool {
    content.contains(REQUIRE_DIRECTIVE)
}

/// `content` without the require line and the comment above it, for the
/// uninstall. Only whole lines that are exactly ours go; a require the
/// user wrote inside other code stays.
pub fn without_require_line(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut pending_comment: Option<&str> = None;
    for line in content.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed == SOURCE_COMMENT {
            // Only dropped when our require follows it.
            if let Some(comment) = pending_comment.take() {
                out.push_str(comment);
            }
            pending_comment = Some(line);
            continue;
        }
        if trimmed == REQUIRE_DIRECTIVE {
            pending_comment = None;
            continue;
        }
        if let Some(comment) = pending_comment.take() {
            out.push_str(comment);
        }
        out.push_str(line);
    }
    if let Some(comment) = pending_comment {
        out.push_str(comment);
    }
    out
}

/// `content` with the require line added after `require("hypr.autostart")`
/// (or at the end), or `None` when it is already there.
fn with_require_line(content: &str) -> Option<String> {
    if content.contains(REQUIRE_DIRECTIVE) {
        return None;
    }
    Some(
        if let Some(pos) = content.find("require(\"hypr.autostart\")") {
            let insert_at = content[pos..]
                .find('\n')
                .map(|offset| pos + offset + 1)
                .unwrap_or(content.len());
            format!(
                "{}{}\n{}\n{}",
                &content[..insert_at],
                SOURCE_COMMENT,
                REQUIRE_DIRECTIVE,
                &content[insert_at..]
            )
        } else if let Some(pos) = content.find("require(\"default.hypr.toggles\")") {
            // Omarchy's toggles must load after Omarchist so they keep
            // winning (No Gaps over a gaps override).
            let line_start = content[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            format!(
                "{}{}\n{}\n{}",
                &content[..line_start],
                SOURCE_COMMENT,
                REQUIRE_DIRECTIVE,
                &content[line_start..]
            )
        } else {
            format!(
                "{}\n\n{}\n{}\n",
                content.trim_end(),
                SOURCE_COMMENT,
                REQUIRE_DIRECTIVE
            )
        },
    )
}

// Guarantees `~/.config/hypr/omarchist.lua` exists so `require("hypr.omarchist")`
// never dangles, even before the user has saved any Hyprland setting, and
// that it carries the current settings, keybind overrides and recording
// submap (the file is regenerated only when its content would change).
fn ensure_omarchist_lua_stub() -> Result<bool> {
    let dir = user_hyprland_config_dir().ok_or(Error::UnknownDirectory("home"))?;
    if !dir.exists() {
        return Err(Error::UnknownDirectory("Hyprland config"));
    }
    manager::write_omarchist_lua(&manager::saved_overrides())
}

/// The user's `~/.config/hypr/hyprland.lua`.
pub fn hypr_config_path() -> Result<PathBuf> {
    get_hypr_config_path()
}

/// Waits for `hyprland.lua` to change (a reset or migration run in a
/// terminal) and then makes sure it still requires Omarchist, polling
/// every two seconds for at most `timeout`. Blocking: run it off the UI
/// thread. `true` when the require line had to be put back.
pub fn restore_require_line_after_change(timeout: std::time::Duration) -> bool {
    let Ok(path) = get_hypr_config_path() else {
        return false;
    };
    let modified = |path: &std::path::Path| fs::metadata(path).and_then(|m| m.modified()).ok();
    let before = modified(&path);
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_secs(2));
        if modified(&path) != before {
            return ensure_require_line().unwrap_or(false);
        }
    }
    false
}

fn get_hypr_config_path() -> Result<PathBuf> {
    let dir = user_hyprland_config_dir().ok_or(Error::UnknownDirectory("home"))?;
    Ok(dir.join("hyprland.lua"))
}

#[cfg(test)]
mod tests {
    use super::{REQUIRE_DIRECTIVE, with_require_line, without_require_line};

    #[test]
    fn the_uninstall_takes_out_only_the_lines_it_added() {
        let omarchy = "require(\"hypr.bindings\")\nrequire(\"hypr.autostart\")\n\nrequire(\"default.hypr.toggles\")\n";
        let added = with_require_line(omarchy).expect("added");
        assert_eq!(without_require_line(&added), omarchy);
        // A comment that is not followed by our require stays, as does a
        // require inside other code.
        let hand =
            "-- Added by Omarchist\nlocal x = 1\nif x then require(\"hypr.omarchist\") end\n";
        assert_eq!(without_require_line(hand), hand);
        assert_eq!(without_require_line(omarchy), omarchy);
    }

    #[test]
    fn the_line_goes_after_autostart_and_only_once() {
        let omarchy = "require(\"hypr.bindings\")\nrequire(\"hypr.autostart\")\n\nrequire(\"default.hypr.toggles\")\n";
        let added = with_require_line(omarchy).expect("added");
        assert_eq!(
            added,
            "require(\"hypr.bindings\")\nrequire(\"hypr.autostart\")\n-- Added by Omarchist\nrequire(\"hypr.omarchist\")\n\nrequire(\"default.hypr.toggles\")\n"
        );
        assert!(with_require_line(&added).is_none(), "idempotent");
    }

    #[test]
    fn a_file_without_autostart_gets_the_line_at_the_end() {
        let added = with_require_line("require(\"hypr.input\")").expect("added");
        assert!(added.ends_with(&format!("\n\n-- Added by Omarchist\n{REQUIRE_DIRECTIVE}\n")));
    }

    #[test]
    fn without_autostart_the_line_still_precedes_omarchys_toggles() {
        let added =
            with_require_line("require(\"hypr.input\")\nrequire(\"default.hypr.toggles\")\n")
                .expect("added");
        assert_eq!(
            added,
            "require(\"hypr.input\")\n-- Added by Omarchist\nrequire(\"hypr.omarchist\")\nrequire(\"default.hypr.toggles\")\n"
        );
    }

    #[test]
    fn omarchys_reset_copy_is_recognised_as_missing_the_line() {
        // What `omarchy-refresh-hyprland` copies over the user's file.
        let reset = "require(\"default.hypr.omarchy\")\nrequire(\"hypr.autostart\")\nrequire(\"default.hypr.toggles\")\n";
        assert!(with_require_line(reset).is_some());
    }
}
