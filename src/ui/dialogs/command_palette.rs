// The command palette: every app-wide command in one searchable list, with
// shortcuts looked up from the keymap.
//
// Items carry no `CommandItem::action`. The palette dispatches that from
// inside the dialog, which is outside the main view's element path, so
// `on_confirm` closes the dialog and dispatches through the main window's
// focus handle instead.
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

/// The palette's contents; shortcut hints come from the keymap.
fn groups() -> Vec<Group> {
    vec![
        Group {
            title: "Go to",
            entries: vec![
                entry("Themes", &["page", "gallery"], app_menu::NavigateToThemes),
                entry(
                    "Configuration",
                    &["page", "hyprland", "settings"],
                    app_menu::NavigateToConfig,
                ),
                entry(
                    "Keybinds",
                    &["page", "shortcuts", "hyprland"],
                    app_menu::NavigateToKeybinds,
                ),
                entry(
                    "Omarchy",
                    &["page", "update", "release notes"],
                    app_menu::NavigateToOmarchy,
                ),
                entry("Settings", &["page", "font"], app_menu::NavigateToSettings),
                entry("About", &["page", "version"], app_menu::NavigateToAbout),
            ],
        },
        Group {
            title: "Themes",
            entries: vec![
                entry("Create a new theme", &["new", "add"], app_menu::NewTheme),
                entry(
                    "Re-apply the Omarchy theme",
                    &["refresh", "reload", "colors"],
                    app_menu::RefreshTheme,
                ),
            ],
        },
        Group {
            title: "View",
            entries: vec![
                entry("Reload the current page", &["refresh"], focus::ReloadPage),
                entry(
                    "Toggle sidebar",
                    &["collapse", "expand"],
                    app_menu::ToggleSidebar,
                ),
                entry(
                    "Light appearance",
                    &["theme", "mode"],
                    app_menu::SwitchToLight,
                ),
                entry(
                    "Dark appearance",
                    &["theme", "mode"],
                    app_menu::SwitchToDark,
                ),
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
    window.open_dialog(cx, move |dialog, _, _| {
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
            .w(px(560.))
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
