// Headless keyboard-navigation tests: the production `Root` and
// `MainWindowView` in a headless window, driven through gpui-kit's
// `TestWindowExt`.
mod common;

use std::time::Duration;

use common::{assert_page, open, settle, with};
use gpui_kit::component::WindowExt;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, TestAppContext};
use omarchist::ActivePage;

/// The sidebar page list's test target (`SidebarNav` in `app_view.rs`).
const SIDEBAR: &str = "sidebar-nav";
/// Dialog bodies' test targets (`dialog_body` in `focus.rs`).
const SHORTCUTS_DIALOG: &str = "shortcuts-dialog";
const CREATE_THEME_DIALOG: &str = "create-theme-dialog";

#[gpui_kit::test]
fn ctrl_number_switches_pages(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::Themes);

    cx.update_window(handle.into(), |_, window, cx| window.press("ctrl-2", cx))
        .unwrap();
    cx.run_until_parked();
    assert_page(cx, &view, ActivePage::Configuration);

    cx.update_window(handle.into(), |_, window, cx| window.press("ctrl-3", cx))
        .unwrap();
    cx.run_until_parked();
    assert_page(cx, &view, ActivePage::Keybinds);
}

#[gpui_kit::test]
fn sidebar_arrows_and_enter_navigate(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::Themes);

    cx.update_window(handle.into(), |_, window, cx| {
        let sidebar = window.find(SIDEBAR);
        assert!(sidebar.visible());
        assert_eq!(
            sidebar.focused(),
            Some(true),
            "the sidebar has focus at startup"
        );

        window.press("down", cx);
    })
    .unwrap();
    cx.update(|cx| assert_eq!(view.read(cx).sidebar_index(), 1));

    cx.update_window(handle.into(), |_, window, cx| window.press("enter", cx))
        .unwrap();
    cx.run_until_parked();
    assert_page(cx, &view, ActivePage::Configuration);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find(SIDEBAR).focused(),
            Some(false),
            "Enter moves focus onto the page"
        );

        // Escape from the page returns to the sidebar; End jumps to its last entry.
        window.press("escape", cx);
        assert_eq!(window.find(SIDEBAR).focused(), Some(true));
        window.press("end", cx);
    })
    .unwrap();
    cx.update(|cx| assert_eq!(view.read(cx).sidebar_index(), 3));
}

#[gpui_kit::test]
async fn shortcuts_dialog_opens_focused_and_closes_on_escape(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::Themes);

    cx.update_window(handle.into(), |_, window, cx| {
        window.press("ctrl-/", cx);
        let dialog = window.find(SHORTCUTS_DIALOG);
        assert!(dialog.visible(), "Ctrl+/ opens the shortcuts dialog");
        assert_eq!(
            window.find("shortcuts-list-content").focused(),
            Some(true),
            "focus lands on the dialog's list, so the arrow keys scroll it"
        );
        assert_eq!(window.find(SIDEBAR).focused(), Some(false));
        window.press("escape", cx);
    })
    .unwrap();

    cx.wait_for(handle.into(), Duration::from_secs(2), |window, cx| {
        window.try_find(SHORTCUTS_DIALOG).is_none() && !window.has_active_dialog(cx)
    })
    .await;
    assert_page(cx, &view, ActivePage::Themes);
}

/// Arrow and page keys scroll a scroll area while focus is inside it.
#[gpui_kit::test]
fn page_down_scrolls_the_shortcuts_list(cx: &mut TestAppContext) {
    let (handle, _view) = open(cx, ActivePage::Themes);

    cx.update_window(handle.into(), |_, window, cx| window.press("ctrl-/", cx))
        .unwrap();
    // The dialog moves focus onto its first stop on the next frame.
    cx.update_window(handle.into(), |_, window, cx| window.render_frame(cx))
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let content = window.find("shortcuts-list-content");
        assert_eq!(content.focused(), Some(true), "the list takes focus");
        let before = content.bounds().origin.y;

        window.press("pagedown", cx);
        window.render_frame(cx);
        let after_page = window.find("shortcuts-list-content").bounds().origin.y;
        assert!(
            after_page < before,
            "Page Down scrolls: {before:?} -> {after_page:?}"
        );

        window.press("up", cx);
        window.render_frame(cx);
        let after_up = window.find("shortcuts-list-content").bounds().origin.y;
        assert!(
            after_up > after_page,
            "Up scrolls back: {after_page:?} -> {after_up:?}"
        );
    })
    .unwrap();
}

