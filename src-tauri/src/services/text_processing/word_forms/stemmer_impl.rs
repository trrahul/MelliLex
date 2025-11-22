use crate::services::text_processing::WordFormsAnalyzer;
use anyhow::Result;
use rust_stemmers::{Algorithm, Stemmer};

pub struct StemmerAnalyzer {
    stemmer: Stemmer,
    language: String,
}

impl StemmerAnalyzer {
    pub fn new(language: &str) -> Result<Self> {
        let algorithm = Self::language_to_algorithm(language)?;
        let stemmer = Stemmer::create(algorithm);

        Ok(Self {
            stemmer,
            language: language.to_string(),
        })
    }

    pub fn english() -> Result<Self> {
        Self::new("english")
    }

    fn language_to_algorithm(language: &str) -> Result<Algorithm> {
        match language.to_lowercase().as_str() {
            "en" | "english" => Ok(Algorithm::English),
            _ => Err(anyhow::anyhow!(
                "Unsupported language for stemming: {}. Only English is supported.",
                language
            )),
        }
    }

    pub fn language(&self) -> &str {
        &self.language
    }
}

impl StemmerAnalyzer {
    fn get_lemma(&self, word: &str) -> String {
        let word_lower = word.to_lowercase();
        self.stemmer.stem(&word_lower).to_string()
    }
}

impl WordFormsAnalyzer for StemmerAnalyzer {
    fn get_variations(&self, word: &str) -> Result<Vec<String>> {
        let lemma = self.get_lemma(word);

        let mut variations = vec![lemma.clone()];

        let suffixes = [
            "s", "es", "ed", "ing", "er", "est", "tion", "sion", "ation", "ment", "ness", "ly",
            "ful", "less", "able", "ible",
        ];

        for suffix in &suffixes {
            variations.push(format!("{}{}", lemma, suffix));
        }

        // Handle double consonants (e.g., "run" -> "running")
        if lemma.len() >= 2 {
            let last_char = lemma.chars().last().unwrap();
            if last_char.is_alphabetic() && !matches!(last_char, 'a' | 'e' | 'i' | 'o' | 'u') {
                variations.push(format!("{}{}{}", lemma, last_char, "ing"));
                variations.push(format!("{}{}{}", lemma, last_char, "ed"));
            }
        }

        // Add original word if different from lemma
        let word_lower = word.to_lowercase();
        if word_lower != lemma {
            variations.push(word_lower);
        }

        // Deduplicate and sort
        variations.sort();
        variations.dedup();

        Ok(variations)
    }

    fn name(&self) -> &str {
        "Stemmer (Porter/Snowball)"
    }
}

impl Default for StemmerAnalyzer {
    fn default() -> Self {
        Self::english().expect("Failed to create default StemmerAnalyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stemmer_analyzer_creation() {
        let analyzer = StemmerAnalyzer::new("english");
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_stemmer_lemmatization() {
        let analyzer = StemmerAnalyzer::english().unwrap();

        assert_eq!(analyzer.get_lemma("running"), "run");
        assert_eq!(analyzer.get_lemma("run"), analyzer.get_lemma("Run"));
        assert_eq!(analyzer.get_lemma("run"), analyzer.get_lemma("RUN"));
        assert_eq!(analyzer.get_lemma("lamentation"), "lament");
    }

    #[test]
    fn test_stemmer_variations() {
        let analyzer = StemmerAnalyzer::english().unwrap();

        let variations = analyzer.get_variations("lament").unwrap();
        assert!(variations.contains(&"lament".to_string()));
        assert!(variations.contains(&"lamentation".to_string()));
    }




}
