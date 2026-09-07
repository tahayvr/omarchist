mod paths;

pub mod btop;
pub mod chromium;
pub mod colors;
pub mod icons;
pub mod lifecycle;
pub mod lock;

pub use btop::update_btop_theme;
pub use chromium::update_chromium_config;
pub use colors::update_colors_toml;
pub use icons::update_icons_theme;
pub use lifecycle::{
    create_theme_from_defaults, generate_unique_theme_name, load_theme_for_editing, rename_theme,
    save_theme_data,
};
pub use lock::update_lock_toml;
