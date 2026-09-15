// Omarchy's keys string (`"SUPER + SHIFT + K"`) parsed into modifiers + key.
//
// Modifier names and masks follow Hyprland (SHIFT=1, CTRL=4, ALT=8, SUPER=64;
// order in the string is cosmetic). The key part keeps the author's spelling
// because Hyprland resolves it as an xkb keysym name and some names are
// case-sensitive (Omarchy binds `comma`, not `COMMA`); `code:N` raw keycodes,
// `mouse:272`, `mouse_down`, and `switch:on:Lid Switch` pass through as-is.
use std::fmt;
use std::ops::{BitOr, BitOrAssign};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModMask(pub u8);

impl ModMask {
    pub const NONE: ModMask = ModMask(0);
    pub const SHIFT: ModMask = ModMask(1);
    pub const CTRL: ModMask = ModMask(4);
    pub const ALT: ModMask = ModMask(8);
    pub const SUPER: ModMask = ModMask(64);

    /// Hyprland modifier names, in the order Omarchy prints them.
    const ORDERED: [(ModMask, &'static str); 4] = [
        (ModMask::SUPER, "SUPER"),
        (ModMask::SHIFT, "SHIFT"),
        (ModMask::CTRL, "CTRL"),
        (ModMask::ALT, "ALT"),
    ];

    pub fn from_name(name: &str) -> Option<ModMask> {
        match name.to_ascii_uppercase().as_str() {
            "SHIFT" => Some(ModMask::SHIFT),
            "CTRL" | "CONTROL" => Some(ModMask::CTRL),
            "ALT" | "MOD1" => Some(ModMask::ALT),
            "SUPER" | "WIN" | "LOGO" | "MOD4" => Some(ModMask::SUPER),
            _ => None,
        }
    }

    pub fn contains(self, other: ModMask) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Set modifier names, e.g. `["SUPER", "SHIFT"]`.
    pub fn names(self) -> Vec<&'static str> {
        Self::ORDERED
            .iter()
            .filter(|(mask, _)| self.contains(*mask))
            .map(|(_, name)| *name)
            .collect()
    }
}

impl BitOr for ModMask {
    type Output = ModMask;

    fn bitor(self, rhs: ModMask) -> ModMask {
        ModMask(self.0 | rhs.0)
    }
}

impl BitOrAssign for ModMask {
    fn bitor_assign(&mut self, rhs: ModMask) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Chord {
    pub mods: ModMask,
    pub key: String,
}

impl Chord {
    pub fn new(mods: ModMask, key: impl Into<String>) -> Self {
        Self {
            mods,
            key: key.into(),
        }
    }

    /// Parses Omarchy's `"SUPER + SHIFT + K"` form. Modifiers are
    /// case-insensitive and may appear in any order; exactly one non-modifier
    /// part is required.
    pub fn parse(text: &str) -> Result<Chord> {
        let mut mods = ModMask::NONE;
        let mut key: Option<&str> = None;

        for part in text.split('+').map(str::trim) {
            if part.is_empty() {
                return Err(Error::Invalid(format!("Invalid keybind '{text}'")));
            }
            if let Some(modifier) = ModMask::from_name(part) {
                mods |= modifier;
            } else if key.is_some() {
                return Err(Error::Invalid(format!(
                    "Invalid keybind '{text}': only one key is allowed"
                )));
            } else {
                key = Some(part);
            }
        }

        match key {
            Some(key) => Ok(Chord::new(mods, key)),
            None => Err(Error::Invalid(format!(
                "Invalid keybind '{text}': a key is required"
            ))),
        }
    }

    /// Canonical Omarchy spelling: `SUPER + SHIFT + K`. Single letters are
    /// uppercased (Hyprland treats `k` and `K` as the same key; Omarchy
    /// writes them uppercase); every other key keeps its spelling.
    pub fn to_omarchy_string(&self) -> String {
        let mut parts = self.mods.names();
        let key = self.canonical_key();
        parts.push(&key);
        parts.join(" + ")
    }

    fn canonical_key(&self) -> String {
        let mut chars = self.key.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if c.is_ascii_alphabetic() => c.to_ascii_uppercase().to_string(),
            _ => self.key.clone(),
        }
    }

