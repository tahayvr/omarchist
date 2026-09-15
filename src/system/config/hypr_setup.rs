use crate::error::{Error, Result};
use std::fs;
use std::path::PathBuf;

use crate::system::hyprland_config::manager;
use crate::system::omarchy_paths::user_hyprland_config_dir;

const SOURCE_COMMENT: &str = "-- Added by Omarchist";
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
    let hypr_config_path = get_hypr_config_path()?;
    ensure_omarchist_lua_stub()?;

    if !hypr_config_path.exists() {
        return Err(Error::Invalid(format!(
            "Hyprland config not found at: {}",
            hypr_config_path.display()
        )));
    }

    let content = fs::read_to_string(&hypr_config_path)
        .map_err(|e| Error::io("Failed to read hyprland.lua", e))?;

    if content.contains(REQUIRE_DIRECTIVE) {
        return Ok(false);
    }

    let new_content = if let Some(pos) = content.find("require(\"hypr.autostart\")") {
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
    } else {
        format!(
            "{}\n\n{}\n{}\n",
            content.trim_end(),
            SOURCE_COMMENT,
            REQUIRE_DIRECTIVE
        )
    };

    fs::write(&hypr_config_path, new_content)
        .map_err(|e| Error::io("Failed to write hyprland.lua", e))?;

    println!("Added omarchist require directive to hyprland.lua");

    Ok(true)
}

// Guarantees `~/.config/hypr/omarchist.lua` exists so `require("hypr.omarchist")`
// never dangles, even before the user has saved any Hyprland setting, and
// that it carries the current settings, keybind overrides and recording
// submap (the file is regenerated only when its content would change).
fn ensure_omarchist_lua_stub() -> Result<()> {
    let dir = user_hyprland_config_dir().ok_or(Error::UnknownDirectory("home"))?;
    if !dir.exists() {
        return Err(Error::UnknownDirectory("Hyprland config"));
    }
    manager::write_omarchist_lua(&manager::saved_config())
}

fn get_hypr_config_path() -> Result<PathBuf> {
    let dir = user_hyprland_config_dir().ok_or(Error::UnknownDirectory("home"))?;
    Ok(dir.join("hyprland.lua"))
}
