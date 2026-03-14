use crate::errors::AppError;
use crate::models::WordHistory;
use crate::utils::sync;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct HistoryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl HistoryRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn add(&self, word: &str, ai_provider: &str) -> Result<WordHistory, AppError> {
        let conn = sync::lock(&self.conn, "history connection")?;
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO word_history (id, word, timestamp, ai_provider)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, word, timestamp, ai_provider],
        )
        .map_err(AppError::from)?;

        Ok(WordHistory {
            id,
            word: word.to_string(),
            timestamp,
            ai_provider: ai_provider.to_string(),
        })
    }

    pub fn list(&self, limit: i32) -> Result<Vec<WordHistory>, AppError> {
        let conn = sync::lock(&self.conn, "history connection")?;
        let mut stmt = conn
            .prepare(
                "SELECT id, word, timestamp, ai_provider
             FROM word_history
             ORDER BY timestamp DESC
             LIMIT ?1",
            )
            .map_err(AppError::from)?;

        let rows = stmt
            .query_map([limit], |row| {
                Ok(WordHistory {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    timestamp: row.get(2)?,
                    ai_provider: row.get(3)?,
                })
            })
            .map_err(AppError::from)?;

        let mut history = Vec::new();
        for entry in rows {
            history.push(entry.map_err(AppError::from)?);
        }

        Ok(history)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let conn = sync::lock(&self.conn, "history connection")?;
        let rows = conn
            .execute("DELETE FROM word_history WHERE id = ?1", params![id])
            .map_err(AppError::from)?;

        if rows == 0 {
            return Err(AppError::not_found("History item not found"));
        }

        Ok(())
    }

    pub fn clear(&self) -> Result<(), AppError> {
        let conn = sync::lock(&self.conn, "history connection")?;
        conn.execute("DELETE FROM word_history", [])
            .map_err(AppError::from)?;
        Ok(())
    }
}
