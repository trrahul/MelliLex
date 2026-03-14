use crate::errors::AppError;
use crate::models::PhraseDefinitionData;
use crate::utils::sync;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// Repository for caching phrase definitions
#[derive(Clone)]
pub struct PhraseCacheRepository {
    conn: Arc<Mutex<Connection>>,
}

impl PhraseCacheRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn cache_key(provider: &str, phrase: &str) -> String {
        format!("{}:phrase:{}", provider, phrase.to_lowercase())
    }

    pub fn get(&self, phrase: &str, provider: &str) -> Result<Option<PhraseDefinitionData>, AppError> {
        let cache_key = Self::cache_key(provider, phrase);
        let conn = sync::lock(&self.conn, "phrase cache connection")?;

        let result = conn.query_row(
            "SELECT definition FROM cached_phrases WHERE cache_key = ?1",
            [&cache_key],
            |row| {
                let json: String = row.get(0)?;
                Ok(json)
            },
        );

        match result {
            Ok(json) => {
                let data: PhraseDefinitionData = serde_json::from_str(&json).map_err(|e| {
                    AppError::Parse(format!("Failed to parse cached phrase definition: {}", e))
                })?;
                log::debug!("Cache hit for phrase: {}", phrase);
                Ok(Some(data))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                log::debug!("Cache miss for phrase: {}", phrase);
                Ok(None)
            }
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn cache(
        &self,
        phrase: &str,
        data: &PhraseDefinitionData,
        provider: &str,
    ) -> Result<(), AppError> {
        let cache_key = Self::cache_key(provider, phrase);
        let json = serde_json::to_string(data)
            .map_err(|e| AppError::Parse(format!("Failed to serialize phrase definition: {}", e)))?;
        let cached_at = chrono::Utc::now().timestamp();

        let conn = sync::lock(&self.conn, "phrase cache connection")?;
        conn.execute(
            "INSERT OR REPLACE INTO cached_phrases (cache_key, phrase, definition, provider, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&cache_key, phrase, &json, provider, cached_at],
        )
        .map_err(AppError::from)?;

        log::debug!("Cached phrase definition for: {}", phrase);
        Ok(())
    }

    pub fn clear_all(&self) -> Result<usize, AppError> {
        let conn = sync::lock(&self.conn, "phrase cache connection")?;
        let deleted = conn
            .execute("DELETE FROM cached_phrases", [])
            .map_err(AppError::from)?;

        log::info!("Cleared {} cached phrase definitions", deleted);
        Ok(deleted)
    }

    pub fn clear_old(&self, days: i64) -> Result<usize, AppError> {
        let conn = sync::lock(&self.conn, "phrase cache connection")?;
        let cutoff_time = chrono::Utc::now().timestamp() - (days * 86400);

        let deleted = conn
            .execute(
                "DELETE FROM cached_phrases WHERE cached_at < ?1",
                params![cutoff_time],
            )
            .map_err(AppError::from)?;

        log::info!("Cleared {} old cached phrase definitions", deleted);
        Ok(deleted)
    }

    pub fn get_stats(&self) -> Result<(usize, i64), AppError> {
        let conn = sync::lock(&self.conn, "phrase cache connection")?;

        let count: usize = conn
            .query_row("SELECT COUNT(*) FROM cached_phrases", [], |row| row.get(0))
            .map_err(AppError::from)?;

        let size: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(LENGTH(definition)), 0) FROM cached_phrases",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok((count, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        FormalityInfo, PhraseOrigin, PhraseRegion, PhraseSection1Overview, PhraseSection2Context,
        PhraseSection3Related, PhraseType,
    };
    use rusqlite::Connection;

    fn create_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE cached_phrases (
                cache_key TEXT PRIMARY KEY,
                phrase TEXT NOT NULL,
                definition TEXT NOT NULL,
                provider TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn create_test_phrase_data() -> PhraseDefinitionData {
        PhraseDefinitionData {
            section1: PhraseSection1Overview {
                phrase: "break the ice".to_string(),
                phrase_type: PhraseType::Idiom,
                tldr: "Start a conversation to ease tension".to_string(),
                literal_meaning: Some("Break frozen water".to_string()),
                actual_meaning: "Initiate conversation in a social setting".to_string(),
                formality: FormalityInfo {
                    level: "Neutral".to_string(),
                    percentage: 50,
                },
                region: PhraseRegion::Universal,
                token_usage: None,
            },
            section2: PhraseSection2Context {
                origin: PhraseOrigin {
                    story: "Originated from ships breaking ice in frozen waters".to_string(),
                    era: Some("17th century".to_string()),
                    source: Some("Maritime terminology".to_string()),
                    evolution: None,
                },
                usage_notes: vec![],
                common_mistakes: vec![],
                token_usage: None,
            },
            section3: PhraseSection3Related {
                variations: vec![],
                similar_phrases: vec![],
                opposite_phrases: vec![],
                see_also: vec![],
                token_usage: None,
            },
        }
    }

    #[test]
    fn test_cache_and_retrieve() {
        let conn = create_test_db();
        let repo = PhraseCacheRepository::new(conn);
        let data = create_test_phrase_data();

        // Cache the phrase
        repo.cache("break the ice", &data, "openai").unwrap();

        // Retrieve it
        let retrieved = repo.get("break the ice", "openai").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().section1.phrase, "break the ice");
    }

    #[test]
    fn test_cache_miss() {
        let conn = create_test_db();
        let repo = PhraseCacheRepository::new(conn);

        let retrieved = repo.get("unknown phrase", "openai").unwrap();
        assert!(retrieved.is_none());
    }
}
