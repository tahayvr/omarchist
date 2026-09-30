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
    /// The identity this spec will have once Hyprland registers it, so an
    /// Omarchist-origin bind found by the scanner can be matched back to
    /// the override that emitted it.
    pub fn identity(&self) -> BindIdentity {
        BindIdentity {
            keys: Chord::parse(&self.keys)
                .map(|c| c.to_omarchy_string())
                .unwrap_or_else(|_| self.keys.clone()),
            description: self.description.clone(),
            dispatcher: self.dispatcher.identity(),
        }
    }

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

    pub fn restore(&self) -> &[BindSpec] {
        match self {
            Override::Rebind { restore, .. } | Override::Disable { restore, .. } => restore,
            Override::Add { .. } => &[],
        }
    }

    fn restore_mut(&mut self) -> Option<&mut Vec<BindSpec>> {
        match self {
            Override::Rebind { restore, .. } | Override::Disable { restore, .. } => Some(restore),
            Override::Add { .. } => None,
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
    /// A bind that another override re-registers as a sibling (`restore`)
    /// and that is now targeted itself is dropped from that list, or the
    /// restore would bring back what the new override removes.
    pub fn upsert(&mut self, override_: Override) {
        if let Some(target) = override_.target().cloned() {
            for other in &mut self.overrides {
                if other.target() != Some(&target)
                    && let Some(restore) = other.restore_mut()
                {
                    restore.retain(|spec| spec.identity() != target);
                }
            }
            if let Some(ix) = self.find_for_target(&target) {
                self.overrides[ix] = override_;
                return;
            }
        }
        self.overrides.push(override_);
    }

    /// The override whose `restore` list re-registers `identity`: the bind
    /// is still in effect through Omarchist's copy of it.
    pub fn restored_by(&self, identity: &BindIdentity) -> Option<usize> {
        self.overrides
            .iter()
            .position(|o| o.restore().iter().any(|spec| spec.identity() == *identity))
    }

    /// Overrides whose target no longer exists in the scanned config: the
    /// bind's description or command changed with an Omarchy update, so
    /// the override matches nothing but its `hl.unbind` still fires.
    pub fn stale(&self, binds: &[Keybind]) -> Vec<usize> {
        self.overrides
            .iter()
            .enumerate()
            .filter_map(|(ix, o)| {
                let target = o.target()?;
                let exists = binds
                    .iter()
                    .any(|bind| bind.origin != Origin::Omarchist && bind.identity() == *target);
                (!exists).then_some(ix)
            })
            .collect()
    }

    pub fn remove(&mut self, ix: usize) -> Option<Override> {
        (ix < self.overrides.len()).then(|| self.overrides.remove(ix))
    }

    /// Rejects anything that would not round-trip into safe Lua: unparsable
    /// chords, and Lua dispatchers that are not a single `hl.dsp.*(...)`
    /// call (a hand-edited json must not be able to inject arbitrary code
    /// into the compositor).
    pub fn validate(&self) -> Result<()> {
        self.overrides.iter().try_for_each(validate_override)
    }

    /// The overrides that pass [`Self::validate`]. `omarchist.lua` is
    /// rendered from these, so a hand-edited `keybinds.json` can never put
    /// arbitrary Lua into the compositor's config.
    pub fn valid_only(&self) -> Self {
        Self {
            version: self.version,
            overrides: self
                .overrides
                .iter()
                .filter(|o| match validate_override(o) {
                    Ok(()) => true,
                    Err(e) => {
                        eprintln!("Skipping an invalid keybind override: {e}");
                        false
                    }
                })
                .cloned()
                .collect(),
        }
    }
}

fn validate_override(override_: &Override) -> Result<()> {
    if let Some(target) = override_.target() {
        Chord::parse(&target.keys)?;
    }
    let specs = override_.bind().into_iter().chain(override_.restore());
    for spec in specs {
        Chord::parse(&spec.keys)?;
        validate_dispatcher(&spec.dispatcher)?;
    }
    Ok(())
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

/// `hl.dsp.<path>(<literal args>)` on one line, and nothing else: the
/// arguments may only be string, number, boolean and `nil` literals and
/// tables of those, which is everything the scanner ever reconstructs and
/// everything the action builder ever produces. Identifiers, calls and
/// operators inside the parentheses are rejected, so a hand-edited json or
/// an imported flow cannot smuggle `os.execute(..)` into the compositor as
/// an argument (`hl.dsp.focus(os.execute("x"))` is a valid Lua call).
pub(crate) fn is_dsp_call(expr: &str) -> bool {
    if expr.contains(['\n', '\r']) {
        return false;
    }
    let Some(rest) = expr.strip_prefix("hl.dsp.") else {
        return false;
    };
    let Some(open) = rest.find('(') else {
        return false;
    };
    let path = &rest[..open];
    if !path.split('.').all(is_identifier) {
        return false;
    }
    let mut parser = LiteralArgs {
        chars: rest[open..].chars().collect(),
        pos: 0,
    };
    parser.call_args() && parser.pos == parser.chars.len()
}

fn is_identifier(word: &str) -> bool {
    let mut chars = word.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Recursive-descent parser for a parenthesised list of Lua literals.
struct LiteralArgs {
    chars: Vec<char>,
    pos: usize,
}

impl LiteralArgs {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skip_spaces(&mut self) {
        while self.peek().is_some_and(|c| c == ' ' || c == '\t') {
            self.pos += 1;
        }
    }

    /// `( value, value, ... )`
    fn call_args(&mut self) -> bool {
        if !self.eat('(') {
            return false;
        }
        self.skip_spaces();
        if self.eat(')') {
            return true;
        }
        loop {
            if !self.value() {
                return false;
            }
            self.skip_spaces();
            if self.eat(')') {
                return true;
            }
            if !self.eat(',') {
                return false;
            }
            self.skip_spaces();
        }
    }

    fn value(&mut self) -> bool {
        self.skip_spaces();
        match self.peek() {
            Some('"') | Some('\'') => self.string(),
            Some('{') => self.table(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            Some(c) if c.is_ascii_alphabetic() => {
                let word = self.word();
                matches!(word.as_str(), "true" | "false" | "nil")
            }
            _ => false,
        }
    }

    fn word(&mut self) -> String {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// A quoted string; any backslash escape is accepted as two characters.
    fn string(&mut self) -> bool {
        let Some(quote) = self.peek() else {
            return false;
        };
        self.pos += 1;
        loop {
            match self.peek() {
                None => return false,
                Some('\\') => self.pos += 2,
                Some(c) if c == quote => {
                    self.pos += 1;
                    return true;
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    /// `-12`, `1.5`, `1e+20`, `0x1f`.
    fn number(&mut self) -> bool {
        self.eat('-');
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_hexdigit() || matches!(c, '.' | 'x' | 'X' | '+' | '-'))
        {
            // A sign is only part of an exponent.
            if matches!(self.peek(), Some('+') | Some('-'))
                && !matches!(self.chars.get(self.pos - 1), Some('e') | Some('E'))
            {
                break;
            }
            self.pos += 1;
        }
        self.pos > start && self.chars[start].is_ascii_digit()
    }

    /// `{ value, key = value, ["key"] = value }` with an optional trailing comma.
    fn table(&mut self) -> bool {
        if !self.eat('{') {
            return false;
        }
        loop {
            self.skip_spaces();
            if self.eat('}') {
                return true;
            }
            if !self.entry() {
                return false;
            }
            self.skip_spaces();
            if self.eat('}') {
                return true;
            }
            if !self.eat(',') {
                return false;
            }
        }
    }

    fn entry(&mut self) -> bool {
        self.skip_spaces();
        if self.eat('[') {
            if !self.value() {
                return false;
            }
            self.skip_spaces();
            if !self.eat(']') {
                return false;
            }
            self.skip_spaces();
            return self.eat('=') && self.value();
        }
        if self
            .peek()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        {
            let start = self.pos;
            let word = self.word();
            self.skip_spaces();
            if self.eat('=') {
                return is_identifier(&word) && self.value();
            }
            // Not a key: a bare literal such as `true`.
            self.pos = start;
        }
        self.value()
    }
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
    fn dsp_calls_cannot_chain_other_code() {
        assert!(is_dsp_call("hl.dsp.window.close()"));
        assert!(is_dsp_call(r#"hl.dsp.exec_cmd("echo (hi)")"#));
        assert!(is_dsp_call("hl.dsp.focus({ direction = \"l\" })"));
        assert!(!is_dsp_call(
            r#"hl.dsp.window.close() or os.execute("x") or hl.dsp.window.close()"#
        ));
        assert!(!is_dsp_call("hl.dsp.window.close()()"));
        assert!(!is_dsp_call("hl.dsp.a() .. hl.dsp.b()"));
        assert!(!is_dsp_call("hl.dsp.a(); os.exit()"));
    }

    #[test]
    fn dsp_call_arguments_must_be_literals() {
        // Every shape the scanner reconstructs and the builder produces.
        for ok in [
            "hl.dsp.window.resize({ x = 0, y = -100, relative = true })",
            "hl.dsp.window.move({ workspace = \"3\", follow = false })",
            "hl.dsp.window.cycle_next({ next = false })",
            "hl.dsp.layout(\"togglesplit\")",
            "hl.dsp.send_key_state({ mods = \"SUPER\", key = \"a\", state = \"down\" })",
            "hl.dsp.group.active({ index = 2 })",
            "hl.dsp.exec_cmd('single quotes')",
            r#"hl.dsp.exec_cmd("escaped \" quote and \\ backslash")"#,
            "hl.dsp.a(1.5, 1e+20, 0x1f, nil, { 1, 2, [\"k\"] = \"v\", [3] = true, })",
            "hl.dsp.a({})",
            "hl.dsp.a( )",
        ] {
            assert!(is_dsp_call(ok), "{ok}");
        }
        for bad in [
            r#"hl.dsp.focus(os.execute("curl x | sh"))"#,
            "hl.dsp.focus({ direction = os.getenv(\"X\") })",
            "hl.dsp.focus({ [os.exit()] = 1 })",
            "hl.dsp.focus(hl.dsp.window.close())",
            "hl.dsp.focus(x)",
            "hl.dsp.focus(1 + 1)",
            "hl.dsp.focus(\"a\" .. \"b\")",
            "hl.dsp.focus(function() end)",
            "hl.dsp.focus({ f = function() end })",
            "hl.dsp.focus(-x)",
            "hl.dsp.focus(\"unterminated)",
            "hl.dsp.focus(\"multi\nline\")",
            "hl.dsp..focus()",
            "hl.dsp.focus[1]()",
            "hl.dsp.focus() -- comment",
            "hl.dsp.focus(--[[ ]] 1)",
        ] {
            assert!(!is_dsp_call(bad), "{bad}");
        }
    }

    /// Every Lua bind in the installed Omarchy config passes the guard, so
    /// rebinding or disabling one can always re-emit its siblings.
    #[test]
    fn omarchys_lua_binds_pass_the_guard() {
        let Some(home) = dirs::home_dir() else {
            return;
        };
        let config = home.join(".config/hypr/hyprland.lua");
        let Ok(events) = crate::system::keybinds::scanner::run_scan(&config) else {
            eprintln!("skipping: no Omarchy config to scan");
            return;
        };
        let rejected: Vec<String> = events
            .iter()
            .filter_map(|event| match event {
                crate::system::keybinds::scanner::ScanEvent::Bind(bind) => match &bind.dispatcher {
                    Dispatcher::Lua(expr) if !is_dsp_call(expr) => Some(expr.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert!(rejected.is_empty(), "{rejected:#?}");
    }

    #[test]
    fn invalid_overrides_are_left_out_of_the_lua() {
        let bad = KeybindOverrides {
            version: 1,
            overrides: vec![Override::Add {
                bind: BindSpec {
                    keys: "SUPER + X".into(),
                    description: "Bad".into(),
                    dispatcher: Dispatcher::Lua(r#"os.execute("x")"#.into()),
                    options: Default::default(),
                },
            }],
        };
        assert!(bad.validate().is_err());
        assert!(bad.valid_only().overrides.is_empty());
        assert!(!emit_keybinds_lua(&bad.valid_only()).contains("os.execute"));
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
    fn disabling_a_restored_sibling_drops_it_from_the_other_override() {
        let cycle = keybind(
            1,
            "ALT + TAB",
            "Cycle",
            Dispatcher::Lua("hl.dsp.window.cycle_next()".into()),
            Origin::Default,
        );
        let raise = keybind(
            2,
            "ALT + TAB",
            "Raise",
            Dispatcher::Lua("hl.dsp.window.bring_to_top()".into()),
            Origin::Default,
        );
        let mut overrides = KeybindOverrides::default();
        // Disabling Cycle re-registers Raise, so Raise is restored by it.
        overrides.upsert(Override::Disable {
            target: cycle.identity(),
            restore: vec![BindSpec::from_keybind(&raise)],
        });
        assert_eq!(overrides.restored_by(&raise.identity()), Some(0));
        assert_eq!(overrides.restored_by(&cycle.identity()), None);

        // Now disabling Raise too must not leave it in Cycle's restore
        // list, or the Lua would unbind it and register it again.
        overrides.upsert(Override::Disable {
            target: raise.identity(),
            restore: vec![],
        });
        assert_eq!(overrides.overrides.len(), 2);
        assert!(overrides.overrides[0].restore().is_empty());
        assert_eq!(overrides.restored_by(&raise.identity()), None);
    }

    #[test]
    fn overrides_whose_target_is_gone_are_stale() {
        let bind = keybind(
            1,
            "SUPER + K",
            "Keybindings",
            Dispatcher::Exec("x".into()),
            Origin::Default,
        );
        let renamed = keybind(
            1,
            "SUPER + K",
            "Keybinding menu",
            Dispatcher::Exec("x".into()),
            Origin::Default,
        );
        let overrides = KeybindOverrides {
            version: 1,
            overrides: vec![
                Override::Disable {
                    target: bind.identity(),
                    restore: vec![],
                },
                Override::Add {
                    bind: BindSpec::from_keybind(&renamed),
                },
            ],
        };
        assert!(overrides.stale(std::slice::from_ref(&bind)).is_empty());
        assert_eq!(
            overrides.stale(&[renamed]),
            vec![0],
            "an Add is never stale"
        );
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
