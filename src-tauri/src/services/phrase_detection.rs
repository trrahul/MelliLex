pub struct PhraseDetector;

impl PhraseDetector {
    pub fn normalize_phrase(input: &str) -> String {
        input.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::InputType;

    fn detect(input: &str) -> InputType {
        let word_count = input.split_whitespace().count();
        if word_count > 1 {
            InputType::Phrase
        } else {
            InputType::Word
        }
    }

    #[test]
    fn test_single_word() {
        assert_eq!(detect("hello"), InputType::Word);
        assert_eq!(detect("  hello  "), InputType::Word);
        assert_eq!(detect("HELLO"), InputType::Word);
    }

    #[test]
    fn test_hyphenated_word() {
        // Hyphenated words are still single words
        assert_eq!(detect("self-esteem"), InputType::Word);
        assert_eq!(detect("well-known"), InputType::Word);
        assert_eq!(detect("mother-in-law"), InputType::Word);
    }

    #[test]
    fn test_phrase() {
        assert_eq!(detect("break the ice"), InputType::Phrase);
        assert_eq!(detect("piece of cake"), InputType::Phrase);
        assert_eq!(detect("look up"), InputType::Phrase);
    }

    #[test]
    fn test_phrase_with_extra_whitespace() {
        assert_eq!(detect("  break   the   ice  "), InputType::Phrase);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(detect(""), InputType::Word);
        assert_eq!(detect("   "), InputType::Word);
    }

    #[test]
    fn test_normalize_phrase() {
        assert_eq!(
            PhraseDetector::normalize_phrase("  Break The ICE  "),
            "break the ice"
        );
        assert_eq!(
            PhraseDetector::normalize_phrase("PIECE   OF   CAKE"),
            "piece of cake"
        );
    }
}
