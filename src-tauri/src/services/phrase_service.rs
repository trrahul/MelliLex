use crate::models::{
    PhraseSection1Overview, PhraseSection2Context, PhraseSection3Related, TokenUsage,
};
use crate::services::ai_provider::PromptSender;
use crate::services::prompt_manager::PromptManager;
use crate::services::response_parser::ResponseParser;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct PhraseService {
    ai_provider: Arc<dyn PromptSender>,
    prompt_manager: PromptManager,
}

impl PhraseService {
    pub fn new(provider: Arc<dyn PromptSender>) -> Self {
        PhraseService {
            ai_provider: provider,
            prompt_manager: PromptManager::new(),
        }
    }

    pub async fn generate_section1_overview(
        &self,
        phrase: &str,
        language: &str,
    ) -> Result<(PhraseSection1Overview, Option<TokenUsage>)> {
        log::debug!(
            "Generating phrase section 1 (overview) for: '{}' in {}",
            phrase,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_language("phrase_section1_overview", phrase, language)
            .ok_or_else(|| anyhow::anyhow!("Template 'phrase_section1_overview' not found"))?;

        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let overview: PhraseSection1Overview =
            ResponseParser::parse_partial(&response, "phrase_section1_overview")?;

        log::info!(
            "Phrase Section 1 completed: type={:?}, region={:?}",
            overview.phrase_type,
            overview.region
        );
        log_token_usage(&token_usage, "Phrase Section 1 (overview)");

        Ok((overview, token_usage))
    }

    pub async fn generate_section2_context(
        &self,
        phrase: &str,
        language: &str,
    ) -> Result<(PhraseSection2Context, Option<TokenUsage>)> {
        log::debug!(
            "Generating phrase section 2 (context) for: '{}' in {}",
            phrase,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_language("phrase_section2_context", phrase, language)
            .ok_or_else(|| anyhow::anyhow!("Template 'phrase_section2_context' not found"))?;

        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let context: PhraseSection2Context =
            ResponseParser::parse_partial(&response, "phrase_section2_context")?;

        log::info!(
            "Phrase Section 2 completed: {} usage notes, {} mistakes",
            context.usage_notes.len(),
            context.common_mistakes.len()
        );
        log_token_usage(&token_usage, "Phrase Section 2 (context)");

        Ok((context, token_usage))
    }

    pub async fn generate_section3_related(
        &self,
        phrase: &str,
        language: &str,
    ) -> Result<(PhraseSection3Related, Option<TokenUsage>)> {
        log::debug!(
            "Generating phrase section 3 (related) for: '{}' in {}",
            phrase,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_language("phrase_section3_related", phrase, language)
            .ok_or_else(|| anyhow::anyhow!("Template 'phrase_section3_related' not found"))?;

        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let related: PhraseSection3Related =
            ResponseParser::parse_partial(&response, "phrase_section3_related")?;

        log::info!(
            "Phrase Section 3 completed: {} variations, {} similar, {} opposite",
            related.variations.len(),
            related.similar_phrases.len(),
            related.opposite_phrases.len()
        );
        log_token_usage(&token_usage, "Phrase Section 3 (related)");

        Ok((related, token_usage))
    }
}

fn log_token_usage(usage: &Option<TokenUsage>, context: &str) {
    if let Some(tokens) = usage {
        log::info!(
            "{} - Total tokens: {} (prompt: {}, completion: {})",
            context,
            tokens.total_tokens,
            tokens.prompt_tokens,
            tokens.completion_tokens
        );
    }
}
