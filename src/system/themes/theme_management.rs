mod paths;

pub mod btop;
pub mod chromium;
pub mod colors;
pub mod hyprland;
pub mod hyprlock;
pub mod icons;
pub mod lifecycle;
pub mod mako;
pub mod swayosd;
pub mod terminal;
pub mod walker;
pub mod waybar;

pub use btop::update_btop_theme;
pub use chromium::update_chromium_config;
pub use colors::{colors_config_from_terminal, update_colors_toml};
pub use hyprland::update_hyprland_conf;
pub use hyprlock::update_hyprlock_conf;
pub use icons::update_icons_theme;
pub use lifecycle::{
    create_theme_from_defaults, generate_unique_theme_name, load_theme_for_editing, rename_theme,
    save_theme_data,
};
pub use mako::update_mako_ini;
pub use swayosd::update_swayosd_css;
pub use terminal::update_terminal_configs;
pub use walker::update_walker_css;
pub use waybar::update_waybar_css;
