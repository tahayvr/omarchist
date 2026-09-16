// Headless keyboard-navigation tests: the real main window, driven through
// gpui-kit's test harness with the same key bindings `main.rs` registers.
use std::time::Duration;

use gpui::{AppContext, Entity, TestAppContext, WindowHandle};
use gpui_component::{Root, WindowExt};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use omarchist::ui::app_events::AppEvents;
use omarchist::{ActivePage, MainTitleBar, MainWindowView};

fn open(cx: &mut TestAppContext, page: ActivePage) -> (WindowHandle<Root>, Entity<MainWindowView>) {
    cx.update(|cx| {
        cx.set_global(AppEvents::default());
        gpui_component::init(cx);
        cx.bind_keys(omarchist::ui::shortcuts::key_bindings());
    });
    let mut main_view = None;
    let window = cx.add_window(|window, cx| {
        let title_bar = cx.new(|_| MainTitleBar::new());
        let view = cx.new(|cx| MainWindowView::new(title_bar, page, window, cx));
        main_view = Some(view.clone());
        Root::new(view, window, cx)
    });
    cx.update_window(window.into(), |_, window, cx| window.render_frame(cx))
        .unwrap();
    (window, main_view.expect("main view created"))
}

fn active_page(view: &Entity<MainWindowView>, cx: &TestAppContext) -> ActivePage {
    view.read_with(cx, |view, _| view.active_page().clone())
}

#[gpui::test]
fn ctrl_number_switches_pages(cx: &mut TestAppContext) {
    let (window, view) = open(cx, ActivePage::Themes);

    cx.update_window(window.into(), |_, window, cx| window.press("ctrl-2", cx))
        .unwrap();
    cx.run_until_parked();
    assert_eq!(active_page(&view, cx), ActivePage::Configuration);

    cx.update_window(window.into(), |_, window, cx| window.press("ctrl-3", cx))
        .unwrap();
    cx.run_until_parked();
    assert_eq!(active_page(&view, cx), ActivePage::Keybinds);
}

#[gpui::test]
fn sidebar_arrows_and_enter_navigate(cx: &mut TestAppContext) {
    let (window, view) = open(cx, ActivePage::Themes);
    assert_eq!(view.read_with(cx, |view, _| view.sidebar_index()), 0);

    cx.update_window(window.into(), |_, window, cx| {
        window.press("down", cx);
    })
    .unwrap();
    assert_eq!(view.read_with(cx, |view, _| view.sidebar_index()), 1);

    cx.update_window(window.into(), |_, window, cx| window.press("enter", cx))
        .unwrap();
    cx.run_until_parked();
    assert_eq!(active_page(&view, cx), ActivePage::Configuration);

    // Escape from the page returns to the sidebar; End jumps to the last entry.
    cx.update_window(window.into(), |_, window, cx| {
        window.press("escape", cx);
        window.press("end", cx);
    })
    .unwrap();
    assert_eq!(view.read_with(cx, |view, _| view.sidebar_index()), 2);
}

#[gpui::test]
async fn shortcuts_dialog_traps_tab_and_closes_on_escape(cx: &mut TestAppContext) {
    let (window, view) = open(cx, ActivePage::Themes);

    cx.update_window(window.into(), |_, window, cx| {
        window.press("ctrl-/", cx);
        assert!(
            window.has_active_dialog(cx),
            "Ctrl+/ opens the shortcuts dialog"
        );
        for _ in 0..6 {
            window.press("tab", cx);
        }
        assert!(window.has_active_dialog(cx), "Tab stays inside the dialog");
        window.press("escape", cx);
    })
    .unwrap();

    cx.wait_for(window.into(), Duration::from_secs(2), |window, cx| {
        !window.has_active_dialog(cx)
    })
    .await;
    assert_eq!(active_page(&view, cx), ActivePage::Themes);
}

#[gpui::test]
async fn command_palette_runs_the_chosen_command(cx: &mut TestAppContext) {
    let (window, view) = open(cx, ActivePage::Themes);

    cx.update_window(window.into(), |_, window, cx| {
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

    cx.wait_for(window.into(), Duration::from_secs(2), |window, cx| {
        !window.has_active_dialog(cx)
    })
    .await;
    cx.run_until_parked();
    assert_eq!(active_page(&view, cx), ActivePage::Keybinds);
}
