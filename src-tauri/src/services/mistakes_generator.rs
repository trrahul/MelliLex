use crate::constants::explore_features;
use crate::db::Database;
use crate::errors::AppError;
use crate::models::MistakeItem;
use crate::services::dictionary_service::DictionaryService;
use crate::services::explore_cache_repository::ExploreCacheRepository;
use log::{debug, info};

pub struct MistakesGenerator<'a> {
    cache_repo: ExploreCacheRepository<'a>,
    provider_scope: String,
    provider_name: String,
    language: String,
    db: &'a Database,
}

impl<'a> MistakesGenerator<'a> {
    pub fn new(
        cache_repo: ExploreCacheRepository<'a>,
        provider_scope: String,
        provider_name: String,
        language: String,
        db: &'a Database,
    ) -> Self {
        Self {
            cache_repo,
            provider_scope,
            provider_name,
            language,
            db,
        }
    }

    pub async fn generate(
        &self,
        word: &str,
        dictionary_service: &DictionaryService,
        force: bool,
    ) -> Result<Vec<MistakeItem>, AppError> {
        info!("Generating common mistakes for word: {} (force: {})", word, force);

        if !force {
            // Try dedicated mistakes cache first
            if let Some(cached) = self.cache_repo.fetch_feature::<Vec<MistakeItem>>(
                word,
                &self.provider_scope,
                explore_features::MISTAKES,
            )? {
                if !cached.is_empty() {
                    debug!("Using cached mistakes for '{}'", word);
                    return Ok(cached);
                }
            }

            // Fallback: check progressive word cache
            if let Some(progressive_cache) = self
                .db
                .get_cached_word_progressive(word, &self.provider_name)?
            {
                if let Some(ref mistakes_data) = progressive_cache.mistakes {
                    if !mistakes_data.mistakes.is_empty() {
                        debug!("Using mistakes from progressive cache for '{}'", word);
                        let mistakes = mistakes_data.mistakes.clone();
                        // Store in dedicated mistakes cache for faster future lookups
                        self.cache_repo.store_feature(
                            word,
                            &self.provider_scope,
                            explore_features::MISTAKES,
                            &mistakes,
                        );
                        return Ok(mistakes);
                    }
                }
            }
        }

        // Generate fresh mistakes
        info!("Generating fresh mistakes for '{}'", word);
        let (section_3, _) = dictionary_service
            .generate_section3_mistakes(word, &self.language)
            .await?;

        // Cache the newly generated mistakes
        let mistakes = section_3.mistakes.clone();
        self.cache_mistakes_in_both_caches(word, &section_3, mistakes.clone())
            .await;

        Ok(mistakes)
    }

    async fn cache_mistakes_in_both_caches(
        &self,
        word: &str,
        section_3: &crate::models::WordMistakes,
        mistakes: Vec<MistakeItem>,
    ) {
        // Store in dedicated mistakes cache
        self.cache_repo
            .store_feature(word, &self.provider_scope, explore_features::MISTAKES, &mistakes);

        // Also update progressive cache if it exists
        if let Ok(Some(mut progressive)) = self
            .db
            .get_cached_word_progressive(word, &self.provider_name)
        {
            progressive.mistakes = Some(section_3.clone());
            if let Err(e) =
                self.db
                    .cache_word_progressive(word, &progressive, &self.provider_name)
            {
                log::warn!(
                    "Failed to update progressive cache with new mistakes for '{}': {}",
                    word,
                    e
                );
            }
        }
    }
}
