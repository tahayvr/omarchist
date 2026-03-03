use gpui::{App, AppContext, Application, KeyBinding, WindowOptions};
use gpui_component::{Root, Theme, ThemeMode, ThemeSet, TitleBar};
use omarchist::cli::{CliArgs, ViewOption};
use omarchist::system::config::config_setup;
use omarchist::system::config::hypr_setup;
use omarchist::system::config::waybar_setup;
use omarchist::system::ui_theme_watcher;
use omarchist::ui::app_view::ActivePage;
use omarchist::ui::menu::app_menu;
use omarchist::{CombinedAssets, MainTitleBar, MainWindowView};
use std::rc::Rc;

/// Convert CLI arguments to the initial ActivePage
fn cli_args_to_active_page(args: &CliArgs) -> ActivePage {
    match args.view {
        Some(ViewOption::System) => ActivePage::SystemMonitor,
        Some(ViewOption::Config) => ActivePage::Configuration,
        Some(ViewOption::Settings) => ActivePage::Settings,
        Some(ViewOption::About) => ActivePage::About,
        Some(ViewOption::Omarchy) => ActivePage::Omarchy,
        Some(ViewOption::Themes) => {
            // If a theme name is provided, open theme edit page
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
    // Load the embedded JetBrains Mono font
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

    // Register the font with GPUI's text system
    if let Err(err) = cx.text_system().add_fonts(vec![font_data]) {
        eprintln!("Failed to add font: {}", err);
    } else {
        println!("Font registered successfully");
    }
}

fn main() {
    // Parse CLI arguments before starting the application
    let cli_args = CliArgs::parse_args();

    let app = Application::new().with_assets(CombinedAssets::new());

    app.run(move |cx| {
        // Determine initial page from CLI arguments
        let initial_page = cli_args_to_active_page(&cli_args);

        // Ensure config directory and settings.json exist
        if let Err(e) = config_setup::ensure_config() {
            eprintln!("Failed to initialize config: {}", e);
        }

        // Ensure Hyprland config includes omarchist source directive
        if let Err(e) = hypr_setup::ensure_hypr_source() {
            eprintln!("Failed to set up Hyprland config: {}", e);
        }

        // Ensure waybar config directory exists
        if let Err(e) = waybar_setup::ensure_waybar_config() {
            eprintln!("Failed to set up waybar config: {}", e);
        }

        gpui_component::init(cx);
        load_custom_fonts(cx);
        apply_embedded_themes(cx);
        // Apply the omarchy current theme immediately at startup, falling back to embedded theme
        ui_theme_watcher::load_and_apply_omarchy_theme(cx);
        // Start watching for theme switches
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

            // Map pixel size to font size string and save to settings
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
            omarchist::ui::app_view::PENDING_TOGGLE_SIDEBAR.with(|flag| {
                *flag.borrow_mut() = true;
            });
            cx.refresh_windows();
        });

        cx.bind_keys([
            KeyBinding::new("ctrl-q", app_menu::Quit, None),
            KeyBinding::new("ctrl-,", app_menu::NavigateToSettings, None),
            KeyBinding::new("ctrl-alt-l", app_menu::SwitchToLight, None),
            KeyBinding::new("ctrl-alt-d", app_menu::SwitchToDark, None),
            KeyBinding::new("ctrl-z", gpui_component::input::Undo, None),
            KeyBinding::new("ctrl-shift-z", gpui_component::input::Redo, None),
            KeyBinding::new("ctrl-x", gpui_component::input::Cut, None),
            KeyBinding::new("ctrl-c", gpui_component::input::Copy, None),
            KeyBinding::new("ctrl-v", gpui_component::input::Paste, None),
            KeyBinding::new("ctrl-r", app_menu::RefreshTheme, None),
            KeyBinding::new("ctrl-b", app_menu::ToggleSidebar, None),
            // Global page navigation shortcuts
            KeyBinding::new("ctrl-1", app_menu::NavigateToThemes, None),
            KeyBinding::new("ctrl-2", app_menu::NavigateToConfig, None),
            KeyBinding::new("ctrl-3", app_menu::NavigateToStatusBar, None),
            // Keyboard navigation bindings - using MainWindow context
            KeyBinding::new("tab", app_menu::NextFocus, Some("MainWindow")),
            KeyBinding::new("shift-tab", app_menu::PrevFocus, Some("MainWindow")),
            KeyBinding::new("down", app_menu::NextItem, Some("MainWindow")),
            KeyBinding::new("up", app_menu::PrevItem, Some("MainWindow")),
            KeyBinding::new("right", app_menu::SelectNext, Some("MainWindow")),
            KeyBinding::new("left", app_menu::SelectPrev, Some("MainWindow")),
            KeyBinding::new("escape", app_menu::EscapeFocus, Some("MainWindow")),
            KeyBinding::new("enter", app_menu::ActivateItem, Some("MainWindow")),
            KeyBinding::new("space", app_menu::ActivateItem, Some("MainWindow")),
            // Theme edit page navigation
            KeyBinding::new("right", app_menu::ThemeEditNextTab, Some("ThemeEditPage")),
            KeyBinding::new("left", app_menu::ThemeEditPrevTab, Some("ThemeEditPage")),
            KeyBinding::new(
                "ctrl-right",
                app_menu::ThemeEditNextTab,
                Some("ThemeEditPage"),
            ),
            KeyBinding::new(
                "ctrl-left",
                app_menu::ThemeEditPrevTab,
                Some("ThemeEditPage"),
            ),
            KeyBinding::new("escape", app_menu::NavigateBack, Some("ThemeEditPage")),
            // About page keyboard navigation
            KeyBinding::new("tab", app_menu::NextFocus, Some("AboutView")),
            KeyBinding::new("shift-tab", app_menu::PrevFocus, Some("AboutView")),
            KeyBinding::new("enter", app_menu::ActivateItem, Some("AboutView")),
            KeyBinding::new("space", app_menu::ActivateItem, Some("AboutView")),
            KeyBinding::new("escape", app_menu::EscapeFocus, Some("AboutView")),
            // Omarchy page keyboard navigation
            KeyBinding::new("tab", app_menu::NextFocus, Some("OmarchyView")),
            KeyBinding::new("shift-tab", app_menu::PrevFocus, Some("OmarchyView")),
            KeyBinding::new("enter", app_menu::ActivateItem, Some("OmarchyView")),
            KeyBinding::new("space", app_menu::ActivateItem, Some("OmarchyView")),
            KeyBinding::new("escape", app_menu::EscapeFocus, Some("OmarchyView")),
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
                let title_bar = cx.new(|_| MainTitleBar::new());
                let main_view =
                    cx.new(|cx| MainWindowView::new(title_bar, initial_page.clone(), window, cx));
                cx.new(|cx| Root::new(main_view, window, cx))
            })?;

            // Attempt to activate the window after creation
            window_handle.update(cx, |_view, window, _cx| {
                window.activate_window();
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
