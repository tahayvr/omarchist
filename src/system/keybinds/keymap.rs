// Translates what GPUI reports for a key press into a Hyprland chord, and
// renders chords for display.
//
// GPUI's Linux backend (gpui/src/platform/linux/platform.rs, `Keystroke::from_xkb`)
// names keys as: lowercase letters, digits, the *shifted* symbol when Shift is
// held (`!` for shift-1), literal punctuation (`,` `.` `/` `` ` ``), a few
// named keys (`enter`, `pageup`, `escape`, ...), keypad keys with `kp_`
// stripped, and every other keysym as its lowercase xkb name
// (`xf86audioraisevolume`). Hyprland wants xkb keysym *names*, which are
// case-sensitive for some symbols, so the tables below map back to the
// spellings Omarchy itself uses and xkbcommon canonicalises the rest.
use gpui::{Keystroke, Modifiers};
use xkbcommon::xkb;

use super::chord::{Chord, ModMask};

pub fn modifiers_to_modmask(modifiers: &Modifiers) -> ModMask {
    let mut mask = ModMask::NONE;
    if modifiers.platform {
        mask |= ModMask::SUPER;
    }
    if modifiers.shift {
        mask |= ModMask::SHIFT;
    }
    if modifiers.control {
        mask |= ModMask::CTRL;
    }
    if modifiers.alt {
        mask |= ModMask::ALT;
    }
    mask
}

/// `None` when the keystroke carries no bindable key (modifier-only events).
pub fn keystroke_to_chord(keystroke: &Keystroke) -> Option<Chord> {
    let (key, force_shift) = gpui_key_to_hypr(&keystroke.key)?;
    let mut mods = modifiers_to_modmask(&keystroke.modifiers);
    if force_shift {
        mods |= ModMask::SHIFT;
    }
    Some(Chord::new(mods, key))
}

/// Maps a GPUI key name to a Hyprland key name. The bool is set when the
/// GPUI name was a shifted symbol (`!`), which Hyprland expresses as the
/// base key plus SHIFT (`SHIFT + 1`).
pub fn gpui_key_to_hypr(key: &str) -> Option<(String, bool)> {
    if key.is_empty() || is_modifier_key_name(key) {
        return None;
    }

    let mut chars = key.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        if c.is_ascii_alphabetic() {
            return Some((c.to_ascii_uppercase().to_string(), false));
        }
        if c.is_ascii_digit() {
            return Some((c.to_string(), false));
        }
        if let Some((base, shifted)) = symbol_keysym(c) {
            return Some((base.to_string(), shifted));
        }
    }

    if let Some(name) = named_key(key) {
        return Some((name.to_string(), false));
    }

    if let Some(number) = key.strip_prefix('f')
        && number.parse::<u8>().is_ok_and(|n| (1..=35).contains(&n))
    {
        return Some((key.to_ascii_uppercase(), false));
    }

    canonical_keysym_name(key).map(|name| (name, false))
}

fn is_modifier_key_name(key: &str) -> bool {
    matches!(
        key,
        "shift"
            | "control"
            | "alt"
            | "platform"
            | "function"
            | "shift_l"
            | "shift_r"
            | "control_l"
            | "control_r"
            | "alt_l"
            | "alt_r"
            | "super_l"
            | "super_r"
            | "meta_l"
            | "meta_r"
            | "hyper_l"
            | "hyper_r"
    )
}

/// ASCII punctuation → (keysym name, was_shifted), US layout.
fn symbol_keysym(c: char) -> Option<(&'static str, bool)> {
    Some(match c {
        ',' => ("comma", false),
        '.' => ("period", false),
        '/' => ("slash", false),
        ';' => ("semicolon", false),
        '\'' => ("apostrophe", false),
        '[' => ("bracketleft", false),
        ']' => ("bracketright", false),
        '\\' => ("backslash", false),
        '`' => ("grave", false),
        '-' => ("minus", false),
        '=' => ("equal", false),
        '!' => ("1", true),
        '@' => ("2", true),
        '#' => ("3", true),
        '$' => ("4", true),
        '%' => ("5", true),
        '^' => ("6", true),
        '&' => ("7", true),
        '*' => ("8", true),
        '(' => ("9", true),
        ')' => ("0", true),
        '_' => ("minus", true),
        '+' => ("equal", true),
        '{' => ("bracketleft", true),
        '}' => ("bracketright", true),
        '|' => ("backslash", true),
        ':' => ("semicolon", true),
        '"' => ("apostrophe", true),
        '<' => ("comma", true),
        '>' => ("period", true),
        '?' => ("slash", true),
        '~' => ("grave", true),
        _ => return None,
    })
}

