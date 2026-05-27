use serde::{Deserialize, Serialize};
use crate::error::{Result, SkillsMergeError};

/// LLM API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_temperature() -> f32 {
    0.3
}

fn default_max_tokens() -> u32 {
    4096
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o".to_string(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

/// OpenAI Chat Completion request
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".to_string(), content: content.into() }
    }
}

/// OpenAI Chat Completion response
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

/// LLM Client
pub struct LlmClient {
    config: LlmConfig,
    http: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        let http = reqwest::Client::new();
        Self { config, http }
    }

    /// Send a chat completion request
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        if self.config.api_key.is_empty() {
            return Err(SkillsMergeError::ParseError {
                file: "AI config".to_string(),
                reason: "API key is not set. Please set SKILLSMERGE_API_KEY environment variable or configure in config file.".to_string(),
            });
        }

        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };

        let response = self.http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| SkillsMergeError::ParseError {
                file: "AI request".to_string(),
                reason: format!("Failed to send request: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(SkillsMergeError::ParseError {
                file: "AI response".to_string(),
                reason: format!("API error ({}): {}", status, body),
            });
        }

        let chat_response: ChatResponse = response.json().await.map_err(|e| {
            SkillsMergeError::ParseError {
                file: "AI response".to_string(),
                reason: format!("Failed to parse response: {}", e),
            }
        })?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| SkillsMergeError::ParseError {
                file: "AI response".to_string(),
                reason: "No response from AI model".to_string(),
            })
    }

    /// Simple chat with system + user message
    pub async fn ask(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_prompt),
        ];
        self.chat(messages).await
    }
}
