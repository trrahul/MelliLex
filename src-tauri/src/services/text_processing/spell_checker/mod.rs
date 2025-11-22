use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SpellCheckResult {
    pub is_correct: bool,
    pub original: String,
    pub suggestions: Vec<String>,
}

pub trait SpellChecker: Send + Sync {
    fn check(&self, word: &str) -> Result<SpellCheckResult>;

    fn name(&self) -> &str;
}

pub mod symspell_impl;

pub use symspell_impl::SymSpellChecker;
