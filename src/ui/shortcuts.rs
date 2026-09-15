// The single source of truth for keyboard shortcuts: `main.rs` registers
// `key_bindings()`, and the shortcuts help dialog and key hints read the
// same table, so a shortcut cannot be bound without being documented.
use gpui::{Action, KeyBinding};

use crate::ui::focus::{self, dialog, tab_strip};
use crate::ui::menu::app_menu;
use crate::ui::sidebar_nav;
use crate::ui::theme_edit_page::theme_edit_view as theme_edit;
use crate::ui::themes_page::theme_grid;

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
pub const TABS: &str = "Tab strips";
pub const THEMES: &str = "Themes";
pub const THEME_EDIT: &str = "Theme Designer";
pub const DIALOGS: &str = "Dialogs";

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
    // Tab strips (Themes filter, Designer tabs)
    shortcut!(
        "left",
        tab_strip::Prev,
        Some("TabStrip"),
        TABS,
        "Previous tab"
    ),
    shortcut!("right", tab_strip::Next, Some("TabStrip"), TABS, "Next tab"),
    shortcut!(
        "home",
        tab_strip::First,
        Some("TabStrip"),
        TABS,
        "First tab"
    ),
    shortcut!("end", tab_strip::Last, Some("TabStrip"), TABS, "Last tab"),
    shortcut!(
        "enter",
        tab_strip::Activate,
        Some("TabStrip"),
        TABS,
        "Go to the tab's content"
    ),
    shortcut!(
        "down",
        tab_strip::Activate,
        Some("TabStrip"),
        TABS,
        "Go to the tab's content"
    ),
    // Theme grid
    shortcut!(
        "up",
        theme_grid::Up,
        Some("ThemeGrid"),
        THEMES,
        "Card above"
    ),
    shortcut!(
        "down",
        theme_grid::Down,
        Some("ThemeGrid"),
        THEMES,
        "Card below"
    ),
    shortcut!(
        "left",
        theme_grid::Left,
        Some("ThemeGrid"),
        THEMES,
        "Previous card"
    ),
    shortcut!(
        "right",
        theme_grid::Right,
        Some("ThemeGrid"),
        THEMES,
        "Next card"
    ),
    shortcut!(
        "home",
        theme_grid::First,
        Some("ThemeGrid"),
        THEMES,
        "First card"
    ),
    shortcut!(
        "end",
        theme_grid::Last,
        Some("ThemeGrid"),
        THEMES,
        "Last card"
    ),
    shortcut!(
        "pageup",
        theme_grid::PageUp,
        Some("ThemeGrid"),
        THEMES,
        "Page up"
    ),
    shortcut!(
        "pagedown",
        theme_grid::PageDown,
        Some("ThemeGrid"),
        THEMES,
        "Page down"
    ),
    shortcut!(
        "enter",
        theme_grid::Apply,
        Some("ThemeGrid"),
        THEMES,
        "Apply the theme"
    ),
    shortcut!(
        "e",
        theme_grid::Edit,
        Some("ThemeGrid"),
        THEMES,
        "Edit the theme"
    ),
    shortcut!(
        "o",
        theme_grid::OpenFolder,
        Some("ThemeGrid"),
        THEMES,
        "Open the theme folder"
    ),
    shortcut!(
        "delete",
        theme_grid::Delete,
        Some("ThemeGrid"),
        THEMES,
        "Delete the theme"
    ),
    shortcut!(
        "escape",
        theme_grid::LeaveGrid,
        Some("ThemeGrid"),
        THEMES,
        "Back to the filter tabs"
    ),
    // Theme Designer
    shortcut!(
        "escape",
        app_menu::NavigateBack,
        Some("ThemeEditPage"),
        THEME_EDIT,
        "Back to Themes"
    ),
    shortcut!(
        "alt-left",
        app_menu::NavigateBack,
        Some("ThemeEditPage"),
        THEME_EDIT,
        "Back to Themes"
    ),
    shortcut!(
        "ctrl-pagedown",
        app_menu::ThemeEditNextTab,
        Some("ThemeEditPage"),
        THEME_EDIT,
        "Next tab"
    ),
    shortcut!(
        "ctrl-pageup",
        app_menu::ThemeEditPrevTab,
        Some("ThemeEditPage"),
        THEME_EDIT,
        "Previous tab"
    ),
    shortcut!(
        "ctrl-tab",
        app_menu::ThemeEditNextTab,
        Some("ThemeEditPage"),
        THEME_EDIT,
        "Next tab"
    ),
    shortcut!(
        "ctrl-shift-tab",
        app_menu::ThemeEditPrevTab,
        Some("ThemeEditPage"),
        THEME_EDIT,
        "Previous tab"
    ),
    shortcut!(
        "ctrl-s",
        theme_edit::ApplyTheme,
        Some("ThemeEditPage"),
        THEME_EDIT,
        "Apply the theme"
    ),
    // Dialogs
    shortcut!(
        "ctrl-enter",
        dialog::Submit,
        Some("DialogBody"),
        DIALOGS,
        "Confirm"
    ),
    shortcut!(
        "ctrl-enter",
        dialog::Submit,
        Some("DialogBody > Input"),
        DIALOGS,
        "Confirm (from a text field)"
    ),
    shortcut!(
        "tab",
        focus::FocusNext,
        Some("DialogBody > Input"),
        DIALOGS,
        "Next control (from a text field)"
    ),
    shortcut!(
        "shift-tab",
        focus::FocusPrev,
        Some("DialogBody > Input"),
        DIALOGS,
        "Previous control (from a text field)"
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
