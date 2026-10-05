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
use omarchist::system::flows::catalog;
use omarchist::system::flows::{Flow, Step, StepKind};
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
        // Every test calls this first, so the variables are set before any
        // thread of the binary reads them. The data directory is named by
        // its own variable, which a desktop session exports as a path into
        // the real home.
        unsafe {
            std::env::set_var("HOME", &dir);
            std::env::set_var("XDG_DATA_HOME", dir.join(".local/share"));
            std::env::set_var("XDG_CONFIG_HOME", dir.join(".config"));
            std::env::set_var("XDG_STATE_HOME", dir.join(".local/state"));
            std::env::set_var("XDG_CACHE_HOME", dir.join(".cache"));
            // The gallery is a folder in the test home, signed with a key
            // made for the tests: no test reaches the network.
            std::env::set_var(
                "OMARCHIST_CATALOG_URL",
                format!("file://{}", dir.join("catalog/out").display()),
            );
            std::env::set_var("OMARCHIST_CATALOG_KEY", GALLERY_PUBLIC_KEY);
        }
        dir
    })
}

/// A signing key made with openssl for the tests only.
const GALLERY_KEY: &str = "-----BEGIN PRIVATE KEY-----\n\
    MC4CAQAwBQYDK2VwBCIEIPscK1MVSSz6HWpY5OpocH/dNRNHrxyJ5ZC4Ip7uQRSr\n\
    -----END PRIVATE KEY-----\n";
const GALLERY_PUBLIC_KEY: &str = "K0NttSSLCqz20brnxyb65tQZqYgyikGooBUKRqoeNxs=";

/// A flow as the gallery's repository holds it.
fn gallery_flow(
    name: &str,
    author: &str,
    version: u32,
    category: &str,
    steps: Vec<Step>,
) -> String {
    let mut flow = Flow::new(String::new(), name.to_string());
    flow.description = format!("{name}, a flow for the tests.");
    flow.meta.author = author.to_string();
    flow.meta.version = version.to_string();
    flow.meta.category = category.to_string();
    flow.meta.license = catalog::LICENSE.to_string();
    flow.steps = steps;
    flow.to_toml().expect("a gallery flow")
}

/// Publishes the tests' gallery, once per test binary, and keeps a copy
/// as a visit to the Gallery page would. Three flows: `pause` (version 2,
/// by the verified ada, featured, the most installed), `breathe`, and
/// `careful`, which runs a command as administrator and is never run.
pub fn gallery() -> &'static catalog::Index {
    static GALLERY: OnceLock<catalog::Index> = OnceLock::new();
    GALLERY.get_or_init(|| {
        let repo = home().join("catalog/repo");
        let out = home().join("catalog/out");
        std::fs::create_dir_all(repo.join("flows")).expect("gallery repo");
        let wait = |ms: u64| Step::new(StepKind::Wait { ms });
        let write = |slug: &str, text: String| {
            std::fs::write(repo.join("flows").join(format!("{slug}.flow.toml")), text)
                .expect("a gallery flow file");
        };
        write(
            "pause",
            gallery_flow(
                "Pause",
                "ada",
                2,
                "Focus",
                vec![wait(500), Step::new(StepKind::notify("Back", "Go on"))],
            ),
        );
        write(
            "breathe",
            gallery_flow("Breathe", "grace", 1, "Media", vec![wait(900)]),
        );
        write(
            "careful",
            gallery_flow(
                "Careful",
                "grace",
                1,
                "System",
                vec![Step::new(StepKind::Exec {
                    command: "sudo -n true".into(),
                    wait: true,
                })],
            ),
        );
        std::fs::write(repo.join("featured.txt"), "pause\n").expect("featured");
        std::fs::write(repo.join("verified.txt"), "ada\n").expect("verified");
        let index = catalog::build(&repo, &out, None, "2026-10-05", 100).expect("the gallery");
        catalog::write_index(&out, &index, Some(GALLERY_KEY)).expect("the signed index");
        std::fs::write(
            out.join("v1/installs.json"),
            r#"{"pause": 1200, "breathe": 3}"#,
        )
        .expect("install counts");
        catalog::load().expect("the gallery loads").index
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
    // Until a frame leaves nothing waiting for the next one; a frame is the
    // costly part of a test, so no more of them than that.
    for _ in 0..6 {
        cx.run_until_parked();
        let delivered = with(cx, handle, |window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx)
        });
        if delivered == 0 {
            break;
        }
    }
    cx.run_until_parked();
    with(cx, handle, |window, cx| window.render_frame(cx));
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