#[gpui_kit::test]
fn dialog_traps_tab_inside_its_controls(cx: &mut TestAppContext) {
    let (handle, _view) = open(cx, ActivePage::Themes);

    cx.update_window(handle.into(), |_, window, cx| {
        window.press("ctrl-n", cx);
        let body = window.find(CREATE_THEME_DIALOG);
        assert!(body.visible(), "Ctrl+N opens the create-theme dialog");
        assert_eq!(
            body.focused(),
            Some(true),
            "focus lands on its first control"
        );

        // The dialog's close button is outside the body but inside the trap.
        let mut left_body = false;
        let mut returned_to_body = false;
        for _ in 0..8 {
            window.press("tab", cx);
            assert_eq!(
                window.find(SIDEBAR).focused(),
                Some(false),
                "Tab never escapes to the page behind the dialog"
            );
            match window.find(CREATE_THEME_DIALOG).focused() {
                Some(true) if left_body => returned_to_body = true,
                Some(false) => left_body = true,
                _ => {}
            }
        }
        assert!(
            left_body && returned_to_body,
            "Tab wraps around inside the dialog"
        );
        window.close_all_dialogs(cx);
    })
    .unwrap();
}

#[gpui_kit::test]
async fn command_palette_runs_the_chosen_command(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::Themes);

    cx.update_window(handle.into(), |_, window, cx| {
        window.press("ctrl-shift-p", cx);
        assert!(
            window.has_active_dialog(cx),
            "Ctrl+Shift+P opens the palette"
        );
        window.render_frame(cx);
        window.input("keybinds", cx);
        window.press("enter", cx);
    })
    .unwrap();

    cx.wait_for(handle.into(), Duration::from_secs(2), |window, cx| {
        !window.has_active_dialog(cx)
    })
    .await;
    cx.run_until_parked();
    assert_page(cx, &view, ActivePage::Keybinds);
}

#[gpui_kit::test]
async fn theme_designer_refuses_themes_omarchist_did_not_create(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::Themes);
    cx.update_window(handle.into(), |_, window, cx| {
        view.update(cx, |view, cx| {
            view.navigate_to(
                ActivePage::ThemeEdit("not-an-omarchist-theme".into()),
                window,
                cx,
            )
        });
    })
    .unwrap();
    cx.run_until_parked();
    assert_page(cx, &view, ActivePage::Themes);
    // The refusal is explained with a toast, which only exists on screen
    // when the main view renders the notification layer.
    cx.wait_for(handle.into(), Duration::from_secs(2), |window, _| {
        window
            .try_find("notification")
            .is_some_and(|toast| toast.visible())
    })
    .await;
}

#[gpui_kit::test]
fn the_theme_is_renamed_from_its_title(cx: &mut TestAppContext) {
    omarchist::system::themes::theme_management::create_theme_from_defaults("ui-rename")
        .expect("a theme in the test home");
    let (handle, view) = open(cx, ActivePage::ThemeEdit("ui-rename".into()));
    with(cx, handle, |window, cx| window.click("theme-title", cx));
    settle(cx, handle);
    // The whole name is selected, so typing replaces it; Enter keeps it.
    with(cx, handle, |window, cx| {
        assert_eq!(window.find("theme-name").focused(), Some(true));
        window.input("UI Renamed", cx);
        window.press("enter", cx);
    });
    settle(cx, handle);
    // The folder moved to the slug, and the page reopened under it.
    assert_page(cx, &view, ActivePage::ThemeEdit("ui-renamed".into()));
    let themes = common::home().join(".config/omarchy/themes");
    assert!(themes.join("ui-renamed/omarchist.json").exists());
    assert!(!themes.join("ui-rename").exists());
    with(cx, handle, |window, _| {
        assert!(window.try_find("theme-name").is_none());
        assert!(window.find("theme-title").visible());
    });
}

#[gpui_kit::test]
async fn notifications_are_shown(cx: &mut TestAppContext) {
    let (handle, _) = open(cx, ActivePage::Themes);
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(window.try_find("notification").is_none());
        window.push_notification("Saved 'test'", cx);
    })
    .unwrap();
    cx.wait_for(handle.into(), Duration::from_secs(2), |window, _| {
        window
            .try_find("notification")
            .is_some_and(|toast| toast.visible())
    })
    .await;
}

#[gpui_kit::test]
fn about_page_text_can_be_selected(cx: &mut TestAppContext) {
    let (handle, _) = open(cx, ActivePage::About);

    cx.update_window(handle.into(), |_, window, cx| {
        window.double_click("omarchist-version", cx);
        gpui_kit::base::TextSelection::selected_text(window, cx)
    })
    .map(|selected| {
        // A double click selects one word, and the dots of the version split
        // it into several.
        assert!(!selected.is_empty());
        assert!(concat!("v", env!("CARGO_PKG_VERSION")).contains(&selected));
    })
    .unwrap();
}
