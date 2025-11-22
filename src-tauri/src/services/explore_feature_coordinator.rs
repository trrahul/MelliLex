use crate::constants::explore_features;
use crate::db::Database;
use crate::errors::AppError;
use crate::models::{
    CachedPracticeData, DomainExploration, MistakeItem, PracticeExercise, UsagePattern,
};
use crate::services::dictionary_service::DictionaryService;
use crate::services::explore_cache_repository::ExploreCacheRepository;
use crate::services::exploration_service::{DomainPromptLimits, ExplorationService};
use crate::services::mistakes_generator::MistakesGenerator;
use log::{debug, info};

/// Batch size for caching practice exercises.
pub const PRACTICE_CACHE_BATCH: usize = 5;

/// Coordinator for explore features - handles cache lookup + AI generation for each feature.
pub struct ExploreFeatureCoordinator<'a> {
    cache_repo: ExploreCacheRepository<'a>,
    provider_scope: String,
    exploration_service: &'a ExplorationService,
    mistakes_generator: MistakesGenerator<'a>,
    language: String,
}

impl<'a> ExploreFeatureCoordinator<'a> {
    pub fn new(
        cache_repo: ExploreCacheRepository<'a>,
        provider_scope: String,
        provider_name: String,
        language: String,
        exploration_service: &'a ExplorationService,
        db: &'a Database,
    ) -> Self {
        Self {
            cache_repo,
            provider_scope: provider_scope.clone(),
            exploration_service,
            mistakes_generator: MistakesGenerator::new(
                cache_repo,
                provider_scope,
                provider_name,
                language.clone(),
                db,
            ),
            language,
        }
    }

    pub async fn generate_practice_exercises(
        &self,
        word: &str,
        count: usize,
    ) -> Result<Vec<PracticeExercise>, AppError> {
        info!(
            "Generating practice exercises (count: {}) for word: {}",
            count, word
        );

        if count == PRACTICE_CACHE_BATCH {
            if let Some(cached) = self.cache_repo.fetch_feature::<CachedPracticeData>(
                word,
                &self.provider_scope,
                explore_features::PRACTICE,
            )? {
                debug!("Using cached practice exercises for '{}'", word);
                return Ok(cached.practice_exercises);
            }
        }

        let (result, _) = self
            .exploration_service
            .generate_practice_only(word, count, &self.language)
            .await
            .map_err(|e| {
                log::error!("Practice generation error for '{}': {}", word, e);
                AppError::from(e)
            })?;

        let exercises = result.practice_exercises;

        if count == PRACTICE_CACHE_BATCH {
            self.cache_repo.store_feature(
                word,
                &self.provider_scope,
                explore_features::PRACTICE,
                &CachedPracticeData {
                    practice_exercises: exercises.clone(),
                },
            );
        }

        info!("Generated {} practice exercises", exercises.len());
        Ok(exercises)
    }

    pub async fn generate_domain_exploration(
        &self,
        word: &str,
    ) -> Result<Vec<DomainExploration>, AppError> {
        info!("Generating domain exploration for '{}'", word);
        let limits = DomainPromptLimits::compact();

        if let Some(cached) = self.cache_repo.fetch_feature::<Vec<DomainExploration>>(
            word,
            &self.provider_scope,
            explore_features::DOMAINS,
        )? {
            if cached.len() >= limits.max_domains {
                debug!("Using cached domain exploration for '{}'", word);
                let mut domains = cached;
                domains.truncate(limits.max_domains);
                return Ok(domains);
            }
            debug!("Cached domains for '{}' insufficient; regenerating", word);
        }

        let (result, _) = self
            .exploration_service
            .generate_domains_only(word, limits, &self.language)
            .await
            .map_err(|e| {
                log::error!("Domain exploration error for '{}': {}", word, e);
                AppError::from(e)
            })?;

        let mut domains = result.domain_explorations;
        if domains.len() > limits.max_domains {
            domains.truncate(limits.max_domains);
        }

        self.cache_repo.store_feature(
            word,
            &self.provider_scope,
            explore_features::DOMAINS,
            &domains,
        );

        Ok(domains)
    }

    pub async fn generate_usage_patterns(&self, word: &str) -> Result<Vec<UsagePattern>, AppError> {
        info!("Generating usage patterns for '{}'", word);

        if let Some(cached) = self.cache_repo.fetch_feature::<Vec<UsagePattern>>(
            word,
            &self.provider_scope,
            explore_features::USAGE,
        )? {
            debug!("Using cached usage patterns for '{}'", word);
            return Ok(cached);
        }

        let (result, _) = self
            .exploration_service
            .generate_usage_only(word, &self.language)
            .await
            .map_err(|e| {
                log::error!("Usage patterns error for '{}': {}", word, e);
                AppError::from(e)
            })?;

        let patterns = result.usage_patterns;
        self.cache_repo.store_feature(
            word,
            &self.provider_scope,
            explore_features::USAGE,
            &patterns,
        );

        Ok(patterns)
    }

