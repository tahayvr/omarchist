use gpui::{App, AppContext, WindowOptions};
use gpui_component::{Root, Theme, ThemeMode, ThemeSet, TitleBar};
use omarchist::cli::{CliArgs, ViewOption};
use omarchist::system::config::config_setup;
use omarchist::system::config::hypr_setup;
use omarchist::system::instance;
use omarchist::system::omarchy_paths;
use omarchist::system::ui_theme_watcher;
use omarchist::ui::app_events::{self, AppEvent, AppEvents};
use omarchist::ui::app_view::ActivePage;
use omarchist::ui::menu::app_menu;
use omarchist::{CombinedAssets, MainTitleBar, MainWindowView, OmarchyUpdates};
use std::process::ExitCode;
use std::rc::Rc;

/// The page to open: `--view` wins, then the Settings page's startup page
/// (or the page shown last), then Themes.
fn cli_args_to_active_page(args: &CliArgs, settings: &config_setup::SettingsSchema) -> ActivePage {
    match args.view {
        Some(ViewOption::Config) => ActivePage::Configuration,
        Some(ViewOption::Keybinds) => ActivePage::Keybinds,
        Some(ViewOption::Flows) => ActivePage::Flows,
        Some(ViewOption::Settings) => ActivePage::Settings,
        Some(ViewOption::About) => ActivePage::About,
        Some(ViewOption::Omarchy) => ActivePage::Omarchy,
        Some(ViewOption::Themes) => {
            if let Some(ref theme_name) = args.theme {
                ActivePage::ThemeEdit(theme_name.clone())
            } else {
                ActivePage::Themes
            }
        }
        None => {
            let name = match settings.settings.startup_page.as_str() {
                "last" => settings.metadata.last_page.as_deref().unwrap_or("themes"),
                name => name,
            };
            ActivePage::from_view_name(name).unwrap_or(ActivePage::Themes)
        }
    }
}

/// Saves the Settings page's "Look" and applies it: a forced light or dark
/// mode, or `omarchy` to follow the desktop theme again.
fn set_theme_mode(mode: &str, cx: &mut App) {
    let mode = mode.to_string();
    if let Err(e) = config_setup::update_settings(move |s| s.theme_mode = mode.clone()) {
        eprintln!("Failed to save the appearance setting: {e}");
    }
    ui_theme_watcher::load_and_apply_omarchy_theme(cx);
    cx.refresh_windows();
}

/// Answers other launches over the instance socket: brings the window
/// forward and navigates to the requested page.
fn serve_open_requests(listener: std::os::unix::net::UnixListener, cx: &mut App) {
    let listener = std::sync::Arc::new(listener);
    cx.spawn(async move |cx| {
        loop {
            let accepting = listener.clone();
            let request = match cx
                .background_spawn(async move { instance::accept(&accepting) })
                .await
            {
                instance::Accepted::Request(request) => request,
                // A peer that sent nothing usable must not stop the server,
                // or every later launch opens another window.
                instance::Accepted::Rejected => continue,
                instance::Accepted::Gone => break,
            };
            let page = match (request.view.as_deref(), request.theme) {
                (Some("themes"), Some(theme)) => Some(ActivePage::ThemeEdit(theme)),
                (Some(view), _) => ActivePage::from_view_name(view),
                (None, _) => None,
            };
            cx.update(|cx| {
                if let Some(page) = page {
                    app_events::emit(cx, AppEvent::Navigate(page));
                }
                for window in cx.windows() {
                    let _ = window.update(cx, |_, window, _| window.activate_window());
                }
            });
        }
    })
    .detach();
}

const THEME_FILE: &str = include_str!("../ui_themes/theme.json");

fn apply_embedded_themes(cx: &mut App) {
    let theme_set: ThemeSet = match serde_json::from_str(THEME_FILE) {
        Ok(theme_set) => theme_set,
        Err(err) => {
            eprintln!("Failed to parse Omarchist theme JSON: {}", err);
            return;
        }
    };

    let mut light_theme = None;
    let mut dark_theme = None;

    for theme in theme_set.themes {
        if theme.mode.is_dark() {
            dark_theme = Some(theme);
        } else {
            light_theme = Some(theme);
        }
    }

    if let Some(theme) = light_theme {
        Theme::global_mut(cx).light_theme = Rc::new(theme);
    }
    if let Some(theme) = dark_theme {
        Theme::global_mut(cx).dark_theme = Rc::new(theme);
    }

    // Default to dark mode as fallback when omarchy theme is unavailable.
    Theme::change(ThemeMode::Dark, None, cx);
}

fn load_custom_fonts(cx: &mut App) {
    let font_data = match cx
        .asset_source()
        .load("fonts/JetBrainsMonoNerdFontMono-Regular.ttf")
    {
        Ok(Some(data)) => data,
        Ok(None) => {
            eprintln!("Font file not found in assets");
            return;
        }
        Err(err) => {
            eprintln!("Failed to load font: {}", err);
            return;
        }
    };

    if let Err(err) = cx.text_system().add_fonts(vec![font_data]) {
        eprintln!("Failed to add font: {}", err);
    }
}

