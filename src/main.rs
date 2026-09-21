use gpui::{App, AppContext, KeyBinding, WindowOptions};
use gpui_component::{Root, Theme, ThemeMode, ThemeSet, TitleBar};
use omarchist::cli::{CliArgs, ViewOption};
use omarchist::system::config::config_setup;
use omarchist::system::config::hypr_setup;
use omarchist::system::ui_theme_watcher;
use omarchist::ui::app_events::{self, AppEvent, AppEvents};
use omarchist::ui::app_view::ActivePage;
use omarchist::ui::keybinds_page::keystroke_input;
use omarchist::ui::menu::app_menu;
use omarchist::{CombinedAssets, MainTitleBar, MainWindowView, OmarchyUpdates};
use std::process::ExitCode;
use std::rc::Rc;

fn cli_args_to_active_page(args: &CliArgs) -> ActivePage {
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
        None => ActivePage::Themes, // Default page
    }
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

    let app = gpui_platform::application().with_assets(CombinedAssets::new());

    app.run(move |cx| {
        let initial_page = cli_args_to_active_page(&cli_args);

        if let Err(e) = config_setup::ensure_config() {
            eprintln!("Failed to initialize config: {}", e);
        }

        if let Err(e) = hypr_setup::ensure_hypr_source() {
            eprintln!("Failed to set up Hyprland config: {}", e);
        }

        cx.set_global(AppEvents::default());
        gpui_component::init(cx);
        load_custom_fonts(cx);
        apply_embedded_themes(cx);
        ui_theme_watcher::load_and_apply_omarchy_theme(cx);
        ui_theme_watcher::spawn_ui_theme_watcher(cx);

        // Load and apply saved font size from settings (after theme change to override default)
        if let Ok(font_size_str) = config_setup::get_font_size() {
            let font_size_px = match font_size_str.as_str() {
                "small" => 14.0,
                "medium" => 16.0,
                "large" => 18.0,
                _ => 16.0,
            };
            gpui_component::Theme::global_mut(cx).font_size = gpui::px(font_size_px);
        }

        cx.on_action(|_: &app_menu::SwitchToLight, cx: &mut App| {
            gpui_component::Theme::change(gpui_component::ThemeMode::Light, None, cx);
            cx.refresh_windows();
        });
        cx.on_action(|_: &app_menu::SwitchToDark, cx: &mut App| {
            gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);
            cx.refresh_windows();
        });
        cx.on_action(|_: &app_menu::Quit, cx: &mut App| {
            cx.quit();
        });
        cx.on_action(|action: &app_menu::SelectFont, cx: &mut App| {
            gpui_component::Theme::global_mut(cx).font_size = gpui::px(action.0 as f32);

            let font_size_str = match action.0 {
                14 => "small",
                16 => "medium",
                18 => "large",
                _ => "medium",
            };

            if let Err(e) = config_setup::update_font_size(font_size_str) {
                eprintln!("Failed to save font size setting: {}", e);
            }

            cx.refresh_windows();
        });
        cx.on_action(|_: &app_menu::ToggleSidebar, cx: &mut App| {
            app_events::emit(cx, AppEvent::ToggleSidebar);
        });

        // Never leave Hyprland stuck in the keystroke-recording submap.
        cx.on_app_quit(|_cx| {
            omarchist::system::keybinds::submap::leave_recording_submap();
            async {}
        })
        .detach();

        // Every app shortcut comes from the shortcuts table.
        cx.bind_keys(omarchist::ui::shortcuts::key_bindings());
        cx.bind_keys([
            // Editing keys that gpui-component does not bind on Linux.
            KeyBinding::new("ctrl-shift-z", gpui_component::input::Redo, None),
            // Keystroke recorder (only while it is focused but not recording)
            KeyBinding::new(
                "enter",
                keystroke_input::StartRecording,
                Some("KeystrokeInput"),
            ),
            KeyBinding::new(
                "space",
                keystroke_input::StartRecording,
                Some("KeystrokeInput"),
            ),
            KeyBinding::new(
                "backspace",
                keystroke_input::ClearKeystrokes,
                Some("KeystrokeInput"),
            ),
            KeyBinding::new(
                "delete",
                keystroke_input::ClearKeystrokes,
                Some("KeystrokeInput"),
            ),
        ]);

        cx.spawn(async move |cx| {
            let window_options = WindowOptions {
                titlebar: Some(TitleBar::title_bar_options()),
                focus: true,
                show: true,
                app_id: Some("omarchist".into()),
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
