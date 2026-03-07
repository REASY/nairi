use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub model_name: String,
    pub api_key: String,
    pub base_url: String,
    pub static_analysis_image: String,
    pub runtime_analysis_image: String,
    pub adb_connection_string: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model_name: std::env::var("AI_MODEL_NAME")
                .unwrap_or_else(|_| "gemini-2.5-pro".to_string()),
            api_key: std::env::var("GEMINI_API_KEY").unwrap_or_else(|_| "".to_string()),
            base_url: std::env::var("GOOGLE_GEMINI_BASE_URL")
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".to_string()),
            static_analysis_image: std::env::var("STATIC_ANALYSIS_IMAGE")
                .unwrap_or_else(|_| "nairi/static-analysis:dev".to_string()),
            runtime_analysis_image: std::env::var("RUNTIME_ANALYSIS_IMAGE")
                .unwrap_or_else(|_| "nairi/runtime-analysis:dev".to_string()),
            adb_connection_string: std::env::var("ADB_CONNECTION_STRING")
                .unwrap_or_else(|_| "host.docker.internal:15555".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub name: String,
    pub content: String,
}
