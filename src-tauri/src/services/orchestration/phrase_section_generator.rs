use crate::errors::AppError;
use crate::models::{PhraseDefinitionData, PhraseSection2Context, PhraseSection3Related};
use crate::services::orchestration::PhraseProgressiveEmitter;
use crate::services::phrase_service::PhraseService;
use log::info;
use std::sync::Arc;
use tokio::task::JoinSet;

/// Result type for parallel phrase section generation tasks.
pub enum PhraseSectionTaskResult {
    Section2(PhraseSection2Context),
    Section3(PhraseSection3Related),
}

/// Handles parallel generation of progressive phrase sections.
/// Section 1 is generated and emitted first, then sections 2 and 3 run in parallel.
pub struct PhraseSectionGenerator;

impl PhraseSectionGenerator {
    /// Generates section 1 immediately and emits it, then spawns parallel tasks for sections 2 and 3.
    /// Returns the combined PhraseDefinitionData on success.
    pub async fn generate_progressive<E: PhraseProgressiveEmitter + ?Sized>(
        phrase_service: Arc<PhraseService>,
        phrase: &str,
        language: &str,
        emitter: &E,
    ) -> Result<PhraseDefinitionData, AppError> {
        let phrase_owned = phrase.to_string();
        let language_owned = language.to_string();

        // Generate section 1 first so the UI can render the overview immediately.
        let (section1, _) = phrase_service
            .generate_section1_overview(&phrase_owned, &language_owned)
            .await
            .map_err(|e| {
                log::error!("Phrase Section 1 error for '{}': {}", phrase_owned, e);
                AppError::from(e)
            })?;

        info!("Emitting phrase section 1 (overview) for: {}", phrase_owned);
        emitter.emit_section1(&section1)?;

        // Generate sections 2 and 3 in parallel
        let (section2, section3) = Self::generate_parallel_sections(
            phrase_service,
            &phrase_owned,
            &language_owned,
            emitter,
        )
        .await?;

        Ok(PhraseDefinitionData {
            section1,
            section2,
            section3,
        })
    }

    /// Spawns parallel tasks for sections 2 and 3, emitting each as they complete.
    async fn generate_parallel_sections<E: PhraseProgressiveEmitter + ?Sized>(
        phrase_service: Arc<PhraseService>,
        phrase: &str,
        language: &str,
        emitter: &E,
    ) -> Result<(PhraseSection2Context, PhraseSection3Related), AppError> {
        let mut section2_data: Option<PhraseSection2Context> = None;
        let mut section3_data: Option<PhraseSection3Related> = None;

        let mut join_set: JoinSet<Result<PhraseSectionTaskResult, AppError>> = JoinSet::new();

        // Spawn section 2 task (context/origin)
        join_set.spawn({
            let svc = phrase_service.clone();
            let section_phrase = phrase.to_string();
            let section_language = language.to_string();
            async move {
                log::debug!(
                    "Generating phrase section 2 (context) [parallel] in {}",
                    section_language
                );
                svc.generate_section2_context(&section_phrase, &section_language)
                    .await
                    .map(|(section, _)| PhraseSectionTaskResult::Section2(section))
                    .map_err(|e| {
                        log::error!("Phrase Section 2 error for '{}': {}", section_phrase, e);
                        AppError::from(e)
                    })
            }
        });

        // Spawn section 3 task (related phrases)
        join_set.spawn({
            let svc = phrase_service.clone();
            let section_phrase = phrase.to_string();
            let section_language = language.to_string();
            async move {
                log::debug!(
                    "Generating phrase section 3 (related) [parallel] in {}",
                    section_language
                );
                svc.generate_section3_related(&section_phrase, &section_language)
                    .await
                    .map(|(section, _)| PhraseSectionTaskResult::Section3(section))
                    .map_err(|e| {
                        log::error!("Phrase Section 3 error for '{}': {}", section_phrase, e);
                        AppError::from(e)
                    })
            }
        });

        // Collect results and emit as they complete
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(PhraseSectionTaskResult::Section2(section))) => {
                    info!("Emitting phrase section 2 (context)");
                    emitter.emit_section2(&section)?;
                    section2_data = Some(section);
                }
                Ok(Ok(PhraseSectionTaskResult::Section3(section))) => {
                    info!("Emitting phrase section 3 (related)");
                    emitter.emit_section3(&section)?;
                    section3_data = Some(section);
                }
                Ok(Err(e)) => {
                    log::error!("Phrase section generation error: {}", e);
                    return Err(e);
                }
                Err(e) => {
                    log::error!("Phrase section task join error: {}", e);
                    return Err(AppError::from(anyhow::anyhow!("Task join error: {}", e)));
                }
            }
        }

        // Ensure we got both sections
        let section2 = section2_data.ok_or_else(|| {
            AppError::from(anyhow::anyhow!("Phrase section 2 was not generated"))
        })?;
        let section3 = section3_data.ok_or_else(|| {
            AppError::from(anyhow::anyhow!("Phrase section 3 was not generated"))
        })?;

        Ok((section2, section3))
    }
}
