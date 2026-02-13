use gpui::{App, AppContext, Application, KeyBinding, WindowOptions};
use gpui_component::{Root, Theme, ThemeSet, TitleBar};
use omarchist::ui::menu::app_menu;
use omarchist::{CombinedAssets, MainTitleBar, MainWindowView};
use std::rc::Rc;

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

    let mode = Theme::global(cx).mode;
    Theme::change(mode, None, cx);
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
    let app = Application::new().with_assets(CombinedAssets::new());

    app.run(move |cx| {
        gpui_component::init(cx);
        load_custom_fonts(cx);
        apply_embedded_themes(cx);
        gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);

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
        ]);

        cx.spawn(async move |cx| {
            let window_options = WindowOptions {
                titlebar: Some(TitleBar::title_bar_options()),
                ..Default::default()
            };
            cx.open_window(window_options, |window, cx| {
                let title_bar = cx.new(|_| MainTitleBar::new());

                let main_view = cx.new(|cx| MainWindowView::new(title_bar, window, cx));

                cx.new(|cx| Root::new(main_view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
