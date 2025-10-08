use crate::services::hyprland::{self, HyprlandConfigService};
use crate::types::{
    HyprlandAnimationSettings, HyprlandAnimationSnapshot, HyprlandDecorationSettings,
    HyprlandDecorationSnapshot, HyprlandGeneralSettings, HyprlandGeneralSnapshot,
    HyprlandInputSettings, HyprlandInputSnapshot, KeyboardCatalog,
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

/// Request payload for updating Hyprland input overrides.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateHyprlandInputPayload {
    pub overrides: HyprlandInputSettings,
}

/// Request payload for updating Hyprland animation overrides.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateHyprlandAnimationPayload {
    pub overrides: HyprlandAnimationSettings,
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

/// Load the current Hyprland input settings snapshot (defaults merged with overrides).
#[tauri::command]
pub fn get_hyprland_input_settings() -> Result<HyprlandInputSnapshot, String> {
    let service = HyprlandConfigService::new().map_err(|err| err.to_string())?;
    service.load_input_snapshot().map_err(|err| err.to_string())
}

/// Load the available keyboard models, layouts, variants, and options from XKB definitions.
#[tauri::command]
pub fn get_keyboard_catalog() -> Result<KeyboardCatalog, String> {
    hyprland::keyboard::load_keyboard_catalog().map_err(|err| err.to_string())
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

/// Update Hyprland input overrides and return the refreshed snapshot.
#[tauri::command]
pub fn update_hyprland_input_settings(
    payload: UpdateHyprlandInputPayload,
) -> Result<HyprlandInputSnapshot, String> {
    let service = HyprlandConfigService::new().map_err(|err| err.to_string())?;
    service
        .persist_input_overrides(&payload.overrides)
        .map_err(|err| err.to_string())?;
    service.load_input_snapshot().map_err(|err| err.to_string())
}

/// Load the current Hyprland animation settings snapshot (defaults merged with overrides).
#[tauri::command]
pub fn get_hyprland_animation_settings() -> Result<HyprlandAnimationSnapshot, String> {
    let service = HyprlandConfigService::new().map_err(|err| err.to_string())?;
    service
        .load_animation_snapshot()
        .map_err(|err| err.to_string())
}

/// Update Hyprland animation overrides and return the refreshed snapshot.
#[tauri::command]
pub fn update_hyprland_animation_settings(
    payload: UpdateHyprlandAnimationPayload,
) -> Result<HyprlandAnimationSnapshot, String> {
    let service = HyprlandConfigService::new().map_err(|err| err.to_string())?;
    service
        .persist_animation_overrides(&payload.overrides)
        .map_err(|err| err.to_string())?;
    service
        .load_animation_snapshot()
        .map_err(|err| err.to_string())
}
