#[tauri::command]
pub async fn update_config(app: String, key: String, value: String) -> Result<(), String> {
    // STUB: This command is currently a placeholder for future color config persistence functionality
    // When implemented, it should persist color configuration changes to the appropriate config files
    log::info!("Received color update: app={app}, key={key}, value={value}");
    Ok(())
}