    /// Case-folded key with `code:N` resolved to its US-layout keysym, so
    /// `SUPER + 1` and `SUPER + code:10` compare equal.
    pub fn key_id(&self) -> String {
        if let Some(keysym) = self.keycode_keysym() {
            return keysym.to_string();
        }
        self.key.to_lowercase()
    }

    pub fn id(&self) -> (ModMask, String) {
        (self.mods, self.key_id())
    }

    pub fn same_as(&self, other: &Chord) -> bool {
        self.id() == other.id()
    }

    pub fn is_mouse(&self) -> bool {
        self.key.starts_with("mouse")
    }

    pub fn is_switch(&self) -> bool {
        self.key.starts_with("switch:")
    }

    /// The US keysym behind a `code:N` key, for the keycodes Omarchy uses.
    pub fn keycode_keysym(&self) -> Option<&'static str> {
        let code: u32 = self.key.strip_prefix("code:")?.parse().ok()?;
        keycode_to_keysym(code)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_omarchy_string())
    }
}

/// X11 keycode (evdev + 8) → US keysym for the layout-independent keycodes
/// Omarchy's defaults use (workspace rows and the bracket/punctuation keys),
/// matching the fallback table in `omarchy-menu-keybindings`.
pub fn keycode_to_keysym(code: u32) -> Option<&'static str> {
    Some(match code {
        10 => "1",
        11 => "2",
        12 => "3",
        13 => "4",
        14 => "5",
        15 => "6",
        16 => "7",
        17 => "8",
        18 => "9",
        19 => "0",
        20 => "minus",
        21 => "equal",
        34 => "bracketleft",
        35 => "bracketright",
        59 => "comma",
        60 => "period",
        61 => "slash",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers_in_any_order_and_case() {
        let chord = Chord::parse("shift + super + k").unwrap();
        assert_eq!(chord.mods, ModMask::SUPER | ModMask::SHIFT);
        assert_eq!(chord.key, "k");
        assert_eq!(chord.to_omarchy_string(), "SUPER + SHIFT + K");

        let chord = Chord::parse("SUPER + CONTROL + ALT + RIGHT").unwrap();
        assert_eq!(chord.mods, ModMask::SUPER | ModMask::CTRL | ModMask::ALT);
        assert_eq!(chord.to_omarchy_string(), "SUPER + CTRL + ALT + RIGHT");
    }

    #[test]
    fn keeps_key_spelling_and_special_keys() {
        assert_eq!(Chord::parse("SUPER + comma").unwrap().key, "comma");
        assert_eq!(Chord::parse("SUPER + code:10").unwrap().key, "code:10");
        assert!(Chord::parse("SUPER + mouse:272").unwrap().is_mouse());
        assert!(Chord::parse("SUPER + mouse_down").unwrap().is_mouse());

        let lid = Chord::parse("switch:on:Lid Switch").unwrap();
        assert!(lid.is_switch());
        assert_eq!(lid.mods, ModMask::NONE);
        assert_eq!(lid.key, "switch:on:Lid Switch");

        let media = Chord::parse("XF86AudioRaiseVolume").unwrap();
        assert_eq!(media.to_omarchy_string(), "XF86AudioRaiseVolume");
    }

    #[test]
    fn rejects_empty_missing_and_multiple_keys() {
        assert!(Chord::parse("").is_err());
        assert!(Chord::parse("SUPER").is_err());
        assert!(Chord::parse("SUPER + ").is_err());
        assert!(Chord::parse("SUPER + A + B").is_err());
    }

    #[test]
    fn key_id_folds_case_and_resolves_keycodes() {
        let one = Chord::parse("SUPER + 1").unwrap();
        let code = Chord::parse("SUPER + code:10").unwrap();
        assert!(one.same_as(&code));
        assert_eq!(code.key_id(), "1");
        assert_eq!(Chord::parse("SUPER + K").unwrap().key_id(), "k");
        assert_eq!(Chord::parse("SUPER + code:99").unwrap().key_id(), "code:99");
        assert!(!Chord::parse("SHIFT + 1").unwrap().same_as(&one));
    }

    #[test]
    fn chord_serde_round_trip() {
        let chord = Chord::parse("SUPER + SHIFT + K").unwrap();
        let json = serde_json::to_string(&chord).unwrap();
        assert_eq!(json, r#"{"mods":65,"key":"K"}"#);
        assert_eq!(serde_json::from_str::<Chord>(&json).unwrap(), chord);
    }
}
