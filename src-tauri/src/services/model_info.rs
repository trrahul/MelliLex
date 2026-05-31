use crate::services::ai_provider::AiModel;

/// Known OpenAI chat models, ordered by priority (first = highest / default).
/// Fastest models come first so they are the default selection.
/// IDs double as prefixes — versioned API IDs like "gpt-5-2025-12-11"
/// match "gpt-5" via longest-prefix lookup.
const KNOWN_MODELS: &[(&str, &str)] = &[
    ("gpt-5-mini", "GPT-5 Mini"),
    ("gpt-5", "GPT-5"),
    ("gpt-5.1", "GPT-5.1"),
    ("gpt-5.2", "GPT-5.2"),
    ("gpt-5.2-pro", "GPT-5.2 Pro"),
    ("gpt-5.3", "GPT-5.3"),
    ("gpt-5.4", "GPT-5.4"),
    ("gpt-5.4-mini", "GPT-5.4 Mini"),
    ("gpt-5.5", "GPT-5.5"),
    ("gpt-5.5-pro", "GPT-5.5 Pro"),
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
        assert!(ModelInfoService::is_useful_chat_model("gpt-5"));
        assert!(ModelInfoService::is_useful_chat_model("gpt-5-mini"));
        assert!(ModelInfoService::is_useful_chat_model("gpt-5.2-pro"));
        assert!(!ModelInfoService::is_useful_chat_model("gpt-4o"));
        assert!(!ModelInfoService::is_useful_chat_model("gpt-3.5-turbo"));
        assert!(!ModelInfoService::is_useful_chat_model("o3"));
        assert!(!ModelInfoService::is_useful_chat_model("gpt-5-mini-transcribe"));
        assert!(!ModelInfoService::is_useful_chat_model("gpt-5-realtime-preview"));
        assert!(!ModelInfoService::is_useful_chat_model("dall-e-3"));
        assert!(!ModelInfoService::is_useful_chat_model("whisper-1"));
    }

    #[test]
    fn priority_matches_list_order() {
        for (idx, (id, _)) in KNOWN_MODELS.iter().enumerate() {
            assert_eq!(
                ModelInfoService::get_model_priority(id),
                idx as u32,
                "priority for {} should equal its list index",
                id
            );
        }

        assert_eq!(ModelInfoService::get_model_priority("gpt-5-mini"), 0);
    }

    #[test]
    fn versioned_ids_resolve_to_friendly_name() {
        let model = ModelInfoService::create_ai_model("gpt-5-mini-2025-09-01".to_string());
        assert_eq!(model.name, "GPT-5 Mini");

        let model = ModelInfoService::create_ai_model("gpt-5.2-2025-12-11".to_string());
        assert_eq!(model.name, "GPT-5.2");
    }

    #[test]
    fn unknown_model_uses_id_as_name() {
        let model = ModelInfoService::create_ai_model("some-new-model".to_string());
        assert_eq!(model.name, "some-new-model");
    }
}
