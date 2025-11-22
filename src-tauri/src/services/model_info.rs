use crate::services::ai_provider::AiModel;

/// Known OpenAI chat models, ordered by priority (first = highest).
/// IDs double as prefixes — versioned API IDs like "gpt-4o-2024-08-06"
/// match "gpt-4o" via longest-prefix lookup.
const KNOWN_MODELS: &[(&str, &str)] = &[
    ("gpt-5.2-pro", "GPT-5.2 Pro"),
    ("gpt-5.2", "GPT-5.2"),
    ("gpt-5.1", "GPT-5.1"),
    ("gpt-5-mini", "GPT-5 Mini"),
    ("gpt-5-nano", "GPT-5 Nano"),
    ("gpt-5", "GPT-5"),
    ("gpt-4.1-mini", "GPT-4.1 Mini"),
    ("gpt-4.1-nano", "GPT-4.1 Nano"),
    ("gpt-4.1", "GPT-4.1"),
    ("chatgpt-4o-latest", "ChatGPT-4o (Latest)"),
    ("gpt-4o-mini", "GPT-4o Mini"),
    ("gpt-4o", "GPT-4o"),
    ("o4-mini", "O4 Mini"),
    ("o3-pro", "O3 Pro"),
    ("o3-mini", "O3 Mini"),
    ("o3", "O3"),
    ("o1-preview", "O1 Preview"),
    ("o1-mini", "O1 Mini"),
    ("o1", "O1"),
    ("gpt-4-turbo", "GPT-4 Turbo"),
    ("gpt-4-32k", "GPT-4 32K"),
    ("gpt-4", "GPT-4"),
    ("gpt-3.5-turbo-16k", "GPT-3.5 Turbo 16K"),
    ("gpt-3.5-turbo", "GPT-3.5 Turbo"),
];

const SPECIALIZED_KEYWORDS: &[&str] = &[
    "audio", "realtime", "transcribe", "tts", "speech", "computer-use",
];

pub struct ModelInfoService;

impl ModelInfoService {
    /// Find the best matching known model for an API id (longest prefix wins).
    fn lookup(id: &str) -> Option<(usize, &'static str)> {
        let normalized = id.trim().to_ascii_lowercase();
        if SPECIALIZED_KEYWORDS.iter().any(|kw| normalized.contains(kw)) {
            return None;
        }
        KNOWN_MODELS
            .iter()
            .enumerate()
            .filter(|(_, (prefix, _))| normalized.starts_with(prefix))
            .max_by_key(|(_, (prefix, _))| prefix.len())
            .map(|(idx, (_, name))| (idx, *name))
    }

    pub fn is_useful_chat_model(id: &str) -> bool {
        Self::lookup(id).is_some()
    }

    pub fn get_model_priority(id: &str) -> u32 {
        Self::lookup(id).map(|(idx, _)| idx as u32).unwrap_or(999)
    }

    pub fn create_ai_model(id: String) -> AiModel {
        let name = Self::lookup(&id)
            .map(|(_, name)| name.to_string())
            .unwrap_or_else(|| id.clone());
        AiModel { name, id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_useful_chat_models() {
        assert!(ModelInfoService::is_useful_chat_model("gpt-4o"));
        assert!(ModelInfoService::is_useful_chat_model("gpt-3.5-turbo"));
        assert!(ModelInfoService::is_useful_chat_model("o1-preview"));
        assert!(!ModelInfoService::is_useful_chat_model("gpt-4o-mini-transcribe"));
        assert!(!ModelInfoService::is_useful_chat_model("gpt-4o-realtime-preview"));
        assert!(!ModelInfoService::is_useful_chat_model("dall-e-3"));
        assert!(!ModelInfoService::is_useful_chat_model("whisper-1"));
    }

    #[test]
    fn priority_respects_list_order() {
        assert!(
            ModelInfoService::get_model_priority("gpt-5.2")
                < ModelInfoService::get_model_priority("gpt-5.1")
        );
        assert!(
            ModelInfoService::get_model_priority("gpt-4o")
                < ModelInfoService::get_model_priority("gpt-4")
        );
        assert!(
            ModelInfoService::get_model_priority("gpt-4o-mini")
                < ModelInfoService::get_model_priority("gpt-4o")
        );
    }

    #[test]
    fn versioned_ids_resolve_to_friendly_name() {
        let model = ModelInfoService::create_ai_model("gpt-4o-mini-2024-07-18".to_string());
        assert_eq!(model.name, "GPT-4o Mini");

        let model = ModelInfoService::create_ai_model("gpt-5.2-2025-12-11".to_string());
        assert_eq!(model.name, "GPT-5.2");
    }

    #[test]
    fn unknown_model_uses_id_as_name() {
        let model = ModelInfoService::create_ai_model("some-new-model".to_string());
        assert_eq!(model.name, "some-new-model");
    }
}
