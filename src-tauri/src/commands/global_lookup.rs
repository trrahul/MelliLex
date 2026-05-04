use crate::errors::AppError;
use crate::global_lookup;
use tauri::AppHandle;

#[tauri::command]
pub fn enable_global_lookup(app: AppHandle) -> Result<(), AppError> {
    log::info!("[Command] Enabling global lookup");
    global_lookup::enable(app)
        .map_err(|e| AppError::Validation(format!("Failed to enable global lookup: {}", e)))
}

#[tauri::command]
pub fn disable_global_lookup(app: AppHandle) -> Result<(), AppError> {
    log::info!("[Command] Disabling global lookup");
    global_lookup::disable(app)
        .map_err(|e| AppError::Validation(format!("Failed to disable global lookup: {}", e)))
}
