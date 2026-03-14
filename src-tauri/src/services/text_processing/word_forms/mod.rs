pub trait WordFormsAnalyzer: Send + Sync {
    fn get_variations(&self, word: &str) -> anyhow::Result<Vec<String>>;

    fn name(&self) -> &str;
}

pub mod stemmer_impl;

pub use stemmer_impl::StemmerAnalyzer;
