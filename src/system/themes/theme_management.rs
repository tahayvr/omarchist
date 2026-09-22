mod paths;

pub mod colors;
pub mod lifecycle;

pub use colors::update_colors_toml;
pub use lifecycle::{
    create_theme_from_defaults, generate_unique_theme_name, load_theme_for_editing, rename_theme,
    save_theme_data, slugify_theme_name, unique_theme_name, update_theme,
};
