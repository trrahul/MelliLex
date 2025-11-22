use log::LevelFilter;
use std::env;
use tauri::Wry;
use tauri_plugin_log::{Builder, RotationStrategy, Target, TargetKind};

const ENV_LOG_LEVEL: &str = "MELLILEX_LOG";

#[derive(Debug, Clone, Copy)]
pub struct LogLevelConfig {
    level: LevelFilter,
}

impl LogLevelConfig {
    pub fn from_env() -> Self {
        let level = env::var(ENV_LOG_LEVEL)
            .ok()
            .and_then(|value| match value.to_uppercase().as_str() {
                "TRACE" => Some(LevelFilter::Trace),
                "DEBUG" => Some(LevelFilter::Debug),
                "INFO" => Some(LevelFilter::Info),
                "WARN" => Some(LevelFilter::Warn),
                "ERROR" => Some(LevelFilter::Error),
                _ => None,
            })
            .unwrap_or(LevelFilter::Info);
        Self { level }
    }

    pub fn level(&self) -> LevelFilter {
        self.level
    }
}

pub fn build_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    let config = LogLevelConfig::from_env();

    Builder::new()
        .max_file_size(100_000)
        .rotation_strategy(RotationStrategy::KeepAll)
        .level(config.level())
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir {
                file_name: Some("mellilex".to_string()),
            }),
            Target::new(TargetKind::Webview),
        ])
        .build()
}
