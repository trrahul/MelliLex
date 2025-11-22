use std::borrow::Cow;

pub struct ResponseSanitizer;

impl ResponseSanitizer {
    pub fn sanitize(response: &str) -> Cow<'_, str> {
        let trimmed = response.trim();

        if trimmed.is_empty() {
            return Cow::Borrowed(trimmed);
        }

        // Try stripping code fence first
        if let Some(mut stripped) = Self::strip_code_fence(trimmed) {
            if !Self::starts_like_json(&stripped) {
                // Extract JSON segment if still not JSON-like
                if let Some(extracted) = super::JsonExtractor::extract_json_segment(&stripped) {
                    stripped = extracted;
                }
            }
            return Cow::Owned(stripped);
        }

        // No code fence, but might still need extraction
        if !Self::starts_like_json(trimmed) {
            if let Some(extracted) = super::JsonExtractor::extract_json_segment(trimmed) {
                return Cow::Owned(extracted);
            }
        }

        Cow::Borrowed(trimmed)
    }

    fn starts_like_json(text: &str) -> bool {
        text.starts_with('{') || text.starts_with('[') || text.starts_with('"')
    }

    fn strip_code_fence(text: &str) -> Option<String> {
        if !text.starts_with("```") {
            return None;
        }

        let rest = &text[3..];
        let content = if let Some(end_idx) = rest.rfind("```") {
            &rest[..end_idx]
        } else {
            rest
        };

        let content = content.trim();
        let content = Self::strip_language_prefix(content);
        let cleaned = content.trim();

        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned.to_string())
        }
    }

    /// Strips language prefix lines (json, JSON, etc.) from code fence content
    fn strip_language_prefix(content: &str) -> &str {
        let content = content.trim_start_matches(['\n', '\r']);
        if Self::starts_like_json(content) {
            return content;
        }

        // Check if first line is a language identifier
        if let Some(idx) = content.find('\n') {
            let first_line = content[..idx].trim_matches('\r').trim();
            if first_line.is_empty()
                || first_line
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '#'))
            {
                let remainder = &content[idx + 1..];
                return remainder.trim_start_matches(['\n', '\r']);
            }
        }

        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_plain_json() {
        let input = r#"{"key": "value"}"#;
        let result = ResponseSanitizer::sanitize(input);
        assert_eq!(result, input);
    }

    #[test]
    fn sanitize_with_code_fence() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        let result = ResponseSanitizer::sanitize(input);
        assert_eq!(result, r#"{"key": "value"}"#);
    }

    #[test]
    fn sanitize_with_uppercase_language() {
        let input = "```JSON\r\n{\"key\": \"value\"}\n```";
        let result = ResponseSanitizer::sanitize(input);
        assert_eq!(result, r#"{"key": "value"}"#);
    }

    #[test]
    fn sanitize_with_surrounding_text() {
        let input = "Here is the data:\n{\"key\": \"value\"}\nEnd";
        let result = ResponseSanitizer::sanitize(input);
        assert_eq!(result, r#"{"key": "value"}"#);
    }

    #[test]
    fn sanitize_empty_string() {
        let result = ResponseSanitizer::sanitize("");
        assert_eq!(result, "");
    }

    #[test]
    fn sanitize_code_fence_without_closing() {
        let input = "```json\n{\"key\": \"value\"}";
        let result = ResponseSanitizer::sanitize(input);
        assert_eq!(result, r#"{"key": "value"}"#);
    }

    #[test]
    fn starts_like_json_detects_object() {
        assert!(ResponseSanitizer::starts_like_json("{"));
    }

    #[test]
    fn starts_like_json_detects_array() {
        assert!(ResponseSanitizer::starts_like_json("["));
    }

    #[test]
    fn starts_like_json_detects_string() {
        assert!(ResponseSanitizer::starts_like_json("\""));
    }

    #[test]
    fn starts_like_json_rejects_text() {
        assert!(!ResponseSanitizer::starts_like_json("text"));
    }
}
