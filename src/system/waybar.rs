mod config;
mod icons;
mod jsonc;
mod library;
mod paths;
mod profiles;
mod types;

pub use config::{
    add_module_to_zone, get_bar_settings, get_module_config, load_waybar_config,
    replace_top_level_value, save_waybar_config, set_bar_setting, set_module_config_field,
};
pub use library::module_library;
pub use paths::{omarchist_config_dir, waybar_profile_config_path, waybar_profiles_dir};
pub use profiles::{
    apply_waybar_profile, create_waybar_profile, delete_waybar_profile, duplicate_waybar_profile,
    has_original_waybar_backup, list_waybar_profiles, rename_waybar_profile,
    restore_original_waybar_config,
};
pub use types::{BarSettings, LibraryModule, WaybarConfig, WaybarModule, WaybarZone};
