use crate::services::import_export::{ImportResult, ThemeImportExportService, ValidationResult};
use crate::services::themes::custom_themes::CustomThemeService;
use std::path::PathBuf;
use tauri::AppHandle;

/// Export a custom theme to a JSON file
#[tauri::command]
pub async fn export_custom_theme(
    app_handle: AppHandle,
    theme_name: String,
    destination: String,
) -> Result<String, String> {
    let service = ThemeImportExportService::new(&app_handle)?;
    let destination_path = PathBuf::from(&destination);

    let export_path = service
        .export_theme(&theme_name, destination_path)
        .map_err(|e| e.to_string())?;

    Ok(export_path.to_string_lossy().to_string())
}

/// Import a theme from a JSON file
#[tauri::command]
pub async fn import_custom_theme(
    app_handle: AppHandle,
    file_path: String,
    rename_on_conflict: bool,
) -> Result<ImportResult, String> {
    log::info!("Importing theme from: {}", file_path);
    let service = ThemeImportExportService::new(&app_handle)?;
    let path = PathBuf::from(&file_path);

    let result = service
        .import_theme_from_file(&path, rename_on_conflict)
        .map_err(|e| e.to_string())?;

    log::info!(
        "Import result: success={}, theme_name={}",
        result.success,
        result.theme_name
    );

    // Invalidate cache for the imported theme
    if result.success {
        let sanitized_name = CustomThemeService::sanitize_name(&result.theme_name);

        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            log::info!("Invalidating cache for theme directory: {}", sanitized_name);
            cache.invalidate_theme(&sanitized_name).await;
            // Trigger background refresh to include the new theme
            log::info!("Triggering background cache refresh");
            let _ = cache.trigger_background_refresh().await;
            log::info!("Cache refresh triggered");
        } else {
            log::warn!("Failed to get theme cache for invalidation");
        }
    }

    Ok(result)
}

/// Validate a theme file before importing
#[tauri::command]
pub async fn validate_theme_file(
    app_handle: AppHandle,
    file_path: String,
) -> Result<ValidationResult, String> {
    let service = ThemeImportExportService::new(&app_handle)?;
    let path = PathBuf::from(&file_path);

    Ok(service.validate_theme_file(&path))
}
