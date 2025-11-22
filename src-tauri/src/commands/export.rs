use crate::db::Database;
use crate::errors::AppError;
use crate::utils::blocking::BlockingExecutor;
use crate::validation;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn export_markdown_file(
    app: AppHandle,
    db: State<'_, Database>,
    word: String,
    provider: String,
    include_timestamp: bool,
) -> Result<String, AppError> {
    use crate::services::export_service::ExportService;

    let validated_word = validation::validate_word_query(&word)?;
    log::info!("Exporting markdown for word: {}", validated_word);

    let downloads_dir = app.path().download_dir().map_err(|err| {
        AppError::validation(format!("Downloads directory not available: {}", err))
    })?;

    let db = db.inner().clone();
    BlockingExecutor::run(move || {
        let export_service = ExportService::new(&db);
        export_service
            .export_word_to_markdown(
                &validated_word,
                &provider,
                &downloads_dir,
                include_timestamp,
            )
            .map(|path| path.to_string_lossy().to_string())
    })
    .await
}

#[tauri::command]
pub async fn export_phrase_markdown_file(
    app: AppHandle,
    db: State<'_, Database>,
    phrase: String,
    provider: String,
    include_timestamp: bool,
) -> Result<String, AppError> {
    use crate::services::export_service::ExportService;

    let validated_phrase = validation::validate_word_query(&phrase)?;
    log::info!("Exporting markdown for phrase: {}", validated_phrase);

    let downloads_dir = app.path().download_dir().map_err(|err| {
        AppError::validation(format!("Downloads directory not available: {}", err))
    })?;

    let db = db.inner().clone();
    BlockingExecutor::run(move || {
        let export_service = ExportService::new(&db);
        export_service
            .export_phrase_to_markdown(
                &validated_phrase,
                &provider,
                &downloads_dir,
                include_timestamp,
            )
            .map(|path| path.to_string_lossy().to_string())
    })
    .await
}

#[tauri::command]
pub async fn export_to_capacities(
    api_token: String,
    space_id: String,
    markdown: String,
    no_timestamp: bool,
) -> Result<(), AppError> {
    use crate::services::export_service::ExportService;

    let validated_token = validation::validate_api_token(&api_token)?;
    let validated_markdown = validation::validate_markdown(&markdown)?;

    log::info!("[Capacities Export] Starting export request");
    log::debug!(
        "[Capacities Export] Space ID: {}, Markdown length: {} chars",
        space_id,
        validated_markdown.len()
    );

    ExportService::export_to_capacities(
        validated_token,
        &space_id,
        validated_markdown,
        no_timestamp,
    )
    .await
}