/// GPUI's named keys → the spellings Omarchy's own bindings use.
fn named_key(key: &str) -> Option<&'static str> {
    Some(match key {
        "enter" => "RETURN",
        "space" => "SPACE",
        "tab" => "TAB",
        "escape" => "ESCAPE",
        "backspace" => "BACKSPACE",
        "delete" => "DELETE",
        "insert" => "INSERT",
        "left" => "LEFT",
        "right" => "RIGHT",
        "up" => "UP",
        "down" => "DOWN",
        "home" => "HOME",
        "end" => "END",
        "pageup" => "Prior",
        "pagedown" => "Next",
        "print" => "Print",
        "pause" => "Pause",
        "menu" => "Menu",
        "scroll_lock" => "Scroll_Lock",
        "num_lock" => "Num_Lock",
        "caps_lock" => "Caps_Lock",
        _ => return None,
    })
}

/// Recovers the properly cased xkb keysym name from GPUI's lowercased one
/// (`xf86audioraisevolume` → `XF86AudioRaiseVolume`).
fn canonical_keysym_name(lowercase: &str) -> Option<String> {
    let keysym = xkb::keysym_from_name(lowercase, xkb::KEYSYM_CASE_INSENSITIVE);
    if keysym == xkb::Keysym::NoSymbol {
        return None;
    }
    let name = xkb::keysym_get_name(keysym);
    if name.is_empty() { None } else { Some(name) }
}

/// Display labels for a chord's chips: `["Super", "Shift", "K"]`.
pub fn chord_display_parts(chord: &Chord) -> Vec<String> {
    let mut parts: Vec<String> = chord
        .mods
        .names()
        .into_iter()
        .map(|name| match name {
            "SUPER" => "Super".to_string(),
            "SHIFT" => "Shift".to_string(),
            "CTRL" => "Ctrl".to_string(),
            "ALT" => "Alt".to_string(),
            other => other.to_string(),
        })
        .collect();
    parts.push(display_key(chord));
    parts
}

fn display_key(chord: &Chord) -> String {
    if let Some(keysym) = chord.keycode_keysym() {
        return keysym_label(keysym);
    }
    if let Some(button) = chord.key.strip_prefix("mouse:") {
        return match button {
            "272" => "Left Click".to_string(),
            "273" => "Right Click".to_string(),
            "274" => "Middle Click".to_string(),
            other => format!("Mouse {other}"),
        };
    }
    match chord.key.as_str() {
        "mouse_down" => return "Scroll Down".to_string(),
        "mouse_up" => return "Scroll Up".to_string(),
        _ => {}
    }
    if let Some(rest) = chord.key.strip_prefix("switch:") {
        return format!("Switch {rest}");
    }
    keysym_label(&chord.key)
}

