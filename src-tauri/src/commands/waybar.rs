use crate::services::waybar::{SaveWaybarConfigPayload, WaybarConfigService, WaybarConfigSnapshot};

/// Load the current Waybar configuration snapshot (layout, modules, globals, passthrough).
#[tauri::command]
pub fn get_waybar_config_snapshot() -> Result<WaybarConfigSnapshot, String> {
    let service = WaybarConfigService::new().map_err(|err| err.to_string())?;
    service.load_snapshot().map_err(|err| err.to_string())
}

/// Persist Waybar configuration changes and return the refreshed snapshot.
#[tauri::command]
pub fn save_waybar_config_snapshot(
    payload: SaveWaybarConfigPayload,
) -> Result<WaybarConfigSnapshot, String> {
    let service = WaybarConfigService::new().map_err(|err| err.to_string())?;
    service
        .save_snapshot(&payload)
        .map_err(|err| err.to_string())
}
