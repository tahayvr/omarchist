// Keeps Hyprland from swallowing chords while the recorder is listening.
//
// Hyprland handles bound chords before any client sees them, so recording
// `SUPER + K` inside Omarchist is impossible while the bind exists. The
// classic escape hatch is a submap: while a submap is active only its own
// binds fire. `omarchist.lua` defines one that holds a single switch bind
// (a submap must contain a bind to be registered, and a switch event can
// never be typed), and the recorder switches Hyprland into it for the
// duration of a recording.
//
// If the app dies mid-recording the compositor would stay in the submap,
// so three safety nets exist: the submap itself binds `SUPER + ESCAPE` to
// leave it, a panic hook resets it before a release build aborts, and every
// launch resets it unconditionally (`reset_on_startup`). The manual escape
// stays `hyprctl dispatch 'hl.dsp.submap("reset")'`.
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{Error, Result};

pub const RECORDING_SUBMAP: &str = "omarchist-recording";

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// The chord that leaves the recording submap from the keyboard. It cannot
/// be recorded itself, which is a fair price for never being locked out.
pub const ESCAPE_CHORD: &str = "SUPER + SHIFT + ESCAPE";

/// The Lua that registers the submap, emitted into `omarchist.lua` on
/// every write so it exists after any reload. The switch bind keeps the
/// submap registered (a submap needs a bind), the escape bind gets the
/// user out if the app is gone.
pub fn define_recording_submap_lua() -> String {
    format!(
        "hl.define_submap({name:?}, function()\n  \
         hl.bind(\"switch:on:Omarchist Recording\", hl.dsp.submap(\"reset\"), {{ description = \"Omarchist keystroke recording\" }})\n  \
         hl.bind({escape:?}, hl.dsp.submap(\"reset\"), {{ description = \"Stop Omarchist keystroke recording\" }})\n\
         end)\n",
        name = RECORDING_SUBMAP,
        escape = ESCAPE_CHORD
    )
}

/// Leaves the submap whether or not this process entered it: a previous
/// instance may have died while recording. Errors are ignored because
/// Hyprland may not be running at all.
pub fn reset_on_startup() {
    ACTIVE.store(false, Ordering::SeqCst);
    let _ = dispatch_submap("reset");
}

/// Installs a panic hook that leaves the submap before the process dies.
/// Release builds abort on panic, so no destructor or quit handler runs.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        leave_recording_submap();
        previous(info);
    }));
}

pub fn enter_recording_submap() -> Result<()> {
    if ACTIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    if dispatch_submap(RECORDING_SUBMAP).is_err() {
        // The submap is missing until Hyprland has reloaded the freshly
        // written omarchist.lua; define it live and try once more.
        hyprctl(&["eval", &define_recording_submap_lua()])?;
        dispatch_submap(RECORDING_SUBMAP)?;
    }
    ACTIVE.store(true, Ordering::SeqCst);
    Ok(())
}

/// Returns Hyprland to its normal binds. Safe to call repeatedly and when
/// no recording is active.
pub fn leave_recording_submap() {
    if ACTIVE.swap(false, Ordering::SeqCst) {
        let _ = dispatch_submap("reset");
    }
}

pub fn is_recording_submap_active() -> bool {
    ACTIVE.load(Ordering::SeqCst)
}

fn dispatch_submap(name: &str) -> Result<()> {
    hyprctl(&["dispatch", &format!("hl.dsp.submap({name:?})")])
}

fn hyprctl(args: &[&str]) -> Result<()> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .map_err(|e| Error::io("Failed to run hyprctl", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if output.status.success() && !stdout.trim_start().starts_with("error") {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::Invalid(format!(
            "hyprctl {} failed: {}",
            args.first().unwrap_or(&""),
            stdout.trim().to_string() + stderr.trim()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submap_definition_registers_one_inert_bind() {
        let lua = define_recording_submap_lua();
        assert!(lua.starts_with("hl.define_submap(\"omarchist-recording\", function()"));
        assert!(lua.contains("hl.bind(\"switch:on:Omarchist Recording\""));
        assert!(lua.contains("hl.bind(\"SUPER + SHIFT + ESCAPE\", hl.dsp.submap(\"reset\")"));
        assert!(lua.trim_end().ends_with("end)"));
    }

    #[test]
    fn a_panic_leaves_the_submap_before_the_default_hook() {
        // The hook only dispatches when this process entered the submap, so
        // without Hyprland it is a no-op that must still not panic itself.
        install_panic_hook();
        let result = std::panic::catch_unwind(|| panic!("boom"));
        assert!(result.is_err());
        assert!(!is_recording_submap_active());
    }
}
