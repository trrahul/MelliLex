use crate::errors::AppError;
use crate::models::{WordProgressiveData, WordSection2Meanings, WordSection3Related};
use crate::services::dictionary_service::DictionaryService;
use crate::services::orchestration::ProgressiveEmitter;
use log::info;
use std::sync::Arc;
use tokio::task::JoinSet;

/// Result type for parallel section generation tasks.
pub enum SectionTaskResult {
    Section2(WordSection2Meanings),
    Section3(WordSection3Related),
}

/// Handles parallel generation of progressive sections (2 and 3) after section 1 is emitted.
pub struct SectionGenerator;

impl SectionGenerator {
    pub async fn generate_progressive<E: ProgressiveEmitter + ?Sized>(
        dictionary_service: Arc<DictionaryService>,
        word: &str,
        language: &str,
        emitter: &E,
    ) -> Result<WordProgressiveData, AppError> {
        let word_owned = word.to_string();
        let language_owned = language.to_string();

        // Generate section 1 first so the UI can render the header immediately.
        let (section1, _) = dictionary_service
            .generate_section1_header(&word_owned, &language_owned)
            .await
            .map_err(|e| {
                log::error!("Section 1 error for '{}': {}", word_owned, e);
                AppError::from(e)
            })?;

        info!("Emitting section 1 (header)");
        emitter.emit_section1(&section1)?;

        // Generate sections 2 and 3 in parallel
        let (section2, section3) = Self::generate_parallel_sections(
            dictionary_service,
            &word_owned,
            &language_owned,
            emitter,
        )
        .await?;

        Ok(WordProgressiveData {
            section1,
            section2,
            mistakes: None,
            section3,
        })
    }

    async fn generate_parallel_sections<E: ProgressiveEmitter + ?Sized>(
        dictionary_service: Arc<DictionaryService>,
        word: &str,
        language: &str,
        emitter: &E,
    ) -> Result<(WordSection2Meanings, WordSection3Related), AppError> {
        let mut section2_data: Option<WordSection2Meanings> = None;
        let mut section3_data: Option<WordSection3Related> = None;

        let mut join_set: JoinSet<Result<SectionTaskResult, AppError>> = JoinSet::new();

        // Spawn section 2 task
        join_set.spawn({
            let svc = dictionary_service.clone();
            let section_word = word.to_string();
            let section_language = language.to_string();
            async move {
                log::debug!(
                    "Generating section 2 (meanings) [parallel] in {}",
                    section_language
                );
                svc.generate_section2_meanings(&section_word, &section_language)
                    .await
                    .map(|(section, _)| SectionTaskResult::Section2(section))
                    .map_err(|e| {
                        log::error!("Section 2 error for '{}': {}", section_word, e);
                        AppError::from(e)
                    })
            }
        });

        // Spawn section 3 task
        join_set.spawn({
            let svc = dictionary_service.clone();
            let section_word = word.to_string();
            let section_language = language.to_string();
            async move {
                log::debug!(
                    "Generating section 3 (related) [parallel] in {}",
                    section_language
                );
                svc.generate_section3_related(&section_word, &section_language)
                    .await
                    .map(|(section, _)| SectionTaskResult::Section3(section))
                    .map_err(|e| {
                        log::error!("Section 3 error for '{}': {}", section_word, e);
                        AppError::from(e)
                    })
            }
        });

        // Collect results and emit as they complete
        while let Some(joined) = join_set.join_next().await {
            let result = joined.map_err(|err| {
                log::error!("Progressive section task failed: {}", err);
                AppError::AiProvider(format!("Progressive section task failed: {}", err))
            })??;

            match result {
                SectionTaskResult::Section2(section) => {
                    info!("Emitting section 2 (meanings)");
                    emitter.emit_section2(&section)?;
                    section2_data = Some(section);
                }
                SectionTaskResult::Section3(section) => {
                    info!("Emitting section 3 (related)");
                    emitter.emit_section3(&section)?;
                    section3_data = Some(section);
                }
            }
        }

        let section2 = section2_data
            .ok_or_else(|| AppError::AiProvider("Section 2 missing after generation".into()))?;
        let section3 = section3_data
            .ok_or_else(|| AppError::AiProvider("Section 3 missing after generation".into()))?;

        Ok((section2, section3))
    }
}
