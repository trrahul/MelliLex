use crate::constants::explore_features;
use crate::db::Database;
use crate::errors::AppError;
use crate::models::{
    CachedExploreFeatures, CachedFormalityData, CachedPracticeData, DomainExploration, UsagePattern,
};
use log::{debug, warn};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone, Copy)]
pub struct ExploreCacheRepository<'a> {
    db: &'a Database,
}

impl<'a> ExploreCacheRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn fetch_feature<T: DeserializeOwned>(
        &self,
        word: &str,
        provider_scope: &str,
        feature: &str,
    ) -> Result<Option<T>, AppError> {
        let result = self
            .db
            .get_cached_exploration_feature::<T>(word, provider_scope, feature)?;
        log_cache_state(word, feature, result.is_some());
        Ok(result)
    }

    pub fn store_feature<T: Serialize>(
        &self,
        word: &str,
        provider_scope: &str,
        feature: &str,
        data: &T,
    ) {
        match self
            .db
            .cache_exploration_feature(word, provider_scope, feature, data)
        {
            Ok(_) => debug!(
                "Cached exploration feature '{}' for '{}' (scope='{}')",
                feature, word, provider_scope
            ),
            Err(e) => warn!(
                "Failed to cache exploration feature '{}' for '{}': {}",
                feature, word, e
            ),
        }
    }

    pub fn load_all(
        &self,
        word: &str,
        provider_scope: &str,
    ) -> Result<CachedExploreFeatures, AppError> {
        let formality = self.fetch_feature::<CachedFormalityData>(
            word,
            provider_scope,
            explore_features::FORMALITY,
        )?;
        let domains = self.fetch_feature::<Vec<DomainExploration>>(
            word,
            provider_scope,
            explore_features::DOMAINS,
        )?;
        let usage =
            self.fetch_feature::<Vec<UsagePattern>>(word, provider_scope, explore_features::USAGE)?;
        let practice = self.fetch_feature::<CachedPracticeData>(
            word,
            provider_scope,
            explore_features::PRACTICE,
        )?;

        Ok(CachedExploreFeatures {
            formality,
            domains,
            usage,
            practice,
        })
    }
}

fn log_cache_state(word: &str, feature: &str, hit: bool) {
    if hit {
        debug!("Cache hit for '{}' feature on '{}'", feature, word);
    } else {
        debug!("Cache miss for '{}' feature on '{}'", feature, word);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CachedFormalityData, FormalityAlternative, FormalityLevel, PracticeExercise};
    use tempfile::TempDir;

    fn create_db() -> (Database, TempDir) {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = dir.path().join("test.db");
        let db = Database::new(path).expect("Failed to create database");
        (db, dir)
    }

    #[test]
    fn load_all_returns_cached_entries() {
        let (db, _dir) = create_db();
        let repo = ExploreCacheRepository::new(&db);
        let provider_scope = "openai::english";

        let formality = CachedFormalityData {
            formality_percentage: 42.0,
            formality_alternatives: vec![FormalityAlternative {
                word: "ignite".into(),
                level: FormalityLevel::Neutral,
                context: "example".into(),
                explanation: "details".into(),
            }],
        };
        repo.store_feature(
            "spark",
            provider_scope,
            explore_features::FORMALITY,
            &formality,
        );

        let practice = CachedPracticeData {
            practice_exercises: vec![PracticeExercise {
                question: "Pick one".into(),
                options: vec!["spark".into(), "ignite".into()],
                correct_answer: "spark".into(),
                explanation: "because".into(),
                exercise_type: "multiple-choice".into(),
                is_answered: false,
                user_answer: String::new(),
            }],
        };
        repo.store_feature(
            "spark",
            provider_scope,
            explore_features::PRACTICE,
            &practice,
        );

        let cached = repo
            .load_all("spark", provider_scope)
            .expect("failed to load cache");

        assert!(cached.formality.is_some());
        assert_eq!(cached.formality.unwrap().formality_percentage, 42.0);
        assert!(cached.practice.is_some());
        assert_eq!(cached.practice.unwrap().practice_exercises.len(), 1);
        assert!(cached.domains.is_none());
        assert!(cached.usage.is_none());
    }

    #[test]
    fn fetch_feature_reports_cache_state() {
        let (db, _dir) = create_db();
        let repo = ExploreCacheRepository::new(&db);
        let provider_scope = "anthropic::spanish";

        let result = repo
            .fetch_feature::<CachedPracticeData>(
                "absent",
                provider_scope,
                explore_features::PRACTICE,
            )
            .expect("fetch should succeed");
        assert!(result.is_none());

        let practice = CachedPracticeData {
            practice_exercises: vec![],
        };
        repo.store_feature(
            "absent",
            provider_scope,
            explore_features::PRACTICE,
            &practice,
        );

        let result = repo
            .fetch_feature::<CachedPracticeData>(
                "absent",
                provider_scope,
                explore_features::PRACTICE,
            )
            .expect("fetch should succeed");
        assert!(result.is_some());
    }
}