fn keysym_label(keysym: &str) -> String {
    let lower = keysym.to_ascii_lowercase();
    let label = match lower.as_str() {
        "return" | "enter" => "Enter",
        "space" => "Space",
        "tab" => "Tab",
        "escape" => "Esc",
        "backspace" => "Backspace",
        "delete" => "Delete",
        "insert" => "Insert",
        "left" => "←",
        "right" => "→",
        "up" => "↑",
        "down" => "↓",
        "home" => "Home",
        "end" => "End",
        "prior" | "page_up" => "Page Up",
        "next" | "page_down" => "Page Down",
        "comma" => ",",
        "period" => ".",
        "slash" => "/",
        "semicolon" => ";",
        "apostrophe" => "'",
        "bracketleft" => "[",
        "bracketright" => "]",
        "backslash" => "\\",
        "grave" => "`",
        "minus" => "-",
        "equal" => "=",
        "print" => "Print",
        _ => {
            if let Some(rest) = keysym.strip_prefix("XF86") {
                return rest.to_string();
            }
            if keysym.chars().count() == 1 {
                return keysym.to_ascii_uppercase();
            }
            return keysym.to_string();
        }
    };
    label.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keystroke(key: &str, modifiers: Modifiers) -> Keystroke {
        Keystroke {
            modifiers,
            key: key.to_string(),
            key_char: None,
        }
    }

    fn super_mod() -> Modifiers {
        Modifiers {
            platform: true,
            ..Modifiers::default()
        }
    }

    #[test]
    fn letters_become_uppercase_with_super() {
        let chord = keystroke_to_chord(&keystroke("k", super_mod())).unwrap();
        assert_eq!(chord.to_omarchy_string(), "SUPER + K");
    }

    #[test]
    fn shifted_symbols_map_to_base_key_plus_shift() {
        let bang = keystroke_to_chord(&keystroke(
            "!",
            Modifiers {
                shift: true,
                ..Modifiers::default()
            },
        ))
        .unwrap();
        assert_eq!(bang.to_omarchy_string(), "SHIFT + 1");

        let tilde = keystroke_to_chord(&keystroke(
            "~",
            Modifiers {
                control: true,
                shift: true,
                ..Modifiers::default()
            },
        ))
        .unwrap();
        assert_eq!(tilde.to_omarchy_string(), "SHIFT + CTRL + grave");
    }

    #[test]
    fn named_and_punctuation_keys_use_omarchy_spellings() {
        assert_eq!(gpui_key_to_hypr("enter").unwrap().0, "RETURN");
        assert_eq!(gpui_key_to_hypr("pageup").unwrap().0, "Prior");
        assert_eq!(gpui_key_to_hypr(",").unwrap().0, "comma");
        assert_eq!(gpui_key_to_hypr("`").unwrap().0, "grave");
        assert_eq!(gpui_key_to_hypr("f9").unwrap().0, "F9");
        assert_eq!(gpui_key_to_hypr("print").unwrap().0, "Print");
        assert_eq!(gpui_key_to_hypr("5").unwrap().0, "5");
    }

    #[test]
    fn xf86_keys_are_canonicalised_by_xkb() {
        assert_eq!(
            gpui_key_to_hypr("xf86audioraisevolume").unwrap().0,
            "XF86AudioRaiseVolume"
        );
        assert_eq!(
            gpui_key_to_hypr("xf86monbrightnessup").unwrap().0,
            "XF86MonBrightnessUp"
        );
        assert!(gpui_key_to_hypr("definitely_not_a_keysym").is_none());
    }

    #[test]
    fn modifier_only_keystrokes_yield_no_chord() {
        assert!(keystroke_to_chord(&keystroke("", super_mod())).is_none());
        assert!(keystroke_to_chord(&keystroke("shift_l", super_mod())).is_none());
    }

    #[test]
    fn display_parts_are_human_readable() {
        let parts = chord_display_parts(&Chord::parse("SUPER + SHIFT + RETURN").unwrap());
        assert_eq!(parts, vec!["Super", "Shift", "Enter"]);
        let parts = chord_display_parts(&Chord::parse("SUPER + code:10").unwrap());
        assert_eq!(parts, vec!["Super", "1"]);
        let parts = chord_display_parts(&Chord::parse("SUPER + mouse:272").unwrap());
        assert_eq!(parts, vec!["Super", "Left Click"]);
        let parts = chord_display_parts(&Chord::parse("XF86AudioMute").unwrap());
        assert_eq!(parts, vec!["AudioMute"]);
        let parts = chord_display_parts(&Chord::parse("SUPER + comma").unwrap());
        assert_eq!(parts, vec!["Super", ","]);
    }
}
