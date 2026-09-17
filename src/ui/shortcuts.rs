// The single source of truth for keyboard shortcuts: `main.rs` registers
// `key_bindings()`, and the shortcuts help dialog and key hints read the
// same table, so a shortcut cannot be bound without being documented.
use gpui::{Action, KeyBinding};

use crate::ui::config_page::config_view::config_nav;
use crate::ui::flows_page::flow_edit_view::flow_edit_nav;
use crate::ui::flows_page::flows_view::flows_nav;
use crate::ui::focus::{self, dialog, tab_strip};
use crate::ui::keybinds_page::keybinds_view::keybinds_nav;
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
pub const CONFIG: &str = "Configuration";
pub const KEYBINDS: &str = "Keybinds";
pub const FLOWS: &str = "Flows";

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
        "ctrl-4",
        app_menu::NavigateToFlows,
        None,
        GLOBAL,
        "Flows page"
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
    shortcut!(
        "ctrl-/",
        focus::ShowShortcuts,
        None,
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "ctrl-shift-p",
        focus::ShowCommands,
        None,
        GLOBAL,
        "Command palette"
    ),
    shortcut!(
        "ctrl-p",
        focus::ShowCommands,
        None,
        GLOBAL,
        "Command palette"
    ),
    shortcut!(
        "?",
        focus::ShowShortcuts,
        Some("Sidebar"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "shift-?",
        focus::ShowShortcuts,
        Some("Sidebar"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "?",
        focus::ShowShortcuts,
        Some("TabStrip"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "shift-?",
        focus::ShowShortcuts,
        Some("TabStrip"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "?",
        focus::ShowShortcuts,
        Some("ThemeGrid"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "shift-?",
        focus::ShowShortcuts,
        Some("ThemeGrid"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "?",
        focus::ShowShortcuts,
        Some("ConfigNav"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "shift-?",
        focus::ShowShortcuts,
        Some("ConfigNav"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "?",
        focus::ShowShortcuts,
        Some("KeybindsTable"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "shift-?",
        focus::ShowShortcuts,
        Some("KeybindsTable"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "?",
        focus::ShowShortcuts,
        Some("KeybindsFilters"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
    shortcut!(
        "shift-?",
        focus::ShowShortcuts,
        Some("KeybindsFilters"),
        GLOBAL,
        "Show keyboard shortcuts"
    ),
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
    // Configuration
    shortcut!(
        "up",
        config_nav::Prev,
        Some("ConfigNav"),
        CONFIG,
        "Previous section"
    ),
    shortcut!(
        "down",
        config_nav::Next,
        Some("ConfigNav"),
        CONFIG,
        "Next section"
    ),
    shortcut!(
        "home",
        config_nav::First,
        Some("ConfigNav"),
        CONFIG,
        "First section"
    ),
    shortcut!(
        "end",
        config_nav::Last,
        Some("ConfigNav"),
        CONFIG,
        "Last section"
    ),
    shortcut!(
        "enter",
        config_nav::Activate,
        Some("ConfigNav"),
        CONFIG,
        "Go to the section's settings"
    ),
    shortcut!(
        "right",
        config_nav::Activate,
        Some("ConfigNav"),
        CONFIG,
        "Go to the section's settings"
    ),
    shortcut!(
        "escape",
        config_nav::Back,
        Some("ConfigContent"),
        CONFIG,
        "Back to the section list"
    ),
    // Keybinds page
    shortcut!(
        "ctrl-f",
        keybinds_nav::FocusSearch,
        Some("KeybindsPage"),
        KEYBINDS,
        "Search"
    ),
    shortcut!(
        "/",
        keybinds_nav::FocusSearch,
        Some("KeybindsTable"),
        KEYBINDS,
        "Search"
    ),
    shortcut!(
        "/",
        keybinds_nav::FocusSearch,
        Some("KeybindsFilters"),
        KEYBINDS,
        "Search"
    ),
    shortcut!(
        "ctrl-k",
        keybinds_nav::ToggleChordSearch,
        Some("KeybindsPage"),
        KEYBINDS,
        "Search by pressing keys"
    ),
    shortcut!(
        "ctrl-shift-n",
        keybinds_nav::AddKeybind,
        Some("KeybindsPage"),
        KEYBINDS,
        "Add a keybind"
    ),
    shortcut!(
        "alt-1",
        keybinds_nav::SetFilter(0),
        Some("KeybindsPage"),
        KEYBINDS,
        "Show all"
    ),
    shortcut!(
        "alt-2",
        keybinds_nav::SetFilter(1),
        Some("KeybindsPage"),
        KEYBINDS,
        "Show modified"
    ),
    shortcut!(
        "alt-3",
        keybinds_nav::SetFilter(2),
        Some("KeybindsPage"),
        KEYBINDS,
        "Show conflicts"
    ),
    shortcut!(
        "alt-4",
        keybinds_nav::SetFilter(3),
        Some("KeybindsPage"),
        KEYBINDS,
        "Show Omarchy defaults"
    ),
    shortcut!(
        "alt-5",
        keybinds_nav::SetFilter(4),
        Some("KeybindsPage"),
        KEYBINDS,
        "Show your keybinds"
    ),
    shortcut!(
        "down",
        keybinds_nav::FocusTable,
        Some("KeybindsSearch"),
        KEYBINDS,
        "From the search box to the table"
    ),
    shortcut!(
        "down",
        keybinds_nav::FocusTable,
        Some("KeybindsSearch > Input"),
        KEYBINDS,
        "From the search box to the table"
    ),
    shortcut!(
        "escape",
        keybinds_nav::ClearSearch,
        Some("KeybindsSearch"),
        KEYBINDS,
        "Clear the search, then go to the table"
    ),
    shortcut!(
        "escape",
        keybinds_nav::ClearSearch,
        Some("KeybindsSearch > Input"),
        KEYBINDS,
        "Clear the search, then go to the table"
    ),
    shortcut!(
        "left",
        keybinds_nav::FilterPrev,
        Some("KeybindsFilters"),
        KEYBINDS,
        "Previous filter"
    ),
    shortcut!(
        "right",
        keybinds_nav::FilterNext,
        Some("KeybindsFilters"),
        KEYBINDS,
        "Next filter"
    ),
    shortcut!(
        "enter",
        keybinds_nav::EditSelected,
        Some("KeybindsTable"),
        KEYBINDS,
        "Edit the selected keybind"
    ),
    shortcut!(
        "delete",
        keybinds_nav::DisableSelected,
        Some("KeybindsTable"),
        KEYBINDS,
        "Disable the selected keybind"
    ),
    shortcut!(
        "ctrl-c",
        keybinds_nav::CopySelectedCommand,
        Some("KeybindsTable"),
        KEYBINDS,
        "Copy the command"
    ),
    shortcut!(
        "home",
        keybinds_nav::TableFirst,
        Some("KeybindsTable"),
        KEYBINDS,
        "First row"
    ),
    shortcut!(
        "end",
        keybinds_nav::TableLast,
        Some("KeybindsTable"),
        KEYBINDS,
        "Last row"
    ),
    shortcut!(
        "pageup",
        keybinds_nav::TablePageUp,
        Some("KeybindsTable"),
        KEYBINDS,
        "Page up"
    ),
    shortcut!(
        "pagedown",
        keybinds_nav::TablePageDown,
        Some("KeybindsTable"),
        KEYBINDS,
        "Page down"
    ),
    shortcut!(
        "escape",
        keybinds_nav::FocusSearch,
        Some("KeybindsTable > DataTable"),
        KEYBINDS,
        "Back to the search box"
    ),
    // Flows page
    shortcut!(
        "ctrl-f",
        flows_nav::FocusSearch,
        Some("FlowsPage"),
        FLOWS,
        "Search flows"
    ),
    shortcut!(
        "ctrl-shift-n",
        flows_nav::NewFlow,
        Some("FlowsPage"),
        FLOWS,
        "Create a flow"
    ),
    shortcut!(
        "down",
        flows_nav::FocusGrid,
        Some("FlowsSearch"),
        FLOWS,
        "From the search box to the flows"
    ),
    shortcut!(
        "down",
        flows_nav::FocusGrid,
        Some("FlowsSearch > Input"),
        FLOWS,
        "From the search box to the flows"
    ),
    shortcut!(
        "escape",
        flows_nav::ClearSearch,
        Some("FlowsSearch > Input"),
        FLOWS,
        "Clear the search"
    ),
    shortcut!(
        "left",
        flows_nav::GridLeft,
        Some("FlowsGrid"),
        FLOWS,
        "Previous flow"
    ),
    shortcut!(
        "right",
        flows_nav::GridRight,
        Some("FlowsGrid"),
        FLOWS,
        "Next flow"
    ),
    shortcut!(
        "up",
        flows_nav::GridUp,
        Some("FlowsGrid"),
        FLOWS,
        "Flow above"
    ),
    shortcut!(
        "down",
        flows_nav::GridDown,
        Some("FlowsGrid"),
        FLOWS,
        "Flow below"
    ),
    shortcut!(
        "home",
        flows_nav::GridFirst,
        Some("FlowsGrid"),
        FLOWS,
        "First flow"
    ),
    shortcut!(
        "end",
        flows_nav::GridLast,
        Some("FlowsGrid"),
        FLOWS,
        "Last flow"
    ),
    shortcut!(
        "enter",
        flows_nav::EditSelected,
        Some("FlowsGrid"),
        FLOWS,
        "Edit the selected flow"
    ),
    shortcut!(
        "ctrl-enter",
        flows_nav::RunSelected,
        Some("FlowsGrid"),
        FLOWS,
        "Run the selected flow"
    ),
    shortcut!(
        "ctrl-d",
        flows_nav::DuplicateSelected,
        Some("FlowsGrid"),
        FLOWS,
        "Duplicate the selected flow"
    ),
    shortcut!(
        "delete",
        flows_nav::DeleteSelected,
        Some("FlowsGrid"),
        FLOWS,
        "Delete the selected flow"
    ),
    // Flow editor
    shortcut!(
        "escape",
        app_menu::NavigateBack,
        Some("FlowEditPage"),
        FLOWS,
        "Back to Flows"
    ),
    shortcut!(
        "alt-left",
        app_menu::NavigateBack,
        Some("FlowEditPage"),
        FLOWS,
        "Back to Flows"
    ),
    shortcut!(
        "ctrl-s",
        flow_edit_nav::Save,
        Some("FlowEditPage"),
        FLOWS,
        "Save the flow"
    ),
    shortcut!(
        "ctrl-enter",
        flow_edit_nav::Run,
        Some("FlowEditPage"),
        FLOWS,
        "Run the flow"
    ),
    shortcut!(
        "ctrl-shift-n",
        flow_edit_nav::AddStep,
        Some("FlowEditPage"),
        FLOWS,
        "Add a step"
    ),
    shortcut!(
        "up",
        flow_edit_nav::StepUp,
        Some("FlowSteps"),
        FLOWS,
        "Previous step"
    ),
    shortcut!(
        "down",
        flow_edit_nav::StepDown,
        Some("FlowSteps"),
        FLOWS,
        "Next step"
    ),
    shortcut!(
        "home",
        flow_edit_nav::StepFirst,
        Some("FlowSteps"),
        FLOWS,
        "First step"
    ),
    shortcut!(
        "end",
        flow_edit_nav::StepLast,
        Some("FlowSteps"),
        FLOWS,
        "Last step"
    ),
    shortcut!(
        "enter",
        flow_edit_nav::EditStep,
        Some("FlowSteps"),
        FLOWS,
        "Edit the selected step"
    ),
    shortcut!(
        "delete",
        flow_edit_nav::RemoveStep,
        Some("FlowSteps"),
        FLOWS,
        "Remove the selected step"
    ),
    shortcut!(
        "alt-up",
        flow_edit_nav::MoveStepUp,
        Some("FlowSteps"),
        FLOWS,
        "Move the step up"
    ),
    shortcut!(
        "alt-down",
        flow_edit_nav::MoveStepDown,
        Some("FlowSteps"),
        FLOWS,
        "Move the step down"
    ),
    shortcut!(
        "space",
        flow_edit_nav::ToggleStep,
        Some("FlowSteps"),
        FLOWS,
        "Turn the step on or off"
    ),
    shortcut!(
        "ctrl-d",
        flow_edit_nav::DuplicateStep,
        Some("FlowSteps"),
        FLOWS,
        "Duplicate the step"
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
    let mut bindings: Vec<KeyBinding> = SHORTCUTS.iter().map(Shortcut::key_binding).collect();
    // The library dialog binds Enter to `Confirm`, which would close the
    // dialog before a focused button receives its keyboard click. `NoAction`
    // outranks that binding inside the body without consuming the key, so
    // the click still happens; controls with their own Enter binding
    // (`Input`, `Select`) sit deeper and are unaffected.
    bindings.push(KeyBinding::new(
        "enter",
        gpui::NoAction,
        Some(focus::DIALOG_BODY_CONTEXT),
    ));
    bindings
}

/// One help row: a label and every key that triggers it.
pub type HelpRow = (&'static str, Vec<&'static str>);
/// A help group: its title and rows.
pub type HelpGroup = (&'static str, Vec<HelpRow>);

/// Shortcuts grouped for the help dialog, in table order: one row per
/// label with every key that triggers it. Bindings that only exist to
/// override a component's own key (`Parent > Input`) and the `shift-`
/// spelling of symbol keys are folded into the plain entry.
pub fn help_rows() -> Vec<HelpGroup> {
    let mut groups: Vec<HelpGroup> = Vec::new();
    for shortcut in SHORTCUTS {
        if shortcut.context.is_some_and(|c| c.contains('>')) || shortcut.keys.starts_with("shift-?")
        {
            continue;
        }
        let rows = match groups.iter_mut().find(|(name, _)| *name == shortcut.group) {
            Some((_, rows)) => rows,
            None => {
                groups.push((shortcut.group, Vec::new()));
                &mut groups.last_mut().unwrap().1
            }
        };
        match rows.iter_mut().find(|(label, _)| *label == shortcut.label) {
            Some((_, keys)) => {
                if !keys.contains(&shortcut.keys) {
                    keys.push(shortcut.keys);
                }
            }
            None => rows.push((shortcut.label, vec![shortcut.keys])),
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
    fn help_rows_merge_keys_by_label() {
        let groups = help_rows();
        let (_, global) = groups.iter().find(|(g, _)| *g == GLOBAL).unwrap();
        let (_, keys) = global
            .iter()
            .find(|(label, _)| *label == "Show keyboard shortcuts")
            .unwrap();
        assert_eq!(keys, &vec!["ctrl-/", "?"]);
        assert!(groups.iter().all(|(_, rows)| !rows.is_empty()));
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
