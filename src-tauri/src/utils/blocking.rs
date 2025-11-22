use crate::errors::AppError;
use tauri::async_runtime;

pub struct BlockingExecutor;

impl BlockingExecutor {
    pub async fn run<F, R>(task: F) -> Result<R, AppError>
    where
        F: FnOnce() -> Result<R, AppError> + Send + 'static,
        R: Send + 'static,
    {
        async_runtime::spawn_blocking(task)
            .await
            .map_err(|err| AppError::Config(format!("Blocking task join error: {}", err)))?
    }
}
