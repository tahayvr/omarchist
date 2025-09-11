mod commands;
pub mod services;
pub mod types;

use services::check_cli_args;
use services::cli_handler::handle_cli_arguments;

/// Create the invoke handler with organized command groups
fn create_invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![
        // Settings commands
        commands::settings::get_app_settings,
        commands::settings::update_app_settings,
        commands::settings::reset_app_settings,
        // Shell commands
        commands::shell::run_update_script,
        commands::shell::get_omarchy_version,
        commands::shell::apply_theme,
        commands::shell::refresh_theme_adjustments,
        commands::shell::execute_bash_command,
        commands::shell::execute_bash_command_async,
        // Theme system commands
        services::themes::get_themes::get_themes,
        services::themes::get_sys_themes::get_sys_themes,
        services::themes::get_sys_themes::get_sys_theme_by_name,
        services::get_sys_themes::get_themes_cached,
        services::get_sys_themes::preload_themes,
        services::get_sys_themes::refresh_theme_cache,
        services::get_sys_themes::get_theme_metadata,
        services::get_sys_themes::clear_color_cache,
        services::get_sys_themes::get_cache_stats,
        services::get_sys_themes::invalidate_theme_cache,
        services::get_sys_themes::invalidate_themes_cache,
        services::get_sys_themes::invalidate_custom_themes_cache,
        services::get_sys_themes::invalidate_system_themes_cache,
        services::get_sys_themes::invalidate_and_refresh_cache,
        services::themes::get_current_theme::get_system_theme_colors,
        // Custom theme commands
        services::themes::custom_themes::create_custom_theme,
        services::themes::custom_themes::create_custom_theme_advanced,
        services::themes::custom_themes::update_custom_theme,
        services::themes::custom_themes::update_custom_theme_advanced,
        services::themes::custom_themes::get_custom_theme,
        services::themes::custom_themes::list_custom_themes,
        services::themes::custom_themes::delete_custom_theme,
        services::themes::custom_themes::init_custom_theme,
        services::themes::custom_themes::get_app_schemas,
        services::themes::custom_themes::get_theme_backgrounds,
        services::themes::custom_themes::add_theme_backgrounds,
        services::themes::custom_themes::remove_theme_background,
        services::themes::custom_themes::get_background_image_data,
        // Configuration commands
        commands::update_config::update_config,
        // Light mode commands
        services::config::light_mode::is_theme_light_mode,
        services::config::light_mode::set_theme_light_mode,
        // Cache commands
        services::cache::cache_config::get_cache_config,
        services::cache::cache_config::update_cache_config,
        services::cache::cache_config::reset_cache_config,
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)] // Omarchy mobile confirmed?
pub fn run() {
    // Apply NVIDIA compatibility fixes before startup (issue #1)
    if let Err(e) = services::nvidia_detection::setup_nvidia_compatibility() {
        log::warn!("Failed to setup NVIDIA compatibility: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_single_instance::init(
            |app_handle, args, _cwd| {
                if let Err(e) = handle_cli_arguments(app_handle, args) {
                    log::error!("Failed to handle CLI arguments: {e}");
                }
            },
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(create_invoke_handler())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Check CLI arguments early in startup process, before UI initialization
            log::info!("Checking CLI arguments during application startup");
            match check_cli_args(app) {
                Ok(cli_result) => {
                    if !cli_result.should_continue {
                        // Early exit logic for refresh command on first instance
                        if let Some(reason) = &cli_result.exit_reason {
                            log::info!("Early exit triggered: {reason}");
                        }
                        log::info!(
                            "Exiting application without launching UI (exit code: {})",
                            cli_result.exit_code
                        );
                        std::process::exit(cli_result.exit_code);
                    } else {
                        log::info!("CLI check passed - continuing with normal startup");
                    }
                },
                Err(e) => {
                    log::error!("Failed to check CLI arguments: {e}");
                    log::info!("Continuing with normal startup despite CLI check error");
                    // Continue with normal startup even if CLI check fails
                },
            }

            // Initialize cache manager (optimized to avoid unnecessary clones)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Initialize cache manager with configuration from file
                match services::cache::cache_config::CacheConfigManager::load_config(&app_handle) {
                    Ok(config) => {
                        // Avoid cloning the entire config, just move the theme_cache part
                        let theme_cache_config = config.theme_cache;
                        let preload_on_startup = theme_cache_config.preload_on_startup;

                        let _cache_manager =
                            services::cache::cache_manager::init_cache_manager_with_config(
                                theme_cache_config,
                            )
                            .await;
                        log::info!("Cache manager initialized successfully");

                        // Preload themes if configured to do so
                        if preload_on_startup {
                            if let Err(e) = services::get_sys_themes::preload_themes().await {
                                log::warn!("Failed to preload themes on startup: {e}");
                            } else {
                                log::info!("Themes preloaded successfully on startup");
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to load cache config, using defaults: {e}");
                        let _cache_manager =
                            services::cache::cache_manager::init_cache_manager().await;
                        log::info!("Cache manager initialized with defaults");
                    },
                }
            });

            // Theme refresh is now handled via CLI commands through single instance plugin

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                log::info!("Window closing");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
