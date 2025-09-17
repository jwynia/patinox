//! Anthropic provider implementation
//!
//! This module implements the LLM provider abstraction for Anthropic's Claude models,
//! supporting the latest Claude 3 family and following Anthropic's API patterns.

use super::error::{ProviderError, ProviderResult};
use super::secret::SecretString;
use super::types::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelCapabilities,
    ModelId, ModelInfo, QualityTier, SpeedTier, StreamingResponse, Usage,
};
use super::ModelProvider;
use async_trait::async_trait;
use reqwest::{header::HeaderMap, Client};
use serde::{Deserialize, Serialize};

/// Default maximum tokens for completion requests when not specified
const DEFAULT_MAX_TOKENS: usize = 1024;

/// Anthropic API provider for Claude models
#[derive(Debug)]
pub struct AnthropicProvider {
    /// HTTP client for API requests
    client: Client,
    /// API key for authentication
    api_key: SecretString,
    /// Base URL for API (defaults to Anthropic's)
    base_url: String,
    /// API version (Anthropic requires this)
    version: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: impl Into<SecretString>) -> Result<Self, ProviderError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(ProviderError::ConfigurationError(
                "Anthropic API key cannot be empty".to_string(),
            ));
        }

        let client = Client::new();

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            version: "2023-06-01".to_string(),
        })
    }

    /// Set a custom base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set a custom API version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Get the current base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the current API version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Create headers for Anthropic API requests
    pub fn create_headers(&self) -> Result<HeaderMap, ProviderError> {
        let mut headers = HeaderMap::new();

        // Anthropic uses x-api-key header
        headers.insert(
            "x-api-key",
            self.api_key.expose_secret().parse().map_err(|e| {
                ProviderError::ConfigurationError(format!("Invalid API key format: {}", e))
            })?,
        );

        // Anthropic requires anthropic-version header
        headers.insert(
            "anthropic-version",
            self.version.parse().map_err(|e| {
                ProviderError::ConfigurationError(format!("Invalid version format: {}", e))
            })?,
        );

        // Content-Type for JSON
        headers.insert(
            "content-type",
            "application/json".parse().map_err(|e| {
                ProviderError::ConfigurationError(format!("Invalid content type: {}", e))
            })?,
        );

        Ok(headers)
    }

    /// Convert our request format to Anthropic's format
    pub fn convert_to_anthropic_format(
        &self,
        request: &CompletionRequest,
    ) -> Result<AnthropicRequest, ProviderError> {
        // Convert messages to Anthropic format
        let messages = self.convert_messages(&request.messages)?;

        Ok(AnthropicRequest {
            model: request.model.name().to_string(),
            max_tokens: request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS) as u32,
            temperature: request.temperature,
            messages,
            tools: None, // TODO: Implement tool support
            stop_sequences: None,
            top_p: None,
            top_k: None,
        })
    }

    /// Convert message strings to Anthropic message format
    fn convert_messages(
        &self,
        messages: &[String],
    ) -> Result<Vec<AnthropicMessage>, ProviderError> {
        if messages.is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Messages cannot be empty".to_string(),
            ));
        }

        let mut anthropic_messages = Vec::new();
        let mut is_human = true; // Start with human message

        for message in messages {
            let role = if is_human { "human" } else { "assistant" };

            anthropic_messages.push(AnthropicMessage {
                role: role.to_string(),
                content: vec![AnthropicContent {
                    content_type: "text".to_string(),
                    text: message.clone(),
                }],
            });

            is_human = !is_human; // Alternate roles
        }

        Ok(anthropic_messages)
    }

    /// Parse Anthropic response format
    pub fn parse_anthropic_response(
        &self,
        response_json: &str,
    ) -> Result<CompletionResponse, ProviderError> {
        let anthropic_response: AnthropicResponse =
            serde_json::from_str(response_json).map_err(|e| {
                ProviderError::SerializationError(format!(
                    "Failed to parse Anthropic response: {}",
                    e
                ))
            })?;

        // Extract text content
        let content = anthropic_response
            .content
            .into_iter()
            .find(|c| c.content_type == "text")
            .map(|c| c.text)
            .unwrap_or_default();

        let usage = Usage {
            prompt_tokens: anthropic_response.usage.input_tokens as usize,
            completion_tokens: anthropic_response.usage.output_tokens as usize,
            total_tokens: (anthropic_response.usage.input_tokens
                + anthropic_response.usage.output_tokens) as usize,
        };

        Ok(CompletionResponse {
            model: ModelId::new(anthropic_response.model),
            content,
            usage: Some(usage),
            finish_reason: anthropic_response
                .stop_reason
                .unwrap_or("end_turn".to_string()),
        })
    }

    /// Get model capabilities based on model name
    fn get_model_capabilities_by_name(&self, model_name: &str) -> ModelCapabilities {
        match model_name {
            name if name.contains("claude-3-opus") => ModelCapabilities {
                max_tokens: 200_000,
                supports_tools: true,
                supports_vision: true,
                supports_streaming: false, // Start with basic implementation
                input_cost_per_1k: Some(15.0),
                output_cost_per_1k: Some(75.0),
                speed_tier: SpeedTier::Standard,
                quality_tier: QualityTier::Ultra,
            },
            name if name.contains("claude-3-sonnet") => ModelCapabilities {
                max_tokens: 200_000,
                supports_tools: true,
                supports_vision: true,
                supports_streaming: false,
                input_cost_per_1k: Some(3.0),
                output_cost_per_1k: Some(15.0),
                speed_tier: SpeedTier::Fast,
                quality_tier: QualityTier::Premium,
            },
            name if name.contains("claude-3-haiku") => ModelCapabilities {
                max_tokens: 200_000,
                supports_tools: true,
                supports_vision: true,
                supports_streaming: false,
                input_cost_per_1k: Some(0.25),
                output_cost_per_1k: Some(1.25),
                speed_tier: SpeedTier::Fast,
                quality_tier: QualityTier::Standard,
            },
            _ => ModelCapabilities::default(),
        }
    }
}

