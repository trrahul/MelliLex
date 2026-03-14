pub mod spell_checker;
pub mod text_processor;
pub mod word_forms;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

pub use spell_checker::{SpellCheckResult, SpellChecker};
pub use text_processor::TextProcessor;
pub use word_forms::WordFormsAnalyzer;
