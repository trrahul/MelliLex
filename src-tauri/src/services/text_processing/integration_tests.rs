#[cfg(test)]
mod text_processing_integration {
    use crate::services::text_processing::{
        spell_checker::SymSpellChecker, word_forms::StemmerAnalyzer, TextProcessor,
    };
    use std::sync::Arc;

    fn create_test_processor() -> anyhow::Result<TextProcessor> {
        let spell_checker = SymSpellChecker::new()?;
        let word_forms = StemmerAnalyzer::new("english")?;

        Ok(TextProcessor::new(
            Arc::new(spell_checker),
            Arc::new(word_forms),
        ))
    }

    #[test]
    fn test_processor_creation() {
        let processor = create_test_processor();
        assert!(
            processor.is_ok(),
            "Should create TextProcessor successfully"
        );
    }

    #[test]
    fn test_spell_check_correct_word() {
        let processor = create_test_processor().unwrap();
        let result = processor.preprocess_word("hello").unwrap();

        assert_eq!(result.original, "hello");
        assert!(!result.is_misspelled);
        assert!(result.corrected.is_none());
        assert!(result.suggestions.is_empty());
    }

    #[test]
    fn test_highlight_patterns_generation() {
        let processor = create_test_processor().unwrap();
        let patterns = processor.get_highlight_patterns("run").unwrap();

        assert!(!patterns.is_empty());
        assert!(patterns.contains(&"run".to_string()));
        // Should contain various forms
        assert!(patterns.len() > 5, "Should have multiple word forms");
    }

    #[test]
    fn test_word_form_detection_via_highlight() {
        let processor = create_test_processor().unwrap();

        let patterns = processor.get_highlight_patterns("run").unwrap();
        assert!(patterns.contains(&"run".to_string()));
        assert!(patterns.contains(&"running".to_string()));
        assert!(patterns.len() > 5, "Should have multiple word forms");
    }

    #[test]
    fn test_preprocessing_for_llm() {
        let processor = create_test_processor().unwrap();

        // Test correct word
        let result = processor.preprocess_word("lamentation").unwrap();
        assert_eq!(result.word_for_llm(), "lamentation");

        // Test misspelled word (if corrected)
        let result2 = processor.preprocess_word("lamantation").unwrap();
        if result2.is_misspelled && result2.corrected.is_some() {
            assert_eq!(result2.word_for_llm(), result2.corrected.as_ref().unwrap());
        }
    }

    #[test]
    fn test_complex_words() {
        let processor = create_test_processor().unwrap();

        let words = vec![
            "running",
            "walked",
            "happiness",
            "beautiful",
            "unbelievable",
            "internationalization",
        ];

        for word in words {
            let result = processor.preprocess_word(word);
            assert!(result.is_ok(), "Should process '{}' successfully", word);
        }
    }

    #[test]
    fn test_edge_cases() {
        let processor = create_test_processor().unwrap();

        // Empty string
        let result = processor.preprocess_word("");
        assert!(result.is_ok());

        // Single character
        let result = processor.preprocess_word("a");
        assert!(result.is_ok());

        // Numbers (should handle gracefully)
        let result = processor.preprocess_word("123");
        assert!(result.is_ok());

        // Special characters
        let result = processor.preprocess_word("hello-world");
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_processing() {
        let processor = create_test_processor().unwrap();

        let words = vec!["run", "walk", "jump", "swim", "fly"];
        let mut all_variations = Vec::new();

        for word in &words {
            let patterns = processor.get_highlight_patterns(word).unwrap();
            all_variations.extend(patterns);
        }

        assert!(
            all_variations.len() > words.len() * 5,
            "Should have many variations"
        );
    }

    #[test]
    fn test_word_for_llm_logic() {
        let processor = create_test_processor().unwrap();

        // Test that word_for_llm returns corrected version when available
        let result = processor.preprocess_word("test").unwrap();
        let llm_word = result.word_for_llm();

        if result.corrected.is_some() {
            assert_eq!(llm_word, result.corrected.as_ref().unwrap());
        } else {
            assert_eq!(llm_word, &result.original);
        }
    }
}
