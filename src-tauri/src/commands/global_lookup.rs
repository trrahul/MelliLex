use crate::errors::AppError;
use crate::global_lookup;
use crate::validation;
use tauri::AppHandle;

/// Internal helper for shortcut operations to reduce duplication
fn handle_shortcut_action<F>(
    app: AppHandle,
    shortcut: String,
    action_name: &str,
    action: F,
) -> Result<(), AppError>
where
    F: FnOnce(AppHandle, &str) -> Result<(), tauri::Error>,
{
    let validated_shortcut = validation::validate_shortcut(&shortcut)?;
    log::info!(
        "[Command] {} global lookup shortcut: {}",
        action_name,
        validated_shortcut
    );

    action(app, &validated_shortcut)
        .map_err(|e| AppError::Validation(format!("Failed to {} shortcut: {}", action_name.to_lowercase(), e)))?;

    Ok(())
}

#[tauri::command]
pub fn register_global_lookup_shortcut(app: AppHandle, shortcut: String) -> Result<(), AppError> {
    handle_shortcut_action(app, shortcut, "Registering", global_lookup::register_shortcut)
}

#[tauri::command]
pub fn unregister_global_lookup_shortcut(app: AppHandle, shortcut: String) -> Result<(), AppError> {
    handle_shortcut_action(app, shortcut, "Unregistering", global_lookup::unregister_shortcut)
}