/// Anthropic API request format
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
}

/// Anthropic message format
#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: Vec<AnthropicContent>,
}

/// Anthropic content format
#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Anthropic tool definition (for future use)
#[derive(Debug, Serialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Anthropic API response format
#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<AnthropicContent>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

/// Anthropic usage format
#[derive(Debug, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        // Anthropic doesn't provide a models endpoint like OpenAI
        // We'll return a hardcoded list of available Claude models
        let models = vec![
            ModelInfo {
                id: ModelId::new("claude-3-opus-20240229"),
                name: "claude-3-opus-20240229".to_string(),
                capabilities: self.get_model_capabilities_by_name("claude-3-opus-20240229"),
            },
            ModelInfo {
                id: ModelId::new("claude-3-sonnet-20240229"),
                name: "claude-3-sonnet-20240229".to_string(),
                capabilities: self.get_model_capabilities_by_name("claude-3-sonnet-20240229"),
            },
            ModelInfo {
                id: ModelId::new("claude-3-haiku-20240307"),
                name: "claude-3-haiku-20240307".to_string(),
                capabilities: self.get_model_capabilities_by_name("claude-3-haiku-20240307"),
            },
        ];

        Ok(models)
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let url = format!("{}/v1/messages", self.base_url);
        let headers = self.create_headers()?;

        // Convert to Anthropic format
        let anthropic_request = self.convert_to_anthropic_format(&request)?;

        // Make the request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            return match status.as_u16() {
                400 => Err(ProviderError::InvalidRequest(format!(
                    "Bad request: {}",
                    error_text
                ))),
                401 => Err(ProviderError::AuthenticationError(
                    "Invalid API key".to_string(),
                )),
                429 => Err(ProviderError::RateLimited { retry_after: None }),
                _ => Err(ProviderError::ApiError(format!(
                    "HTTP {}: {}",
                    status, error_text
                ))),
            };
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        self.parse_anthropic_response(&response_text)
    }

    async fn stream_completion(
        &self,
        _request: CompletionRequest,
    ) -> ProviderResult<StreamingResponse> {
        // TODO: Implement streaming for Anthropic provider
        Err(ProviderError::ApiError(
            "Streaming not yet implemented for Anthropic provider".to_string(),
        ))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        // Anthropic doesn't provide embedding endpoints (as of 2024)
        Err(ProviderError::InvalidRequest(
            "Anthropic does not support embedding endpoints".to_string(),
        ))
    }

    async fn supports_model(&self, model: &ModelId) -> bool {
        model.name().contains("claude")
    }

    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        if self.supports_model(model).await {
            Some(self.get_model_capabilities_by_name(model.name()))
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AnthropicProvider::new("sk-test-key");
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_creation_empty_key() {
        let provider = AnthropicProvider::new("");
        assert!(provider.is_err());
    }

    #[test]
    fn test_message_conversion() {
        let provider = AnthropicProvider::new("sk-test-key").unwrap();
        let messages = vec!["Hello".to_string(), "Hi there!".to_string()];

        let result = provider.convert_messages(&messages);
        assert!(result.is_ok());

        let anthropic_messages = result.unwrap();
        assert_eq!(anthropic_messages.len(), 2);
        assert_eq!(anthropic_messages[0].role, "human");
        assert_eq!(anthropic_messages[1].role, "assistant");
    }

    #[test]
    fn test_model_capabilities() {
        let provider = AnthropicProvider::new("sk-test-key").unwrap();

        let opus_caps = provider.get_model_capabilities_by_name("claude-3-opus-20240229");
        assert_eq!(opus_caps.quality_tier, QualityTier::Ultra);

        let haiku_caps = provider.get_model_capabilities_by_name("claude-3-haiku-20240307");
        assert_eq!(haiku_caps.speed_tier, SpeedTier::Fast);
    }
}
