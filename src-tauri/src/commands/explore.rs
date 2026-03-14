use crate::db::Database;
use crate::errors::AppError;
use crate::models::*;
use crate::services::command_orchestrator::CommandOrchestrator;
use tauri::State;

#[tauri::command]
pub async fn generate_contextual_examples(
    word: String,
    context: String,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<Vec<String>, AppError> {
    orchestrator
        .generate_contextual_examples(&word, &context, &db)
        .await
}

#[tauri::command]
pub async fn generate_formality_analysis(
    word: String,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<(f64, Vec<FormalityAlternative>), AppError> {
    orchestrator.generate_formality_analysis(&word, &db).await
}

#[tauri::command]
pub async fn generate_domain_exploration(
    word: String,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<Vec<DomainExploration>, AppError> {
    orchestrator
        .generate_domain_exploration(&word, &db)
        .await
}

#[tauri::command]
pub async fn generate_usage_patterns(
    word: String,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<Vec<UsagePattern>, AppError> {
    orchestrator.generate_usage_patterns(&word, &db).await
}

#[tauri::command]
pub async fn generate_practice_exercises_only(
    word: String,
    count: usize,
    force: bool,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<Vec<PracticeExercise>, AppError> {
    orchestrator
        .generate_practice_exercises_only(&word, count, force, &db)
        .await
}

#[tauri::command]
pub async fn generate_common_mistakes(
    word: String,
    force: bool,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<Vec<MistakeItem>, AppError> {
    orchestrator.generate_common_mistakes(&word, force, &db).await
}

#[tauri::command]
pub async fn get_cached_exploration_features(
    word: String,
    db: State<'_, Database>,
    orchestrator: State<'_, CommandOrchestrator>,
) -> Result<CachedExploreFeatures, AppError> {
    orchestrator.get_cached_exploration_features(&word, &db)
}
