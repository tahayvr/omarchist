mod config;
mod icons;
mod jsonc;
mod library;
mod paths;
mod profiles;
mod types;

pub use config::{
    add_module_to_zone, get_bar_settings, get_live_bar_settings, get_live_module_config,
    get_module_config, load_live_waybar_config, load_waybar_config, replace_top_level_value,
    save_waybar_config, set_bar_setting, set_module_config_field,
};
pub use library::module_library;
pub use paths::{
    live_waybar_config_path, live_waybar_dir, omarchist_config_dir, waybar_current_profile_path,
    waybar_profile_config_path, waybar_profiles_dir,
};
pub use profiles::{
    CUSTOM_WAYBAR_PROFILE, OMARCHY_DEFAULT_PROFILE, UNKNOWN_MANAGED_PROFILE, adopt_live_waybar,
    apply_waybar_profile, create_waybar_profile, current_live_waybar_profile,
    delete_waybar_profile, duplicate_waybar_profile, ensure_custom_waybar_profile,
    ensure_default_waybar_profile, has_custom_waybar_profile, has_live_waybar_config,
    has_unknown_managed_live_waybar, import_live_waybar_as_profile, is_live_waybar_managed,
    is_read_only_waybar_profile, list_waybar_profiles, rename_waybar_profile,
    start_with_omarchy_default_profile, unique_waybar_profile_name,
};
pub use types::{BarSettings, LibraryModule, WaybarConfig, WaybarModule, WaybarZone};