fn main() -> ExitCode {
    let cli_args = CliArgs::parse_args();

    // Subcommands such as `omarchist flow run` never open the window.
    if let Some(command) = &cli_args.command {
        return omarchist::cli::run_command(command);
    }

    // A running window takes the request instead of a second window opening.
    let request = instance::OpenRequest {
        view: cli_args.view.map(|view| view.name().to_string()),
        theme: cli_args.theme.clone(),
    };
    if instance::forward(&request) {
        return ExitCode::SUCCESS;
    }
    // Own the socket before the window exists, so two launches in the same
    // instant cannot both open a window: the one that loses the bind hands
    // its request to the winner.
    let listener = match instance::listen() {
        instance::Listen::Bound(listener) => Some(listener),
        instance::Listen::Taken => {
            if instance::forward(&request) {
                return ExitCode::SUCCESS;
            }
            None
        }
        instance::Listen::Unavailable => None,
    };

    let app = gpui_platform::application().with_assets(CombinedAssets::new());

    app.run(move |cx| {
        if let Err(e) = config_setup::ensure_config() {
            eprintln!("Failed to initialize config: {}", e);
        }
        let settings = config_setup::read_settings().unwrap_or_else(|e| {
            eprintln!("Failed to read settings: {e}");
            config_setup::SettingsSchema {
                version: String::new(),
                settings: config_setup::SettingsConfig::default(),
                metadata: config_setup::Metadata::default(),
            }
        });
        let initial_page = cli_args_to_active_page(&cli_args, &settings);

        // On anything but Quattro the Hyprland hook and the bar plugin
        // would only damage a config they do not understand; the window
        // opens and says so instead.
        let quattro = omarchy_paths::is_quattro_installed();
        if quattro {
            match hypr_setup::ensure_hypr_source() {
                Ok(true) => println!("Added the omarchist require line to hyprland.lua"),
                Ok(false) => {}
                Err(e) => eprintln!("Failed to set up Hyprland config: {}", e),
            }
        }

        cx.set_global(AppEvents::default());
        if let Some(listener) = listener {
            serve_open_requests(listener, cx);
        }
        if settings.settings.bar_widget && quattro {
            std::thread::spawn(|| {
                if let Err(e) = omarchist::system::bar_widget::ensure_current() {
                    eprintln!("Failed to refresh the bar widget: {e}");
                }
            });
        }
        // Launcher entries and startup hooks name the binary; keep them
        // pointing at this one.
        std::thread::spawn(|| {
            if let Ok(flows) = omarchist::system::flows::store::load_flows()
                && let Err(e) = omarchist::system::flows::launcher::refresh_all(&flows)
            {
                eprintln!("Failed to refresh the flow launcher entries: {e}");
            }
        });
        gpui_component::init(cx);
        load_custom_fonts(cx);
        apply_embedded_themes(cx);
        ui_theme_watcher::load_and_apply_omarchy_theme(cx);
        ui_theme_watcher::spawn_ui_theme_watcher(cx);

        // The menu's light/dark switch is the Settings page's "Look"; the
        // page re-reads the file when shown, so the two never disagree.
        cx.on_action(|_: &app_menu::SwitchToLight, cx: &mut App| {
            set_theme_mode("light", cx);
        });
        cx.on_action(|_: &app_menu::SwitchToDark, cx: &mut App| {
            set_theme_mode("dark", cx);
        });
        cx.on_action(|_: &app_menu::FollowOmarchy, cx: &mut App| {
            set_theme_mode("omarchy", cx);
        });
        cx.on_action(|_: &app_menu::Quit, cx: &mut App| {
            cx.quit();
        });
        cx.on_action(|action: &app_menu::SelectFont, cx: &mut App| {
            let font_size_str = match action.0 {
                14 => "small",
                16 => "medium",
                18 => "large",
                _ => "medium",
            };
            if let Err(e) = config_setup::update_font_size(font_size_str) {
                eprintln!("Failed to save font size setting: {}", e);
            }
            ui_theme_watcher::apply_font_size(cx);
            cx.refresh_windows();
        });
        cx.on_action(|_: &app_menu::ToggleSidebar, cx: &mut App| {
            app_events::emit(cx, AppEvent::ToggleSidebar);
        });

        // Never leave Hyprland stuck in the keystroke-recording submap: a
        // previous instance may have died while recording, a panic must
        // leave it before the abort, and quitting leaves it too. The
        // socket is removed on quit so the next launch opens its own window.
        omarchist::system::keybinds::submap::install_panic_hook();
        std::thread::spawn(omarchist::system::keybinds::submap::reset_on_startup);
        cx.on_app_quit(|_cx| {
            omarchist::system::keybinds::submap::leave_recording_submap();
            instance::remove_socket();
            async {}
        })
        .detach();

        // Every app shortcut comes from the shortcuts table.
        cx.bind_keys(omarchist::ui::shortcuts::key_bindings());

        cx.spawn(async move |cx| {
            let window_options = WindowOptions {
                titlebar: Some(TitleBar::title_bar_options()),
                focus: true,
                show: true,
                // The headless test harness runs a second, sandboxed
                // instance under a class of its own, so its window rules
                // and keys never reach the user's window.
                app_id: Some(
                    std::env::var("OMARCHIST_APP_ID").unwrap_or_else(|_| "omarchist".into()),
                ),
                ..Default::default()
            };
            let window_handle = cx.open_window(window_options, |window, cx| {
                let title_bar = cx.new(MainTitleBar::new);
                OmarchyUpdates::start_periodic(title_bar.read(cx).updates().clone(), cx);
                let main_view =
                    cx.new(|cx| MainWindowView::new(title_bar, initial_page.clone(), window, cx));
                cx.new(|cx| Root::new(main_view, window, cx))
            })?;

            window_handle.update(cx, |_view, window, _cx| {
                window.activate_window();
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
    ExitCode::SUCCESS
}
