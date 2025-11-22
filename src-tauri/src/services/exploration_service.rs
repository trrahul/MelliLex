use crate::models::*;
use crate::services::ai_provider::PromptSender;
use crate::services::prompt_manager::PromptManager;
use crate::services::response_parser::ResponseParser;
use anyhow::Result;
use std::sync::Arc;

pub struct ExplorationService {
    ai_provider: Arc<dyn PromptSender>,
    prompt_manager: PromptManager,
}

#[derive(Clone, Copy, Debug)]
pub struct DomainPromptLimits {
    pub max_domains: usize,
    pub max_examples_per_domain: usize,
    pub max_collocations_per_domain: usize,
}

impl DomainPromptLimits {
    pub const fn new(
        max_domains: usize,
        max_examples_per_domain: usize,
        max_collocations_per_domain: usize,
    ) -> Self {
        Self {
            max_domains,
            max_examples_per_domain,
            max_collocations_per_domain,
        }
    }

    pub const fn compact() -> Self {
        Self::new(4, 3, 4)
    }
}

impl ExplorationService {
    pub fn new(provider: Arc<dyn PromptSender>) -> Self {
        ExplorationService {
            ai_provider: provider,
            prompt_manager: PromptManager::new(),
        }
    }

    async fn generate_formality_analysis(
        &self,
        word: &str,
        language: &str,
    ) -> Result<(FormalityAnalysisResult, Option<TokenUsage>)> {
        log::debug!(
            "Generating formality analysis for: {} in {}",
            word,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_language("formality_analysis", word, language)
            .ok_or_else(|| anyhow::anyhow!("formality_analysis template not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let result: FormalityAnalysisResult =
            ResponseParser::parse_partial(&response, "formality analysis")?;
        log::info!(
            "Formality analysis completed: {}% formality, {} alternatives",
            result.formality_percentage,
            result.formality_alternatives.len()
        );

        Ok((result, token_usage))
    }

    async fn generate_usage_patterns(
        &self,
        word: &str,
        language: &str,
    ) -> Result<(UsagePatternsResult, Option<TokenUsage>)> {
        log::debug!("Generating usage patterns for: {} in {}", word, language);

        let prompt = self
            .prompt_manager
            .render_with_language("usage_patterns", word, language)
            .ok_or_else(|| anyhow::anyhow!("usage_patterns template not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let result: UsagePatternsResult =
            ResponseParser::parse_partial(&response, "usage patterns")?;
        log::info!(
            "Usage patterns generated: {} patterns, {} examples",
            result.usage_patterns.len(),
            result.contextual_examples.len()
        );

        Ok((result, token_usage))
    }

    async fn generate_domain_exploration(
        &self,
        word: &str,
        limits: DomainPromptLimits,
        language: &str,
    ) -> Result<(DomainExplorationResult, Option<TokenUsage>)> {
        log::debug!(
            "Generating domain exploration for: {} in {}",
            word,
            language
        );

        let mut prompt = self
            .prompt_manager
            .render_with_language("domain_exploration", word, language)
            .ok_or_else(|| anyhow::anyhow!("domain_exploration template not found"))?;

        // Replace limit placeholders
        prompt = prompt
            .replace("{{max_domains}}", &limits.max_domains.to_string())
            .replace(
                "{{max_examples}}",
                &limits.max_examples_per_domain.to_string(),
            )
            .replace(
                "{{max_collocations}}",
                &limits.max_collocations_per_domain.to_string(),
            );

        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let result: DomainExplorationResult =
            ResponseParser::parse_partial(&response, "domain exploration")?;
        log::info!(
            "Domain exploration generated: {} domains",
            result.domain_explorations.len()
        );

        Ok((result, token_usage))
    }

    pub async fn generate_practice_exercises(
        &self,
        word: &str,
        count: usize,
        language: &str,
    ) -> Result<(PracticeExercisesResult, Option<TokenUsage>)> {
        log::debug!(
            "Generating {} practice exercises for: {} in {}",
            count,
            word,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_language("practice_exercises", word, language)
            .ok_or_else(|| anyhow::anyhow!("practice_exercises template not found"))?;
        let (response, token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let result: PracticeExercisesResult =
            ResponseParser::parse_partial(&response, "practice exercises")?;
        log::info!(
            "Practice exercises generated: {} exercises",
            result.practice_exercises.len()
        );

        Ok((result, token_usage))
    }

    pub async fn generate_contextual_examples(
        &self,
        word: &str,
        context: &str,
        language: &str,
    ) -> Result<Vec<String>> {
        log::debug!(
            "Generating contextual examples for: {} in context: {} in language: {}",
            word,
            context,
            language
        );

        let prompt = self
            .prompt_manager
            .render_with_context_and_language("contextual_examples", word, context, language)
            .ok_or_else(|| anyhow::anyhow!("contextual_examples template not found"))?;
        let (response, _token_usage) = self.ai_provider.send_prompt(&prompt).await?;

        let result: ContextualExamplesResponse =
            ResponseParser::parse_partial(&response, "contextual examples")?;
        log::info!("Generated {} contextual examples", result.examples.len());

        Ok(result.examples)
    }

    pub async fn generate_formality_only(
        &self,
        word: &str,
        language: &str,
    ) -> Result<(FormalityAnalysisResult, Option<TokenUsage>)> {
        self.generate_formality_analysis(word, language).await
    }

    pub async fn generate_domains_only(
        &self,
        word: &str,
        limits: DomainPromptLimits,
        language: &str,
    ) -> Result<(DomainExplorationResult, Option<TokenUsage>)> {
        self.generate_domain_exploration(word, limits, language)
            .await
    }

    pub async fn generate_usage_only(
        &self,
        word: &str,
        language: &str,
    ) -> Result<(UsagePatternsResult, Option<TokenUsage>)> {
        self.generate_usage_patterns(word, language).await
    }

    pub async fn generate_practice_only(
        &self,
        word: &str,
        count: usize,
        language: &str,
    ) -> Result<(PracticeExercisesResult, Option<TokenUsage>)> {
        self.generate_practice_exercises(word, count, language)
            .await
    }
}

// Response models matching C# implementation

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormalityAnalysisResult {
    pub formality_percentage: f64,
    pub formality_alternatives: Vec<FormalityAlternative>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsagePatternsResult {
    pub usage_patterns: Vec<UsagePattern>,
    pub contextual_examples: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainExplorationResult {
    pub domain_explorations: Vec<DomainExploration>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PracticeExercisesResult {
    pub practice_exercises: Vec<PracticeExercise>,
}

#[derive(Debug, serde::Deserialize)]
struct ContextualExamplesResponse {
    examples: Vec<String>,
}
