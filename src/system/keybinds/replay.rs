// Turns the scanner's ordered bind/unbind events into the effective keybind
// list, applying Hyprland's semantics: binds on one chord stack (all fire),
// and `hl.unbind(chord)` removes every bind registered on that chord so far.
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::system::omarchy_paths::{omarchy_install_dir, user_hyprland_config_dir};

use super::chord::Chord;
use super::scanner::{ScanEvent, run_scan};
use super::{BindStatus, Keybind, Origin};

const HYPRLAND_LUA: &str = "hyprland.lua";
const OMARCHIST_LUA: &str = "omarchist.lua";

/// Paths used to classify where a bind came from.
#[derive(Debug, Clone)]
pub struct ScanPaths {
    pub omarchy_root: PathBuf,
    pub omarchist_lua: Option<PathBuf>,
}

impl ScanPaths {
    pub fn from_environment() -> Self {
        Self {
            omarchy_root: omarchy_install_dir(),
            omarchist_lua: user_hyprland_config_dir().map(|d| d.join(OMARCHIST_LUA)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub binds: Vec<Keybind>,
    /// Non-fatal problems (unparsable keys strings, an evaluation error).
    pub warnings: Vec<String>,
    /// False when the config raised an error part-way; the list is then
    /// whatever was registered before the failure.
    pub complete: bool,
}

pub fn classify_origin(source: &Path, paths: &ScanPaths) -> Origin {
    if source.starts_with(paths.omarchy_root.join("default")) {
        return Origin::Default;
    }
    if paths.omarchist_lua.as_deref() == Some(source)
        || source.file_name().is_some_and(|name| name == OMARCHIST_LUA)
    {
        return Origin::Omarchist;
    }
    Origin::User
}

pub fn replay(events: &[ScanEvent], paths: &ScanPaths) -> ScanResult {
    let mut result = ScanResult::default();

    for event in events {
        match event {
            ScanEvent::Bind(scanned) => match Chord::parse(&scanned.keys) {
                Ok(chord) => result.binds.push(Keybind {
                    seq: scanned.seq,
                    chord,
                    keys_raw: scanned.keys.clone(),
                    description: scanned.description.clone(),
                    dispatcher: scanned.dispatcher.clone(),
                    options: scanned.options,
                    origin: classify_origin(&scanned.source, paths),
                    source: scanned.source.clone(),
                    status: BindStatus::Active,
                }),
                Err(e) => result.warnings.push(format!(
                    "{} ({}): {e}",
                    scanned.source.display(),
                    scanned.keys
                )),
            },
            ScanEvent::Unbind { seq, source, keys } => match Chord::parse(keys) {
                Ok(chord) => {
                    let by_origin = classify_origin(source, paths);
                    for bind in result.binds.iter_mut().filter(|b| b.is_active()) {
                        if bind.chord.same_as(&chord) {
                            bind.status = BindStatus::Unbound {
                                by_seq: *seq,
                                by_origin,
                            };
                        }
                    }
                }
                Err(e) => result
                    .warnings
                    .push(format!("{} (unbind {keys}): {e}", source.display())),
            },
            ScanEvent::Error { message, .. } => {
                result
                    .warnings
                    .push(format!("hyprland.lua failed to evaluate: {message}"));
            }
            ScanEvent::Done { .. } => {
                result.complete = !result
                    .warnings
                    .iter()
                    .any(|w| w.starts_with("hyprland.lua failed"));
            }
        }
    }

    result
}

/// Scans the user's `~/.config/hypr/hyprland.lua` and replays it.
pub fn scan_keybinds() -> Result<ScanResult> {
    let config = user_hyprland_config_dir()
        .ok_or(crate::error::Error::UnknownDirectory("Hyprland config"))?
        .join(HYPRLAND_LUA);
    let events = run_scan(&config)?;
    Ok(replay(&events, &ScanPaths::from_environment()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::keybinds::scanner::ScannedBind;
    use crate::system::keybinds::{BindOptions, Dispatcher};

    fn paths() -> ScanPaths {
        ScanPaths {
            omarchy_root: PathBuf::from("/usr/share/omarchy"),
            omarchist_lua: Some(PathBuf::from("/home/u/.config/hypr/omarchist.lua")),
        }
    }

    fn bind(seq: u32, source: &str, keys: &str, description: &str) -> ScanEvent {
        ScanEvent::Bind(ScannedBind {
            seq,
            source: PathBuf::from(source),
            keys: keys.into(),
            description: description.into(),
            dispatcher: Dispatcher::Exec(format!("cmd-{seq}")),
            options: BindOptions::default(),
        })
    }

    fn unbind(seq: u32, source: &str, keys: &str) -> ScanEvent {
        ScanEvent::Unbind {
            seq,
            source: PathBuf::from(source),
            keys: keys.into(),
        }
    }

    #[test]
    fn classifies_origins_by_path() {
        let p = paths();
        assert_eq!(
            classify_origin(
                Path::new("/usr/share/omarchy/default/hypr/bindings/tiling.lua"),
                &p
            ),
            Origin::Default
        );
        assert_eq!(
            classify_origin(Path::new("/home/u/.config/hypr/omarchist.lua"), &p),
            Origin::Omarchist
        );
        assert_eq!(
            classify_origin(Path::new("/home/u/.config/hypr/bindings.lua"), &p),
            Origin::User
        );
        assert_eq!(
            classify_origin(Path::new("/home/u/.config/hypr/hyprland.lua"), &p),
            Origin::User
        );
    }

    #[test]
    fn unbind_removes_earlier_binds_on_the_chord_only() {
        let default = "/usr/share/omarchy/default/hypr/bindings/x.lua";
        let user = "/home/u/.config/hypr/bindings.lua";
        let events = vec![
            bind(1, default, "SUPER + SHIFT + C", "Calendar"),
            bind(2, default, "SUPER + shift + c", "Calendar (release)"),
            bind(3, default, "SUPER + C", "Copy"),
            unbind(4, user, "super + SHIFT + C"),
            bind(5, user, "SUPER + SHIFT + C", "VS Code"),
            ScanEvent::Done { seq: 5 },
        ];
        let result = replay(&events, &paths());
        assert!(result.complete);
        assert!(result.warnings.is_empty());
        assert_eq!(result.binds.len(), 4);
        assert_eq!(
            result.binds[0].status,
            BindStatus::Unbound {
                by_seq: 4,
                by_origin: Origin::User
            }
        );
        assert_eq!(result.binds[1].status, result.binds[0].status);
        assert!(result.binds[2].is_active(), "other chords untouched");
        assert!(result.binds[3].is_active(), "the later rebind stays active");
        assert_eq!(result.binds[3].origin, Origin::User);
        assert_eq!(result.binds[0].origin, Origin::Default);
    }

    #[test]
    fn keycode_and_keysym_chords_match_for_unbind() {
        let default = "/usr/share/omarchy/default/hypr/bindings/tiling.lua";
        let events = vec![
            bind(1, default, "SUPER + code:10", "Workspace 1"),
            unbind(2, "/home/u/.config/hypr/omarchist.lua", "SUPER + 1"),
        ];
        let result = replay(&events, &paths());
        assert!(matches!(
            result.binds[0].status,
            BindStatus::Unbound {
                by_origin: Origin::Omarchist,
                ..
            }
        ));
    }

    #[test]
    fn bad_keys_and_errors_become_warnings() {
        let events = vec![
            bind(1, "/x.lua", "SUPER + A + B", "Broken"),
            ScanEvent::Error {
                seq: 1,
                message: "attempt to index nil".into(),
            },
            ScanEvent::Done { seq: 1 },
        ];
        let result = replay(&events, &paths());
        assert!(result.binds.is_empty());
        assert_eq!(result.warnings.len(), 2);
        assert!(!result.complete);
    }
}
