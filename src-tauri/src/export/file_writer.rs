use crate::errors::AppError;
use std::path::{Path, PathBuf};

/// Single Responsibility: Write markdown content to filesystem
pub struct FileSystemWriter;

impl FileSystemWriter {
    pub fn write_markdown(
        content: &str,
        word: &str,
        downloads_dir: &Path,
    ) -> Result<PathBuf, AppError> {
        let file_stem = Self::sanitize_filename(word);
        let path = downloads_dir.join(format!("{}-dictionary.md", file_stem));
        std::fs::write(&path, content)?;
        Ok(path)
    }

    pub fn sanitize_filename(word: &str) -> String {
        let cleaned: String = word
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();

        let collapsed = cleaned
            .split('-')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        if collapsed.is_empty() {
            "word".into()
        } else {
            collapsed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            FileSystemWriter::sanitize_filename("hello world"),
            "hello-world"
        );
        assert_eq!(FileSystemWriter::sanitize_filename("test@#$%"), "test");
        assert_eq!(FileSystemWriter::sanitize_filename(""), "word");
    }
}
