// Shared setup for the headless UI tests: the production `Root` and
// `MainWindowView` in a headless window, in a home directory of their own
// so nothing a test saves or runs touches the real one.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use gpui_kit::component::{Root, WindowExt};
use gpui_kit::test::TestWindowExt;
use gpui_kit::{App, AppContext, Entity, TestAppContext, Window, WindowHandle, px, size};
use omarchist::system::flows::Flow;
use omarchist::ui::app_events::AppEvents;
use omarchist::ui::flows_page::FlowEditPage;
use omarchist::{ActivePage, MainTitleBar, MainWindowView};

/// The tests' home directory, one per test binary. `HOME` points at it
/// from the first call on, before any window opens.
pub fn home() -> &'static PathBuf {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("omarchist-ui-tests-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".config/omarchist/flows")).expect("test home");
        // Every test calls this first, so the variable is set before any
        // thread of the binary reads it.
        unsafe { std::env::set_var("HOME", &dir) };
        dir
    })
}

/// Writes a flow file into the test home, as if it had been saved earlier.
pub fn write_flow(id: &str, toml: &str) {
    let path = home()
        .join(".config/omarchist/flows")
        .join(format!("{id}.toml"));
    std::fs::write(path, toml).expect("write the flow");
}

pub fn open(
    cx: &mut TestAppContext,
    page: ActivePage,
) -> (WindowHandle<Root>, Entity<MainWindowView>) {
    home();
    cx.update(|cx| {
        cx.set_global(AppEvents::default());
        gpui_kit::init(cx);
        cx.bind_keys(omarchist::ui::shortcuts::key_bindings());
    });
    let mut main_view = None;
    let handle = cx.open_window(size(px(1280.), px(800.)), |window, cx| {
        let title_bar = cx.new(MainTitleBar::new);
        let view = cx.new(|cx| MainWindowView::new(title_bar, page, window, cx));
        main_view = Some(view.clone());
        Root::new(view, window, cx)
    });
    cx.update_window(handle.into(), |_, window, cx| window.render_frame(cx))
        .unwrap();
    // Startup can raise notifications that depend on the machine (no
    // Omarchy 4 on a CI runner, so its themes cannot be read either). Every
    // test starts without them: let them land, dismiss them, and run the
    // fake clock past the closing animation that removes them.
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.clear_notifications(cx)
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_secs(5));
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| window.render_frame(cx))
        .unwrap();
    (handle, main_view.expect("main view created"))
}

/// Runs `f` against the window, as a test's step.
pub fn with<R>(
    cx: &mut TestAppContext,
    handle: WindowHandle<Root>,
    f: impl FnOnce(&mut Window, &mut App) -> R,
) -> R {
    cx.update_window(handle.into(), |_, window, cx| f(window, cx))
        .unwrap()
}

/// Lets queued work finish and what it changed reach the screen. A test
/// window has no frame loop, so work deferred to the next frame (a dialog
/// taking the keyboard) is delivered here; the tasks it starts need the
/// executor, and the result needs another frame.
pub fn settle(cx: &mut TestAppContext, handle: WindowHandle<Root>) {
    for _ in 0..3 {
        cx.run_until_parked();
        with(cx, handle, |window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
        });
    }
}

/// Waits in real time for something a thread outside the test executor
/// does (a flow run), pumping the executor so its messages are handled.
pub fn wait_real(
    cx: &mut TestAppContext,
    handle: WindowHandle<Root>,
    timeout: Duration,
    mut done: impl FnMut(&mut Window, &mut App) -> bool,
) {
    let deadline = Instant::now() + timeout;
    loop {
        settle(cx, handle);
        if with(cx, handle, |window, cx| done(window, cx)) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out after {timeout:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn assert_page(cx: &mut TestAppContext, view: &Entity<MainWindowView>, page: ActivePage) {
    cx.update(|cx| assert_eq!(*view.read(cx).active_page(), page));
}

/// The flow editor on screen.
pub fn editor(cx: &mut TestAppContext, view: &Entity<MainWindowView>) -> Entity<FlowEditPage> {
    cx.update(|cx| {
        view.read(cx)
            .flow_editor()
            .expect("the flow editor is open")
    })
}

/// The flow as the editor holds it, saved or not.
pub fn edited_flow(cx: &mut TestAppContext, view: &Entity<MainWindowView>) -> Flow {
    let editor = editor(cx, view);
    cx.update(|cx| editor.read(cx).current(cx))
}

/// Opens the Add step dialog and waits for it to take the keyboard.
pub fn open_add_step(cx: &mut TestAppContext, handle: WindowHandle<Root>) {
    with(cx, handle, |window, cx| window.press("ctrl-shift-n", cx));
    settle(cx, handle);
}

/// Types into the search of the Add step dialog and takes the first match.
pub fn pick_step_type(cx: &mut TestAppContext, handle: WindowHandle<Root>, search: &str) {
    with(cx, handle, |window, cx| {
        window.input(search, cx);
        window.press("enter", cx);
    });
    settle(cx, handle);
}
