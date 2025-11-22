use crate::db::Database;
use crate::errors::AppError;
use crate::utils::blocking::BlockingExecutor;
use tauri::State;

#[tauri::command]
pub async fn get_cache_stats(db: State<'_, Database>) -> Result<(usize, usize, usize, i64), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.get_cache_stats()).await
}

#[tauri::command]
pub async fn clear_all_cache(db: State<'_, Database>) -> Result<(), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_all_cache()).await
}

#[tauri::command]
pub async fn clear_definition_cache(db: State<'_, Database>) -> Result<(), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_definition_cache()).await
}

#[tauri::command]
pub async fn clear_exploration_cache(db: State<'_, Database>) -> Result<(), AppError> {
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_exploration_cache()).await
}

#[tauri::command]
pub async fn clear_old_cache(db: State<'_, Database>, days: i64) -> Result<usize, AppError> {
    if days < 1 || days > 365 {
        return Err(AppError::validation("Days must be between 1 and 365"));
    }
    let db = db.inner().clone();
    BlockingExecutor::run(move || db.clear_old_cache(days)).await
}
