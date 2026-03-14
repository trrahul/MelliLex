use crate::errors::AppError;
use tauri::AppHandle;

#[tauri::command]
pub fn is_store_version() -> bool {
    crate::utils::platform::is_store_build()
}

#[tauri::command]
pub async fn check_for_app_updates(app: AppHandle) -> Result<bool, AppError> {
    let result = if crate::utils::platform::is_store_build() {
        // Store builds don't use Tauri updater (Microsoft Store handles updates)
        log::info!("[Command] Update check skipped - Store version (managed by Microsoft Store)");
        Ok(false)
    } else {
        #[cfg(debug_assertions)]
        {
            log::info!("[Command] Update check disabled in development mode");
            drop(app);
            Ok(false)
        }

        #[cfg(not(debug_assertions))]
        {
            use tauri_plugin_updater::UpdaterExt;

            log::info!("[Command] Checking for application updates (GitHub channel)");

            let updater = app.updater_builder().build().map_err(|e| {
                log::error!("Failed to build updater: {}", e);
                AppError::Config(format!("Failed to build updater: {}", e))
            })?;

            match updater.check().await {
                Ok(Some(update)) => {
                    log::info!(
                        "Update available: {} -> {}",
                        update.current_version,
                        update.version
                    );
                    Ok(true)
                }
                Ok(None) => {
                    log::info!("App is up to date");
                    Ok(false)
                }
                Err(e) => {
                    log::error!("Update check failed: {}", e);
                    Err(AppError::Config(format!("Update check failed: {}", e)))
                }
            }
        }
    };

    result
}
