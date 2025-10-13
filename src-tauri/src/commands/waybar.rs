use crate::services::waybar::{
    SaveWaybarConfigPayload, WaybarConfigService, WaybarConfigSnapshot,
    WaybarProfileChangeResponse, WaybarProfileListResponse,
};

/// Load the current Waybar configuration snapshot (layout, modules, globals, passthrough).
#[tauri::command]
pub fn get_waybar_config_snapshot() -> Result<WaybarConfigSnapshot, String> {
    let mut service = WaybarConfigService::new().map_err(|err| err.to_string())?;
    service.load_snapshot().map_err(|err| err.to_string())
}

/// Persist Waybar configuration changes and return the refreshed snapshot.
#[tauri::command]
pub fn save_waybar_config_snapshot(
    payload: SaveWaybarConfigPayload,
) -> Result<WaybarConfigSnapshot, String> {
    let mut service = WaybarConfigService::new().map_err(|err| err.to_string())?;
    service
        .save_snapshot(&payload)
        .map_err(|err| err.to_string())
}

/// Retrieve the list of saved Waybar configuration profiles.
#[tauri::command]
pub fn list_waybar_profiles() -> Result<WaybarProfileListResponse, String> {
    let mut service = WaybarConfigService::new().map_err(|err| err.to_string())?;
    service.list_profiles().map_err(|err| err.to_string())
}

/// Create a new Waybar configuration profile based on the default Omarchy template.
#[tauri::command]
pub fn create_waybar_profile(name: String) -> Result<WaybarProfileChangeResponse, String> {
    let mut service = WaybarConfigService::new().map_err(|err| err.to_string())?;
    service.create_profile(&name).map_err(|err| err.to_string())
}

/// Activate an existing Waybar configuration profile and copy it into Waybar's live location.
#[tauri::command]
pub fn select_waybar_profile(profile_id: String) -> Result<WaybarProfileChangeResponse, String> {
    let mut service = WaybarConfigService::new().map_err(|err| err.to_string())?;
    service
        .select_profile(&profile_id)
        .map_err(|err| err.to_string())
}

/// Delete a stored Waybar configuration profile.
#[tauri::command]
pub fn delete_waybar_profile(profile_id: String) -> Result<WaybarProfileChangeResponse, String> {
    let mut service = WaybarConfigService::new().map_err(|err| err.to_string())?;
    service
        .delete_profile(&profile_id)
        .map_err(|err| err.to_string())
}
