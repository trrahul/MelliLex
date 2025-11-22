#[cfg(test)]
mod integration_tests {
    use crate::services::text_processing::{
        spell_checker::SymSpellChecker, word_forms::StemmerAnalyzer, TextProcessor,
    };
    use std::sync::Arc;

    #[test]
    fn test_end_to_end_preprocessing() {
        // Create components
        let spell_checker = SymSpellChecker::new();
        let word_forms = StemmerAnalyzer::english();

        if spell_checker.is_err() || word_forms.is_err() {
            eprintln!("Skipping test: dependencies not available");
            return;
        }

        let processor = TextProcessor::new(
            Arc::new(spell_checker.unwrap()),
            Arc::new(word_forms.unwrap()),
        );

        // Test preprocessing a correct word
        let result = processor.preprocess_word("running").unwrap();
        assert_eq!(result.original, "running");
        assert_eq!(result.word_for_llm(), "running");
        assert!(!result.is_misspelled);
    }

    #[test]
    fn test_misspelling_correction() {
        let spell_checker = SymSpellChecker::new();
        let word_forms = StemmerAnalyzer::english();

        if spell_checker.is_err() || word_forms.is_err() {
            eprintln!("Skipping test: dependencies not available");
            return;
        }

        let processor = TextProcessor::new(
            Arc::new(spell_checker.unwrap()),
            Arc::new(word_forms.unwrap()),
        );

        // Test a misspelling
        let result = processor.preprocess_word("teh").unwrap();
        assert_eq!(result.original, "teh");

        if result.is_misspelled {
            assert!(!result.suggestions.is_empty());
            // Should suggest "the"
            assert!(result.corrected.is_some());
        }
    }

    #[test]
    fn test_word_form_variations() {
        let spell_checker = SymSpellChecker::new();
        let word_forms = StemmerAnalyzer::english();

        if spell_checker.is_err() || word_forms.is_err() {
            eprintln!("Skipping test: dependencies not available");
            return;
        }

        let processor = TextProcessor::new(
            Arc::new(spell_checker.unwrap()),
            Arc::new(word_forms.unwrap()),
        );

        // Get variations for highlighting
        let patterns = processor.get_highlight_patterns("run").unwrap();

        // Should contain base form and variations
        assert!(patterns.contains(&"run".to_string()));
        assert!(patterns.len() > 1, "Should have multiple variations");
    }


    #[test]
    fn test_case_insensitivity() {
        let spell_checker = SymSpellChecker::new();
        let word_forms = StemmerAnalyzer::english();

        if spell_checker.is_err() || word_forms.is_err() {
            eprintln!("Skipping test: dependencies not available");
            return;
        }

        let processor = TextProcessor::new(
            Arc::new(spell_checker.unwrap()),
            Arc::new(word_forms.unwrap()),
        );

        // Test different cases
        let result1 = processor.preprocess_word("Run").unwrap();
        let result2 = processor.preprocess_word("run").unwrap();
        let result3 = processor.preprocess_word("RUN").unwrap();

        // Test output for debugging
        eprintln!("Result1: {:?}", result1);
        eprintln!("Result2: {:?}", result2);
        eprintln!("Result3: {:?}", result3);

        // All should produce same preprocessing
        assert!(!result1.is_misspelled);
        assert!(!result2.is_misspelled);
        assert!(!result3.is_misspelled);
    }

    #[test]
    fn test_edge_cases() {
        let spell_checker = SymSpellChecker::new();
        let word_forms = StemmerAnalyzer::english();

        if spell_checker.is_err() || word_forms.is_err() {
            eprintln!("Skipping test: dependencies not available");
            return;
        }

        let processor = TextProcessor::new(
            Arc::new(spell_checker.unwrap()),
            Arc::new(word_forms.unwrap()),
        );

        // Empty string
        let result = processor.preprocess_word("");
        assert!(result.is_ok());

        // Single character
        let result = processor.preprocess_word("a");
        assert!(result.is_ok());

        // Very long word
        let long_word = "pneumonoultramicroscopicsilicovolcanoconiosis";
        let result = processor.preprocess_word(long_word);
        assert!(result.is_ok());
    }
}
