// The single source of truth for keyboard shortcuts: `main.rs` registers
// `key_bindings()`, and the shortcuts help dialog and key hints read the
// same table, so a shortcut cannot be bound without being documented.
use gpui::{Action, KeyBinding};

use crate::ui::focus;
use crate::ui::menu::app_menu;
use crate::ui::sidebar_nav;

pub struct Shortcut {
    pub keys: &'static str,
    pub context: Option<&'static str>,
    pub group: &'static str,
    pub label: &'static str,
    /// Builds the concrete `KeyBinding` (the action type is fixed per row).
    bind: fn(&'static str, Option<&'static str>) -> KeyBinding,
    action: fn() -> Box<dyn Action>,
}

impl Shortcut {
    pub fn key_binding(&self) -> KeyBinding {
        (self.bind)(self.keys, self.context)
    }

    pub fn action(&self) -> Box<dyn Action> {
        (self.action)()
    }
}

macro_rules! shortcut {
    ($keys:expr, $action:expr, $context:expr, $group:expr, $label:expr) => {
        Shortcut {
            keys: $keys,
            context: $context,
            group: $group,
            label: $label,
            bind: |keys, context| KeyBinding::new(keys, $action, context),
            action: || Box::new($action),
        }
    };
}

pub const GLOBAL: &str = "Global";
pub const SIDEBAR: &str = "Sidebar";

pub const SHORTCUTS: &[Shortcut] = &[
    // Global
    shortcut!(
        "ctrl-1",
        app_menu::NavigateToThemes,
        None,
        GLOBAL,
        "Themes page"
    ),
    shortcut!(
        "ctrl-2",
        app_menu::NavigateToConfig,
        None,
        GLOBAL,
        "Configuration page"
    ),
    shortcut!(
        "ctrl-3",
        app_menu::NavigateToKeybinds,
        None,
        GLOBAL,
        "Keybinds page"
    ),
    shortcut!(
        "ctrl-,",
        app_menu::NavigateToSettings,
        None,
        GLOBAL,
        "Settings"
    ),
    shortcut!(
        "ctrl-n",
        app_menu::NewTheme,
        None,
        GLOBAL,
        "Create a new theme"
    ),
    shortcut!(
        "ctrl-r",
        focus::ReloadPage,
        None,
        GLOBAL,
        "Reload the current page"
    ),
    shortcut!(
        "ctrl-shift-r",
        app_menu::RefreshTheme,
        None,
        GLOBAL,
        "Re-apply the Omarchy theme"
    ),
    shortcut!(
        "ctrl-b",
        app_menu::ToggleSidebar,
        None,
        GLOBAL,
        "Toggle the sidebar"
    ),
    shortcut!(
        "ctrl-alt-l",
        app_menu::SwitchToLight,
        None,
        GLOBAL,
        "Light appearance"
    ),
    shortcut!(
        "ctrl-alt-d",
        app_menu::SwitchToDark,
        None,
        GLOBAL,
        "Dark appearance"
    ),
    shortcut!("ctrl-q", app_menu::Quit, None, GLOBAL, "Quit"),
    shortcut!("tab", focus::FocusNext, None, GLOBAL, "Next control"),
    shortcut!(
        "shift-tab",
        focus::FocusPrev,
        None,
        GLOBAL,
        "Previous control"
    ),
    shortcut!(
        "escape",
        focus::EscapeToSidebar,
        Some("MainWindow"),
        GLOBAL,
        "Jump between the sidebar and the page"
    ),
    // Sidebar
    shortcut!(
        "down",
        sidebar_nav::Next,
        Some("Sidebar"),
        SIDEBAR,
        "Next item"
    ),
    shortcut!(
        "up",
        sidebar_nav::Prev,
        Some("Sidebar"),
        SIDEBAR,
        "Previous item"
    ),
    shortcut!(
        "home",
        sidebar_nav::First,
        Some("Sidebar"),
        SIDEBAR,
        "First item"
    ),
    shortcut!(
        "end",
        sidebar_nav::Last,
        Some("Sidebar"),
        SIDEBAR,
        "Last item"
    ),
    shortcut!(
        "enter",
        sidebar_nav::Activate,
        Some("Sidebar"),
        SIDEBAR,
        "Open the item"
    ),
    shortcut!(
        "right",
        sidebar_nav::Activate,
        Some("Sidebar"),
        SIDEBAR,
        "Open the item"
    ),
];

pub fn key_bindings() -> Vec<KeyBinding> {
    SHORTCUTS.iter().map(Shortcut::key_binding).collect()
}

/// Shortcuts grouped for display, in table order.
pub fn help_groups() -> Vec<(&'static str, Vec<&'static Shortcut>)> {
    let mut groups: Vec<(&'static str, Vec<&'static Shortcut>)> = Vec::new();
    for shortcut in SHORTCUTS {
        match groups.iter_mut().find(|(name, _)| *name == shortcut.group) {
            Some((_, items)) => items.push(shortcut),
            None => groups.push((shortcut.group, vec![shortcut])),
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_shortcut_is_unique_within_its_context() {
        let mut seen = HashSet::new();
        for shortcut in SHORTCUTS {
            assert!(
                seen.insert((shortcut.keys, shortcut.context)),
                "duplicate shortcut {} in {:?}",
                shortcut.keys,
                shortcut.context
            );
        }
    }

    #[test]
    fn every_shortcut_has_a_label_and_parses() {
        for shortcut in SHORTCUTS {
            assert!(!shortcut.label.is_empty(), "{} has no label", shortcut.keys);
            // `KeyBinding::new` panics on an invalid keystroke or predicate.
            let _ = shortcut.key_binding();
        }
    }
}
