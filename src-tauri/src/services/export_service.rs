use crate::db::Database;
use crate::errors::AppError;
use crate::export::{CapacitiesExporter, FileSystemWriter, MarkdownFormatter};
use std::path::{Path, PathBuf};

/// Single Responsibility: Coordinate word export operations using specialized components
pub struct ExportService<'a> {
    db: &'a Database,
}

impl<'a> ExportService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn export_word_to_markdown(
        &self,
        word: &str,
        provider: &str,
        downloads_dir: &Path,
        include_timestamp: bool,
    ) -> Result<PathBuf, AppError> {
        // Get cached definition
        let data = self
            .db
            .get_cached_word_progressive(word, provider)
            .map_err(|e| {
                AppError::NotFound(format!("Definition not found for '{}': {}", word, e))
            })?;

        let data = data.ok_or_else(|| {
            AppError::NotFound(format!("No cached definition found for '{}'", word))
        })?;

        // Generate markdown using formatter
        let content = MarkdownFormatter::format(word, &data, include_timestamp);

        // Save to file using writer
        FileSystemWriter::write_markdown(&content, word, downloads_dir)
    }

    pub fn export_phrase_to_markdown(
        &self,
        phrase: &str,
        provider: &str,
        downloads_dir: &Path,
        include_timestamp: bool,
    ) -> Result<PathBuf, AppError> {
        // Get cached phrase definition
        let data = self
            .db
            .get_cached_phrase(phrase, provider)
            .map_err(|e| {
                AppError::NotFound(format!("Phrase definition not found for '{}': {}", phrase, e))
            })?;

        let data = data.ok_or_else(|| {
            AppError::NotFound(format!("No cached definition found for phrase '{}'", phrase))
        })?;

        // Generate markdown using formatter
        let content = MarkdownFormatter::format_phrase(phrase, &data, include_timestamp);

        // Save to file using writer (use phrase as filename)
        FileSystemWriter::write_markdown(&content, phrase, downloads_dir)
    }

    pub async fn export_to_capacities(
        api_token: &str,
        space_id: &str,
        markdown: &str,
        no_timestamp: bool,
    ) -> Result<(), AppError> {
        CapacitiesExporter::export(api_token, space_id, markdown, no_timestamp).await
    }
}
