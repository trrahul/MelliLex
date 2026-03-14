pub struct JsonExtractor;

impl JsonExtractor {
    pub fn extract_json_segment(text: &str) -> Option<String> {
        Self::extract_between(text, '{', '}').or_else(|| Self::extract_between(text, '[', ']'))
    }

    pub fn extract_between(text: &str, open: char, close: char) -> Option<String> {
        let start = text.find(open)?;
        let end = text.rfind(close)?;
        if end < start {
            return None;
        }
        let slice = text[start..=end].trim();
        if slice.is_empty() {
            None
        } else {
            Some(slice.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_json_object() {
        let input = "Some text {\"key\": \"value\"} more text";
        let result = JsonExtractor::extract_json_segment(input);
        assert_eq!(result, Some(r#"{"key": "value"}"#.to_string()));
    }

    #[test]
    fn extract_json_array() {
        let input = "Data: [1, 2, 3] end";
        let result = JsonExtractor::extract_json_segment(input);
        assert_eq!(result, Some("[1, 2, 3]".to_string()));
    }

    #[test]
    fn extract_no_json() {
        let input = "No JSON here";
        let result = JsonExtractor::extract_json_segment(input);
        assert_eq!(result, None);
    }

    #[test]
    fn extract_between_valid() {
        let input = "prefix {content} suffix";
        let result = JsonExtractor::extract_between(input, '{', '}');
        assert_eq!(result, Some("{content}".to_string()));
    }

    #[test]
    fn extract_between_no_delimiters() {
        let input = "no delimiters";
        let result = JsonExtractor::extract_between(input, '{', '}');
        assert_eq!(result, None);
    }

    #[test]
    fn extract_between_reversed_delimiters() {
        let input = "} content {";
        let result = JsonExtractor::extract_between(input, '{', '}');
        assert_eq!(result, None);
    }

    #[test]
    fn extract_between_nested() {
        let input = "outer { inner { nested } } end";
        let result = JsonExtractor::extract_between(input, '{', '}');
        // Takes first '{' and last '}'
        assert_eq!(result, Some("{ inner { nested } }".to_string()));
    }
}
