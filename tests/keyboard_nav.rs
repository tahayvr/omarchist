// Headless keyboard-navigation tests: the production `Root` and
// `MainWindowView` in a headless window, driven through gpui-kit's
// `TestWindowExt`.
use std::time::Duration;

use gpui_kit::component::{Root, WindowExt};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{AppContext, Entity, TestAppContext, WindowHandle, px, size};
use omarchist::ui::app_events::AppEvents;
use omarchist::{ActivePage, MainTitleBar, MainWindowView};

/// The sidebar page list's test target (`SidebarNav` in `app_view.rs`).
const SIDEBAR: &str = "sidebar-nav";
/// Dialog bodies' test targets (`dialog_body` in `focus.rs`).
const SHORTCUTS_DIALOG: &str = "shortcuts-dialog";
const CREATE_THEME_DIALOG: &str = "create-theme-dialog";

fn open(cx: &mut TestAppContext, page: ActivePage) -> (WindowHandle<Root>, Entity<MainWindowView>) {
    cx.update(|cx| {
        cx.set_global(AppEvents::default());
        gpui_kit::init(cx);
        cx.bind_keys(omarchist::ui::shortcuts::key_bindings());
    });
    let mut main_view = None;
    let handle = cx.open_window(size(px(1280.), px(800.)), |window, cx| {
        let title_bar = cx.new(|_| MainTitleBar::new());
        let view = cx.new(|cx| MainWindowView::new(title_bar, page, window, cx));
        main_view = Some(view.clone());
        Root::new(view, window, cx)
    });
    cx.update_window(handle.into(), |_, window, cx| window.render_frame(cx))
        .unwrap();
    (handle, main_view.expect("main view created"))
}

fn assert_page(cx: &mut TestAppContext, view: &Entity<MainWindowView>, page: ActivePage) {
    cx.update(|cx| assert_eq!(*view.read(cx).active_page(), page));
}

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
    cx.update(|cx| assert_eq!(view.read(cx).sidebar_index(), 2));
}

#[gpui_kit::test]
async fn shortcuts_dialog_opens_focused_and_closes_on_escape(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::Themes);

    cx.update_window(handle.into(), |_, window, cx| {
        window.press("ctrl-/", cx);
        let dialog = window.find(SHORTCUTS_DIALOG);
        assert!(dialog.visible(), "Ctrl+/ opens the shortcuts dialog");
        assert_eq!(
            dialog.focused(),
            Some(true),
            "focus lands inside the dialog"
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
