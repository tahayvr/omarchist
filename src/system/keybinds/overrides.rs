// User changes to keybinds, stored in `~/.config/omarchist/hyprland/keybinds.json`
// and rendered as Lua into `~/.config/hypr/omarchist.lua`.
//
// Each override targets a scanned bind by identity (chord + description +
// dispatcher). Because `hl.unbind(chord)` removes *every* bind on the chord,
// an override records the sibling binds to re-register (`restore`) at save
// time, so rendering is a pure function of this file and never needs a scan.
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::system::hyprland_config::lua_writer::lua_string;

use super::chord::Chord;
use super::{BindIdentity, BindOptions, Dispatcher, Keybind, Origin};

pub const OVERRIDES_VERSION: u32 = 1;

/// One `hl.bind` call to emit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindSpec {
    pub keys: String,
    #[serde(default)]
    pub description: String,
    pub dispatcher: Dispatcher,
    #[serde(default)]
    pub options: BindOptions,
}

impl BindSpec {
    pub fn from_keybind(bind: &Keybind) -> Self {
        Self {
            keys: bind.chord.to_omarchy_string(),
            description: bind.description.clone(),
            dispatcher: bind.dispatcher.clone(),
            options: bind.options,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Override {
    /// Replace `target`'s chord (and optionally description/command).
    Rebind {
        target: BindIdentity,
        bind: BindSpec,
        #[serde(default)]
        restore: Vec<BindSpec>,
    },
    /// Remove `target` without a replacement.
    Disable {
        target: BindIdentity,
        #[serde(default)]
        restore: Vec<BindSpec>,
    },
    /// A brand-new bind.
    Add { bind: BindSpec },
}

impl Override {
    pub fn target(&self) -> Option<&BindIdentity> {
        match self {
            Override::Rebind { target, .. } | Override::Disable { target, .. } => Some(target),
            Override::Add { .. } => None,
        }
    }

    pub fn bind(&self) -> Option<&BindSpec> {
        match self {
            Override::Rebind { bind, .. } | Override::Add { bind } => Some(bind),
            Override::Disable { .. } => None,
        }
    }

    fn restore(&self) -> &[BindSpec] {
        match self {
            Override::Rebind { restore, .. } | Override::Disable { restore, .. } => restore,
            Override::Add { .. } => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeybindOverrides {
    pub version: u32,
    #[serde(default)]
    pub overrides: Vec<Override>,
}

impl Default for KeybindOverrides {
    fn default() -> Self {
        Self {
            version: OVERRIDES_VERSION,
            overrides: Vec::new(),
        }
    }
}

impl KeybindOverrides {
    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }

    pub fn find_for_target(&self, identity: &BindIdentity) -> Option<usize> {
        self.overrides
            .iter()
            .position(|o| o.target() == Some(identity))
    }

    /// Adds `override_`, replacing any existing override on the same target.
    pub fn upsert(&mut self, override_: Override) {
        if let Some(target) = override_.target()
            && let Some(ix) = self.find_for_target(target)
        {
            self.overrides[ix] = override_;
            return;
        }
        self.overrides.push(override_);
    }

    pub fn remove(&mut self, ix: usize) -> Option<Override> {
        (ix < self.overrides.len()).then(|| self.overrides.remove(ix))
    }

    /// Rejects anything that would not round-trip into safe Lua: unparsable
    /// chords, and Lua dispatchers that are not a single `hl.dsp.*(...)`
    /// call (a hand-edited json must not be able to inject arbitrary code
    /// into the compositor).
    pub fn validate(&self) -> Result<()> {
        for override_ in &self.overrides {
            if let Some(target) = override_.target() {
                Chord::parse(&target.keys)?;
            }
            let specs = override_.bind().into_iter().chain(override_.restore());
            for spec in specs {
                Chord::parse(&spec.keys)?;
                validate_dispatcher(&spec.dispatcher)?;
            }
        }
        Ok(())
    }
}

fn validate_dispatcher(dispatcher: &Dispatcher) -> Result<()> {
    match dispatcher {
        Dispatcher::Exec(_) => Ok(()),
        Dispatcher::Function => Err(Error::Invalid(
            "A Lua function bind cannot be re-emitted".to_string(),
        )),
        Dispatcher::Lua(expr) => {
            if is_dsp_call(expr) {
                Ok(())
            } else {
                Err(Error::Invalid(format!("Unsupported dispatcher '{expr}'")))
            }
        }
    }
}

/// `hl.dsp.<path>(<args>)` on one line, with balanced parentheses and no
/// statement separators, i.e. exactly the shape the scanner reconstructs.
fn is_dsp_call(expr: &str) -> bool {
    let Some(rest) = expr.strip_prefix("hl.dsp.") else {
        return false;
    };
    let Some(open) = rest.find('(') else {
        return false;
    };
    let path = &rest[..open];
    let args = &rest[open..];
    !path.is_empty()
        && path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        && !path.starts_with('.')
        && !path.ends_with('.')
        && args.ends_with(')')
        && !args.contains(['\n', '\r', ';'])
        && parens_balanced(args)
}

fn parens_balanced(text: &str) -> bool {
    let mut depth = 0i32;
    let mut in_string: Option<char> = None;
    let mut escaped = false;
    for c in text.chars() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                in_string = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => in_string = Some(c),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0 && in_string.is_none()
}

/// The other binds on `target`'s chord that `hl.unbind` will remove and that
/// can be re-registered: active, rebindable, and not already Omarchist's own
/// (those are re-emitted from their own overrides).
pub fn restore_specs(target: &Keybind, all: &[Keybind]) -> Vec<BindSpec> {
    let identity = target.identity();
    all.iter()
        .filter(|b| b.is_active() && b.is_rebindable() && b.origin != Origin::Omarchist)
        .filter(|b| b.chord.same_as(&target.chord) && b.identity() != identity)
        .map(BindSpec::from_keybind)
        .collect()
}

/// Sibling binds that are lost for good when `target` is unbound: Lua
/// function dispatchers cannot be re-emitted.
pub fn unrestorable_siblings<'a>(target: &Keybind, all: &'a [Keybind]) -> Vec<&'a Keybind> {
    let identity = target.identity();
    all.iter()
        .filter(|b| b.is_active() && !b.is_rebindable())
        .filter(|b| b.chord.same_as(&target.chord) && b.identity() != identity)
        .collect()
}

/// Renders the Lua block for `omarchist.lua`. Empty when there is nothing
/// to emit. Order: every `hl.unbind` first, then restored siblings, then
/// rebinds, then additions, so the result is independent of override order.
pub fn emit_keybinds_lua(overrides: &KeybindOverrides) -> String {
    let unbinds: BTreeSet<String> = overrides
        .overrides
        .iter()
        .filter_map(Override::target)
        .filter_map(|t| Chord::parse(&t.keys).ok())
        .map(|c| c.to_omarchy_string())
        .collect();

    let mut lines = Vec::new();
    for keys in &unbinds {
        lines.push(format!("hl.unbind({})", lua_string(keys)));
    }
    for spec in overrides.overrides.iter().flat_map(Override::restore) {
        lines.push(bind_call(spec));
    }
    for override_ in &overrides.overrides {
        if let Override::Rebind { bind, .. } = override_ {
            lines.push(bind_call(bind));
        }
    }
    for override_ in &overrides.overrides {
        if let Override::Add { bind } = override_ {
            lines.push(bind_call(bind));
        }
    }

    if lines.is_empty() {
        return String::new();
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// One `hl.bind(keys, dispatcher, { options })` line.
pub fn bind_call(spec: &BindSpec) -> String {
    let keys = Chord::parse(&spec.keys)
        .map(|c| c.to_omarchy_string())
        .unwrap_or_else(|_| spec.keys.clone());
    let dispatcher = match &spec.dispatcher {
        Dispatcher::Exec(cmd) => format!("hl.dsp.exec_cmd({})", lua_string(cmd)),
        Dispatcher::Lua(expr) => expr.clone(),
        Dispatcher::Function => "nil".to_string(),
    };

    let mut options = Vec::new();
    if !spec.description.is_empty() {
        options.push(format!("description = {}", lua_string(&spec.description)));
    }
    for flag in spec.options.set_flags() {
        options.push(format!("{flag} = true"));
    }

    format!(
        "hl.bind({}, {}, {{ {} }})",
        lua_string(&keys),
        dispatcher,
        options.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::system::keybinds::BindStatus;

    fn identity(keys: &str, description: &str, dispatcher: &str) -> BindIdentity {
        BindIdentity {
            keys: keys.into(),
            description: description.into(),
            dispatcher: dispatcher.into(),
        }
    }

    fn exec(keys: &str, description: &str, cmd: &str) -> BindSpec {
        BindSpec {
            keys: keys.into(),
            description: description.into(),
            dispatcher: Dispatcher::Exec(cmd.into()),
            options: BindOptions::default(),
        }
    }

    fn keybind(
        seq: u32,
        keys: &str,
        description: &str,
        dispatcher: Dispatcher,
        origin: Origin,
    ) -> Keybind {
        Keybind {
            seq,
            chord: Chord::parse(keys).unwrap(),
            keys_raw: keys.into(),
            description: description.into(),
            dispatcher,
            options: BindOptions::default(),
            origin,
            source: PathBuf::from("/x.lua"),
            status: BindStatus::Active,
        }
    }

    #[test]
    fn json_round_trip_uses_tagged_kinds_and_skips_defaults() {
        let mut overrides = KeybindOverrides::default();
        overrides.upsert(Override::Rebind {
            target: identity("SUPER + K", "Keybindings", "exec:omarchy-menu-keybindings"),
            bind: exec(
                "SUPER + SHIFT + K",
                "Keybindings",
                "omarchy-menu-keybindings",
            ),
            restore: vec![],
        });
        overrides.upsert(Override::Add {
            bind: BindSpec {
                options: BindOptions {
                    locked: true,
                    ..BindOptions::default()
                },
                ..exec("SUPER + SHIFT + R", "SSH", "alacritty -e ssh box")
            },
        });

        let json = serde_json::to_string_pretty(&overrides).unwrap();
        assert!(json.contains("\"kind\": \"rebind\""));
        assert!(json.contains("\"kind\": \"add\""));
        assert!(json.contains("\"exec\": \"alacritty -e ssh box\""));
        assert!(json.contains("\"locked\": true"));
        assert!(!json.contains("repeating"));
        assert_eq!(
            serde_json::from_str::<KeybindOverrides>(&json).unwrap(),
            overrides
        );

        let minimal: KeybindOverrides = serde_json::from_str(r#"{"version":1}"#).unwrap();
        assert!(minimal.is_empty());
    }

    #[test]
    fn upsert_replaces_an_override_on_the_same_target() {
        let target = identity("SUPER + K", "Keybindings", "exec:x");
        let mut overrides = KeybindOverrides::default();
        overrides.upsert(Override::Disable {
            target: target.clone(),
            restore: vec![],
        });
        overrides.upsert(Override::Rebind {
            target: target.clone(),
            bind: exec("SUPER + J", "Keybindings", "x"),
            restore: vec![],
        });
        assert_eq!(overrides.overrides.len(), 1);
        assert!(matches!(overrides.overrides[0], Override::Rebind { .. }));
        assert_eq!(overrides.find_for_target(&target), Some(0));
        assert!(overrides.remove(0).is_some());
        assert!(overrides.remove(0).is_none());
    }

    #[test]
    fn emits_unbinds_first_then_restores_rebinds_and_adds() {
        let mut overrides = KeybindOverrides::default();
        overrides.upsert(Override::Add {
            bind: exec("SUPER + SHIFT + R", "SSH", "alacritty -e ssh \"box\""),
        });
        overrides.upsert(Override::Rebind {
            target: identity("SUPER + K", "Keybindings", "exec:omarchy-menu-keybindings"),
            bind: BindSpec {
                options: BindOptions {
                    locked: true,
                    repeating: true,
                    ..BindOptions::default()
                },
                ..exec(
                    "shift + super + k",
                    "Keybindings",
                    "omarchy-menu-keybindings",
                )
            },
            restore: vec![BindSpec {
                keys: "SUPER + K".into(),
                description: "Sibling".into(),
                dispatcher: Dispatcher::Lua("hl.dsp.window.close()".into()),
                options: BindOptions::default(),
            }],
        });
        overrides.upsert(Override::Disable {
            target: identity("ALT + TAB", "Focus next", "lua:hl.dsp.window.cycle_next()"),
            restore: vec![],
        });

        let lua = emit_keybinds_lua(&overrides);
        let expected = "\
hl.unbind(\"ALT + TAB\")
hl.unbind(\"SUPER + K\")
hl.bind(\"SUPER + K\", hl.dsp.window.close(), { description = \"Sibling\" })
hl.bind(\"SUPER + SHIFT + K\", hl.dsp.exec_cmd(\"omarchy-menu-keybindings\"), { description = \"Keybindings\", locked = true, repeating = true })
hl.bind(\"SUPER + SHIFT + R\", hl.dsp.exec_cmd(\"alacritty -e ssh \\\"box\\\"\"), { description = \"SSH\" })
";
        assert_eq!(lua, expected);
        assert_eq!(emit_keybinds_lua(&KeybindOverrides::default()), "");
    }

    #[test]
    fn validate_rejects_code_injection_and_bad_chords() {
        let ok = KeybindOverrides {
            version: 1,
            overrides: vec![Override::Add {
                bind: BindSpec {
                    keys: "SUPER + W".into(),
                    description: String::new(),
                    dispatcher: Dispatcher::Lua("hl.dsp.focus({ workspace = \"1)\" })".into()),
                    options: BindOptions::default(),
                },
            }],
        };
        assert!(ok.validate().is_ok());

        for bad in [
            "os.execute(\"rm -rf ~\")",
            "hl.dsp.window.close(); os.exit()",
            "hl.dsp.window.close()\nos.exit()",
            "hl.dsp.window.close())",
            "hl.dsp.(x)",
        ] {
            let overrides = KeybindOverrides {
                version: 1,
                overrides: vec![Override::Add {
                    bind: BindSpec {
                        keys: "SUPER + W".into(),
                        description: String::new(),
                        dispatcher: Dispatcher::Lua(bad.into()),
                        options: BindOptions::default(),
                    },
                }],
            };
            assert!(overrides.validate().is_err(), "should reject {bad}");
        }

        let bad_keys = KeybindOverrides {
            version: 1,
            overrides: vec![Override::Add {
                bind: exec("SUPER +", "x", "y"),
            }],
        };
        assert!(bad_keys.validate().is_err());
    }

    #[test]
    fn restore_specs_keeps_rebindable_non_omarchist_siblings() {
        let target = keybind(
            1,
            "ALT + TAB",
            "Cycle",
            Dispatcher::Lua("hl.dsp.window.cycle_next()".into()),
            Origin::Default,
        );
        let all = vec![
            target.clone(),
            keybind(
                2,
                "ALT + TAB",
                "Raise",
                Dispatcher::Lua("hl.dsp.window.bring_to_top()".into()),
                Origin::Default,
            ),
            keybind(
                3,
                "ALT + TAB",
                "Custom fn",
                Dispatcher::Function,
                Origin::User,
            ),
            keybind(
                4,
                "ALT + TAB",
                "Mine",
                Dispatcher::Exec("x".into()),
                Origin::Omarchist,
            ),
            keybind(
                5,
                "SUPER + TAB",
                "Other chord",
                Dispatcher::Exec("y".into()),
                Origin::Default,
            ),
        ];
        let restore = restore_specs(&target, &all);
        assert_eq!(restore.len(), 1);
        assert_eq!(restore[0].description, "Raise");
        let lost = unrestorable_siblings(&target, &all);
        assert_eq!(lost.len(), 1);
        assert_eq!(lost[0].seq, 3);
    }
}
