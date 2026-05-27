use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ai::client::LlmConfig;
use crate::error::{Result, SkillsMergeError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_strategy")]
    pub default_strategy: String,
    #[serde(default = "default_format")]
    pub default_output_format: String,
    #[serde(default)]
    pub auto_resolve: Option<String>,
    #[serde(default)]
    pub log_level: String,
    #[serde(default)]
    pub ai: AiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_strategy() -> String {
    "ai".to_string()
}

fn default_format() -> String {
    "markdown".to_string()
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4o".to_string()
}

fn default_temperature() -> f32 {
    0.3
}

fn default_max_tokens() -> u32 {
    4096
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: default_base_url(),
            model: default_model(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_strategy: default_strategy(),
            default_output_format: default_format(),
            auto_resolve: None,
            log_level: "info".to_string(),
            ai: AiConfig::default(),
        }
    }
}

impl AiConfig {
    /// Build LlmConfig from this AiConfig, resolving values from env if not set
    pub fn to_llm_config(&self) -> LlmConfig {
        let api_key = self
            .api_key
            .clone()
            .or_else(|| std::env::var("SKILLSMERGE_API_KEY").ok())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .unwrap_or_default();

        let base_url = std::env::var("SKILLSMERGE_BASE_URL")
            .ok()
            .unwrap_or_else(|| self.base_url.clone());

        let model = std::env::var("SKILLSMERGE_MODEL")
            .ok()
            .unwrap_or_else(|| self.model.clone());

        let temperature = std::env::var("SKILLSMERGE_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(self.temperature);

        let max_tokens = std::env::var("SKILLSMERGE_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(self.max_tokens);

        LlmConfig {
            api_key,
            base_url,
            model,
            temperature,
            max_tokens,
        }
    }
}

/// Load configuration from file
pub fn load_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(path).map_err(SkillsMergeError::IoError)?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("toml");

    match ext {
        "json" => serde_json::from_str(&content).map_err(|e| SkillsMergeError::ParseError {
            file: path.display().to_string(),
            reason: format!("Invalid JSON config: {}", e),
        }),
        "yaml" | "yml" => {
            serde_yaml::from_str(&content).map_err(|e| SkillsMergeError::ParseError {
                file: path.display().to_string(),
                reason: format!("Invalid YAML config: {}", e),
            })
        }
        _ => toml::from_str(&content).map_err(|e| SkillsMergeError::ParseError {
            file: path.display().to_string(),
            reason: format!("Invalid TOML config: {}", e),
        }),
    }
}

/// Save configuration to file
pub fn save_config(config: &Config, path: &Path) -> Result<()> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("toml");
    let content = match ext {
        "json" => {
            serde_json::to_string_pretty(config).map_err(|e| SkillsMergeError::ParseError {
                file: path.display().to_string(),
                reason: format!("Failed to serialize config: {}", e),
            })?
        }
        "yaml" | "yml" => {
            serde_yaml::to_string(config).map_err(|e| SkillsMergeError::ParseError {
                file: path.display().to_string(),
                reason: format!("Failed to serialize config: {}", e),
            })?
        }
        _ => toml::to_string_pretty(config).map_err(|e| SkillsMergeError::ParseError {
            file: path.display().to_string(),
            reason: format!("Failed to serialize config: {}", e),
        })?,
    };

    std::fs::write(path, content).map_err(SkillsMergeError::IoError)
}
