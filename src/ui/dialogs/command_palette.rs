// The command palette: every app-wide command in one searchable list, with
// shortcuts looked up from the keymap.
//
// Items carry no `CommandItem::action`: that would dispatch while the
// dialog is still open, and the main view ignores page commands then. So
// `on_confirm` closes the dialog first and dispatches through the main
// window's focus handle.
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    IndexPath, WindowExt,
    command::{Command, CommandGroup, CommandItem, CommandState},
    h_flex,
    kbd::Kbd,
};

use crate::ui::focus;
use crate::ui::menu::app_menu;

struct Entry {
    label: &'static str,
    keywords: &'static [&'static str],
    action: Box<dyn Action>,
}

struct Group {
    title: &'static str,
    entries: Vec<Entry>,
}

fn entry(label: &'static str, keywords: &'static [&'static str], action: impl Action) -> Entry {
    Entry {
        label,
        keywords,
        action: Box::new(action),
    }
}

/// The palette's contents; shortcut hints come from the keymap. The kit
/// filters the rows and keeps their order, so the first row the query
/// matches is the one Enter runs: a keyword is never a word of another
/// row's label (a test checks), or typing that word would run the wrong
/// command.
fn groups() -> Vec<Group> {
    vec![
        Group {
            title: "Go to",
            entries: vec![
                entry("Themes", &["page"], app_menu::NavigateToThemes),
                entry(
                    "Configuration",
                    &["page", "hyprland"],
                    app_menu::NavigateToConfig,
                ),
                entry(
                    "Keybinds",
                    &["page", "hyprland", "binds"],
                    app_menu::NavigateToKeybinds,
                ),
                entry(
                    "Flows",
                    &["page", "automation", "actions"],
                    app_menu::NavigateToFlows,
                ),
                entry(
                    "Omarchy",
                    &["page", "update", "release notes"],
                    app_menu::NavigateToOmarchy,
                ),
                entry("Settings", &["page"], app_menu::NavigateToSettings),
                entry("About", &["page", "version"], app_menu::NavigateToAbout),
            ],
        },
        Group {
            title: "Themes",
            entries: vec![
                entry("New theme", &["create"], app_menu::NewTheme),
                entry(
                    "Re-apply theme",
                    &["refresh", "colors"],
                    app_menu::RefreshTheme,
                ),
            ],
        },
        Group {
            title: "Keybinds",
            entries: vec![entry(
                "Add keybind",
                &["create", "bind", "shortcut"],
                app_menu::NewKeybind,
            )],
        },
        Group {
            title: "Flows",
            entries: vec![
                entry("New flow", &["create"], app_menu::NewFlow),
                entry(
                    "New from template",
                    &["create", "templates"],
                    app_menu::NewFlowFromTemplate,
                ),
                entry(
                    "Catalog",
                    &["browse", "shared", "catalog", "community", "download"],
                    app_menu::OpenCatalog,
                ),
                entry(
                    "Import flow",
                    &["open", "file", "share"],
                    app_menu::ImportFlow,
                ),
            ],
        },
        Group {
            title: "View",
            entries: vec![
                entry("Reload", &["refresh", "rescan"], focus::ReloadPage),
                entry(
                    "Toggle sidebar",
                    &["collapse", "expand"],
                    app_menu::ToggleSidebar,
                ),
                entry(
                    "Follow Omarchy's appearance",
                    &["look", "mode", "auto"],
                    app_menu::FollowOmarchy,
                ),
                entry(
                    "Light appearance",
                    &["look", "mode"],
                    app_menu::SwitchToLight,
                ),
                entry("Dark appearance", &["look", "mode"], app_menu::SwitchToDark),
                entry(
                    "Font size: Small",
                    &["text", "zoom"],
                    app_menu::SelectFont(14),
                ),
                entry(
                    "Font size: Medium",
                    &["text", "zoom"],
                    app_menu::SelectFont(16),
                ),
                entry(
                    "Font size: Large",
                    &["text", "zoom"],
                    app_menu::SelectFont(18),
                ),
            ],
        },
        Group {
            title: "Help",
            entries: vec![entry(
                "Keyboard shortcuts",
                &["help", "keys"],
                focus::ShowShortcuts,
            )],
        },
        Group {
            title: "Application",
            entries: vec![entry("Quit", &["exit", "close"], app_menu::Quit)],
        },
    ]
}

/// Opens the palette. `target` is the main window's focus handle: confirmed
/// commands are dispatched along its path once the dialog has closed.
pub fn open_command_palette(target: FocusHandle, window: &mut Window, cx: &mut App) {
    let groups = Rc::new(groups());
    let state = cx.new(|cx| CommandState::new(window, cx));

    let dialog_state = state.clone();
    window.open_dialog(cx, move |dialog, window, _| {
        let confirm_groups = groups.clone();
        let target = target.clone();
        let command = groups
            .iter()
            .fold(Command::new(&dialog_state), |command, group| {
                command.group(
                    CommandGroup::new()
                        .label(group.title)
                        .items(group.entries.iter().map(palette_item)),
                )
            })
            .placeholder("Type a command...")
            .bordered(false)
            .on_confirm(move |path: IndexPath, window, cx| {
                window.close_dialog(cx);
                let action = confirm_groups
                    .get(path.section)
                    .and_then(|group| group.entries.get(path.row))
                    .map(|entry| entry.action.boxed_clone());
                if let Some(action) = action {
                    target.dispatch_action(action.as_ref(), window, cx);
                }
            })
            .on_cancel(|window, cx| window.close_dialog(cx));

        dialog
            .w(focus::dialog_width(560., window))
            .margin_top(px(96.))
            .overlay(true)
            .overlay_closable(true)
            .keyboard(true)
            .close_button(false)
            .child(command)
    });

    // Focus now and again after the dialog's first frame, as `focus_first_in` does.
    state.update(cx, |state, cx| state.focus(window, cx));
    window.on_next_frame(move |window, cx| {
        state.update(cx, |state, cx| state.focus(window, cx));
    });
}

/// A palette row: the label and, when bound, its shortcut.
fn palette_item(entry: &Entry) -> CommandItem {
    let label = entry.label;
    let action = entry.action.boxed_clone();
    CommandItem::new()
        .label(label)
        .keywords(entry.keywords.iter().copied())
        .child(move |window, _cx| {
            let kbd = Kbd::binding_for_action(action.as_ref(), None, window);
            h_flex()
                .w_full()
                .gap_4()
                .items_center()
                .justify_between()
                .child(label)
                .when_some(kbd, |this, kbd| this.child(kbd))
        })
}

#[cfg(test)]
mod tests {
    use super::groups;

    /// Typing a command's own name must run that command: with the rows
    /// filtered in order, a keyword that is a word of a later row's label
    /// would put the wrong row first.
    #[test]
    fn no_keyword_is_a_word_of_another_label() {
        let entries: Vec<(&str, &[&str])> = groups()
            .iter()
            .flat_map(|group| group.entries.iter().map(|e| (e.label, e.keywords)))
            .collect();
        for (label, keywords) in &entries {
            for keyword in keywords.iter() {
                for (other, _) in &entries {
                    if other == label {
                        continue;
                    }
                    let shadowed = other
                        .split(|c: char| !c.is_alphanumeric())
                        .any(|word| word.eq_ignore_ascii_case(keyword));
                    assert!(
                        !shadowed,
                        "'{label}' has the keyword '{keyword}', a word of '{other}'"
                    );
                }
            }
        }
    }
}
