use crate::services::hyprland::HyprlandConfigService;
use crate::types::{
    HyprlandDecorationSettings, HyprlandDecorationSnapshot, HyprlandGeneralSettings,
    HyprlandGeneralSnapshot,
};

/// Request payload for updating Hyprland general overrides.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateHyprlandGeneralPayload {
    pub overrides: HyprlandGeneralSettings,
}

/// Request payload for updating Hyprland decoration overrides.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateHyprlandDecorationPayload {
    pub overrides: HyprlandDecorationSettings,
}

/// Load the current Hyprland general settings snapshot (defaults merged with overrides).
#[tauri::command]
pub fn get_hyprland_general_settings() -> Result<HyprlandGeneralSnapshot, String> {
    let service = HyprlandConfigService::new().map_err(|err| err.to_string())?;
    service
        .load_general_snapshot()
        .map_err(|err| err.to_string())
}

/// Update Hyprland general overrides and return the refreshed snapshot.
#[tauri::command]
pub fn update_hyprland_general_settings(
    payload: UpdateHyprlandGeneralPayload,
) -> Result<HyprlandGeneralSnapshot, String> {
    let service = HyprlandConfigService::new().map_err(|err| err.to_string())?;
    service
        .persist_general_overrides(&payload.overrides)
        .map_err(|err| err.to_string())?;
    service
        .load_general_snapshot()
        .map_err(|err| err.to_string())
}

/// Load the current Hyprland decoration settings snapshot (defaults merged with overrides).
#[tauri::command]
pub fn get_hyprland_decoration_settings() -> Result<HyprlandDecorationSnapshot, String> {
    let service = HyprlandConfigService::new().map_err(|err| err.to_string())?;
    service
        .load_decoration_snapshot()
        .map_err(|err| err.to_string())
}

/// Update Hyprland decoration overrides and return the refreshed snapshot.
#[tauri::command]
pub fn update_hyprland_decoration_settings(
    payload: UpdateHyprlandDecorationPayload,
) -> Result<HyprlandDecorationSnapshot, String> {
    let service = HyprlandConfigService::new().map_err(|err| err.to_string())?;
    service
        .persist_decoration_overrides(&payload.overrides)
        .map_err(|err| err.to_string())?;
    service
        .load_decoration_snapshot()
        .map_err(|err| err.to_string())
}