    pub async fn generate_custom_examples(
        &self,
        word: &str,
        context: &str,
    ) -> Result<Vec<String>, AppError> {
        let trimmed = context.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(
                "Custom context examples require a non-empty context",
            ));
        }
        info!(
            "Generating custom examples for '{}' with context '{}'",
            word, trimmed
        );
        self.exploration_service
            .generate_contextual_examples(word, trimmed, &self.language)
            .await
            .map_err(|e| {
                log::error!(
                    "Custom examples error for '{}' with context '{}': {}",
                    word,
                    trimmed,
                    e
                );
                AppError::from(e)
            })
    }

    pub async fn generate_mistakes(
        &self,
        word: &str,
        dictionary_service: &DictionaryService,
    ) -> Result<Vec<MistakeItem>, AppError> {
        self.mistakes_generator
            .generate(word, dictionary_service)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants;
    use crate::constants::explore_features;
    use crate::models::{
        DomainExploration, FormalityInfo, MistakeCategory, WordProgressiveData, WordSection1Header,
        WordSection2Meanings, WordMistakes, WordSection3Related, WordFrequency,
    };
    use crate::services::ai_provider::PromptSender;
    use crate::services::exploration_service::ExplorationService;
    use anyhow::Result;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[derive(Clone)]
    struct MockPromptSender {
        response: String,
        call_count: Arc<AtomicUsize>,
    }

    impl MockPromptSender {
        fn new(response: String) -> (Self, Arc<AtomicUsize>) {
            let counter = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    response,
                    call_count: counter.clone(),
                },
                counter,
            )
        }
    }

    #[async_trait]
    impl PromptSender for MockPromptSender {
        async fn send_prompt(
            &self,
            _prompt: &str,
        ) -> Result<(String, Option<crate::models::TokenUsage>)> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok((self.response.clone(), None))
        }
    }

    fn create_db() -> (Database, TempDir) {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = dir.path().join("test.db");
        let db = Database::new(path).expect("Failed to create database");
        (db, dir)
    }

    fn practice_response() -> String {
        serde_json::json!({
            "practiceExercises": [
                {
                    "question": "Pick the correct collocation",
                    "exerciseType": "multiple-choice",
                    "options": ["spark change", "sparkly change"],
                    "correctAnswer": "spark change",
                    "explanation": "'spark' pairs with abstract nouns",
                    "isAnswered": false,
                    "userAnswer": ""
                }
            ]
        })
        .to_string()
    }

    fn mistakes_response() -> String {
        serde_json::json!({
            "mistakes": [
                {
                    "type": "Grammar",
                    "incorrectUsage": "spark a changes",
                    "correction": "spark changes",
                    "category": "grammatical"
                }
            ]
        })
        .to_string()
    }

    fn domain_response(count: usize) -> String {
        let domains: Vec<_> = (0..count)
            .map(|i| {
                serde_json::json!({
                    "domain": format!("domain-{i}"),
                    "usageFrequency": "Common",
                    "commonCollocations": ["spark plug"],
                    "examples": ["spark example"],
                    "isExpanded": false
                })
            })
            .collect();
        serde_json::json!({
            "domainExplorations": domains
        })
        .to_string()
    }

    fn usage_response() -> String {
        serde_json::json!({
            "usagePatterns": [
                {
                    "template": "{word} debate",
                    "patternType": "collocation",
                    "description": "Use before noun",
                    "examples": ["spark debate"]
                }
            ],
            "contextualExamples": []
        })
        .to_string()
    }

    fn custom_examples_response() -> String {
        serde_json::json!({
            "examples": ["Spark innovation in labs"]
        })
        .to_string()
    }

    fn cached_domains(count: usize) -> Vec<DomainExploration> {
        (0..count)
            .map(|i| DomainExploration {
                domain: format!("domain-{i}"),
                usage_frequency: WordFrequency::Common,
                common_collocations: vec!["spark plug".into()],
                examples: vec!["spark plug".into()],
                is_expanded: false,
            })
            .collect()
    }

    fn coordinator<'a>(
        db: &'a Database,
        exploration_service: &'a ExplorationService,
        provider_name: &str,
    ) -> ExploreFeatureCoordinator<'a> {
        let scope = format!("{}::english", provider_name);
        let cache_repo = ExploreCacheRepository::new(db);
        ExploreFeatureCoordinator::new(
            cache_repo,
            scope,
            provider_name.to_string(),
            "English".to_string(),
            exploration_service,
            db,
        )
    }

    #[tokio::test]
    async fn practice_pipeline_caches_after_first_generation() {
        let (db, _dir) = create_db();
        let (mock_sender, call_counter) = MockPromptSender::new(practice_response());
        let exploration_service = ExplorationService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let first = coordinator
            .generate_practice_exercises("spark", PRACTICE_CACHE_BATCH)
            .await
            .expect("practice generation failed");
        assert_eq!(first.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);

        let second = coordinator
            .generate_practice_exercises("spark", PRACTICE_CACHE_BATCH)
            .await
            .expect("practice generation failed");
        assert_eq!(second.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn mistakes_use_cached_payload_before_generation() {
        let (db, _dir) = create_db();
        let (mock_sender, call_counter) = MockPromptSender::new(mistakes_response());
        let exploration_service = ExplorationService::new(Arc::new(mock_sender.clone()));
        let dictionary_service = DictionaryService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let cache_repo = ExploreCacheRepository::new(&db);
        cache_repo.store_feature(
            "spark",
            "openai::english",
            explore_features::MISTAKES,
            &vec![MistakeItem {
                mistake_type: "Grammar".into(),
                incorrect_usage: "spark a change".into(),
                correction: "spark change".into(),
                category: MistakeCategory::Grammatical,
            }],
        );

        let result = coordinator
            .generate_mistakes("spark", &dictionary_service)
            .await
            .expect("mistakes generation failed");
        assert_eq!(result.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn mistakes_hit_progressive_cache_when_available() {
        let (db, _dir) = create_db();
        let (mock_sender, call_counter) = MockPromptSender::new(mistakes_response());
        let exploration_service = ExplorationService::new(Arc::new(mock_sender.clone()));
        let dictionary_service = DictionaryService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let progressive = WordProgressiveData {
            section1: WordSection1Header {
                word: "spark".into(),
                pronunciation: String::new(),
                syllables: String::new(),
                origin: String::new(),
                formality: FormalityInfo {
                    level: String::new(),
                    percentage: 0,
                },
                domains: vec![],
                tldr: String::new(),
            },
            section2: WordSection2Meanings { meanings: vec![] },
            mistakes: Some(WordMistakes {
                mistakes: vec![MistakeItem {
                    mistake_type: "Grammar".into(),
                    incorrect_usage: "spark a changes".into(),
                    correction: "spark changes".into(),
                    category: MistakeCategory::Grammatical,
                }],
            }),
            section3: WordSection3Related {
                synonyms: vec![],
                antonyms: vec![],
                collocations: vec![],
            },
        };

        db.cache_word_progressive("spark", &progressive, "openai")
            .expect("cache progressive data");

        let result = coordinator
            .generate_mistakes("spark", &dictionary_service)
            .await
            .expect("mistakes generation failed");
        assert_eq!(result.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn mistakes_fall_back_to_dictionary_generation() {
        let (db, _dir) = create_db();
        let (mock_sender, call_counter) = MockPromptSender::new(mistakes_response());
        let exploration_service = ExplorationService::new(Arc::new(mock_sender.clone()));
        let dictionary_service = DictionaryService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let result = coordinator
            .generate_mistakes("spark", &dictionary_service)
            .await
            .expect("mistakes generation failed");
        assert_eq!(result.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);

        // Ensure the result was cached for subsequent calls.
        let second = coordinator
            .generate_mistakes("spark", &dictionary_service)
            .await
            .expect("mistakes generation failed");
        assert_eq!(second.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn domains_use_cache_when_sufficient() {
        let (db, _dir) = create_db();
        let (mock_sender, call_counter) = MockPromptSender::new(domain_response(8));
        let exploration_service = ExplorationService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let cache_repo = ExploreCacheRepository::new(&db);
        cache_repo.store_feature(
            "spark",
            "openai::english",
            constants::explore_features::DOMAINS,
            &cached_domains(4),
        );

        let result = coordinator
            .generate_domain_exploration("spark")
            .await
            .expect("domain generation failed");
        assert_eq!(result.len(), 4);
        assert_eq!(call_counter.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn domains_regenerate_when_cache_empty() {
        let (db, _dir) = create_db();
        let (mock_sender, call_counter) = MockPromptSender::new(domain_response(4));
        let exploration_service = ExplorationService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let first = coordinator
            .generate_domain_exploration("spark")
            .await
            .expect("domain generation failed");
        assert_eq!(first.len(), 4);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);

        // Second call should use cache
        let second = coordinator
            .generate_domain_exploration("spark")
            .await
            .expect("domain generation failed");
        assert_eq!(second.len(), 4);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn usage_pipeline_reuses_cache() {
        let (db, _dir) = create_db();
        let (mock_sender, call_counter) = MockPromptSender::new(usage_response());
        let exploration_service = ExplorationService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let first = coordinator
            .generate_usage_patterns("spark")
            .await
            .expect("usage generation failed");
        assert_eq!(first.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);

        let second = coordinator
            .generate_usage_patterns("spark")
            .await
            .expect("usage generation failed");
        assert_eq!(second.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn custom_examples_require_context() {
        let (db, _dir) = create_db();
        let (mock_sender, _counter) = MockPromptSender::new(custom_examples_response());
        let exploration_service = ExplorationService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let err = coordinator
            .generate_custom_examples("spark", "   ")
            .await
            .expect_err("expected validation error");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn custom_examples_execute_with_context() {
        let (db, _dir) = create_db();
        let (mock_sender, call_counter) = MockPromptSender::new(custom_examples_response());
        let exploration_service = ExplorationService::new(Arc::new(mock_sender));
        let coordinator = coordinator(&db, &exploration_service, "openai");

        let result = coordinator
            .generate_custom_examples("spark", "innovation lab")
            .await
            .expect("custom examples failed");
        assert_eq!(result.len(), 1);
        assert_eq!(call_counter.load(Ordering::SeqCst), 1);
    }
}
