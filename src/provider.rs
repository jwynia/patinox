//! LLM Provider abstraction
//!
//! Minimal provider system supporting multiple LLM backends.
//! Starts simple, can be enhanced later with retry logic, rate limiting, etc.

use serde::{Deserialize, Serialize};
use std::env;

/// Provider result type
pub type ProviderResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// OpenAI (GPT models)
    OpenAI,
    /// Anthropic (Claude models)
    Anthropic,
    /// Ollama (local models)
    Ollama,
}

impl Provider {
    /// Get default model for this provider
    pub fn default_model(&self) -> &'static str {
        match self {
            Provider::OpenAI => "gpt-4o-mini",
            Provider::Anthropic => "claude-3-haiku-20240307",
            Provider::Ollama => "llama3.1:8b",
        }
    }

    /// Get API key environment variable name
    pub fn api_key_env(&self) -> Option<&'static str> {
        match self {
            Provider::OpenAI => Some("OPENAI_API_KEY"),
            Provider::Anthropic => Some("ANTHROPIC_API_KEY"),
            Provider::Ollama => None, // Local, no key needed
        }
    }
}

/// Configuration for an LLM provider
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider: Provider,
    pub model: String,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
}

impl ProviderConfig {
    /// Create a new provider config with defaults
    pub fn new(provider: Provider) -> Self {
        let api_key = provider
            .api_key_env()
            .and_then(|env_var| env::var(env_var).ok());

        Self {
            provider,
            model: provider.default_model().to_string(),
            api_key,
            temperature: Some(0.7),
            max_tokens: Some(1000),
        }
    }

    /// Set the model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the temperature
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = Some(tokens);
        self
    }
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}

/// LLM Provider trait - implement this to add new providers
pub trait LLMProvider: Send + Sync {
    /// Send a completion request and get a response
    fn complete(&self, messages: Vec<Message>) -> ProviderResult<String>;
}

/// Mock provider for testing (no API calls)
pub struct MockProvider {
    response: String,
}

impl MockProvider {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

impl LLMProvider for MockProvider {
    fn complete(&self, _messages: Vec<Message>) -> ProviderResult<String> {
        Ok(self.response.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_defaults() {
        assert_eq!(Provider::OpenAI.default_model(), "gpt-4o-mini");
        assert_eq!(Provider::Anthropic.default_model(), "claude-3-haiku-20240307");
    }

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig::new(Provider::OpenAI)
            .model("gpt-4o")
            .temperature(0.5);

        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.temperature, Some(0.5));
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_mock_provider() {
        let provider = MockProvider::new("test response");
        let result = provider.complete(vec![Message::user("test")]).unwrap();
        assert_eq!(result, "test response");
    }
}
