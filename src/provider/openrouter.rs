//! OpenRouter provider implementation
//!
//! This module implements the LLM provider abstraction for OpenRouter,
//! which provides a unified API for accessing 100+ models from multiple providers
//! with automatic routing, fallbacks, and cost optimization.

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

/// OpenRouter API provider for universal model access
#[derive(Debug)]
pub struct OpenRouterProvider {
    /// HTTP client for API requests
    client: Client,
    /// API key for authentication
    api_key: SecretString,
    /// Base URL for API (defaults to OpenRouter's)
    base_url: String,
    /// Optional HTTP-Referer header for attribution
    referer: Option<String>,
    /// Optional X-Title header for app identification
    title: Option<String>,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider
    pub fn new(api_key: impl Into<SecretString>) -> Result<Self, ProviderError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(ProviderError::ConfigurationError(
                "OpenRouter API key cannot be empty".to_string(),
            ));
        }

        let client = Client::new();

        Ok(Self {
            client,
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            referer: None,
            title: None,
        })
    }

    /// Set a custom base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set HTTP-Referer header for attribution
    pub fn with_referer(mut self, referer: impl Into<String>) -> Self {
        self.referer = Some(referer.into());
        self
    }

    /// Set X-Title header for app identification
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Get the current base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the current HTTP-Referer header
    pub fn referer(&self) -> Option<&str> {
        self.referer.as_deref()
    }

    /// Get the current X-Title header
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Create headers for OpenRouter API requests
    pub fn create_headers(&self) -> Result<HeaderMap, ProviderError> {
        let mut headers = HeaderMap::new();

        // OpenRouter uses Bearer token authentication
        headers.insert(
            "authorization",
            format!("Bearer {}", self.api_key.expose_secret())
                .parse()
                .map_err(|e| {
                    ProviderError::ConfigurationError(format!("Invalid API key format: {}", e))
                })?,
        );

        // Optional HTTP-Referer for attribution
        if let Some(referer) = &self.referer {
            headers.insert(
                "http-referer",
                referer.parse().map_err(|e| {
                    ProviderError::ConfigurationError(format!("Invalid referer format: {}", e))
                })?,
            );
        }

        // Optional X-Title for app identification
        if let Some(title) = &self.title {
            headers.insert(
                "x-title",
                title.parse().map_err(|e| {
                    ProviderError::ConfigurationError(format!("Invalid title format: {}", e))
                })?,
            );
        }

        // Content-Type for JSON
        headers.insert(
            "content-type",
            "application/json".parse().map_err(|e| {
                ProviderError::ConfigurationError(format!("Invalid content type: {}", e))
            })?,
        );

        Ok(headers)
    }

    /// Convert our request format to OpenRouter's format (OpenAI-compatible)
    pub fn convert_to_openrouter_format(
        &self,
        request: &CompletionRequest,
    ) -> Result<OpenRouterRequest, ProviderError> {
        if request.messages.is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Messages cannot be empty".to_string(),
            ));
        }

        // Convert messages to OpenAI format
        let messages = self.convert_messages(&request.messages)?;

        // Handle provider preferences if specified
        let provider_config = if let Some(provider_hint) = request.model.provider_hint() {
            Some(OpenRouterProviderConfig {
                order: Some(vec![provider_hint.to_string()]),
                allow_fallbacks: true,
                require_parameters: false,
            })
        } else {
            Some(OpenRouterProviderConfig {
                order: None,
                allow_fallbacks: true,
                require_parameters: false,
            })
        };

        Ok(OpenRouterRequest {
            model: request.model.name().to_string(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens.map(|t| t as u32),
            provider: provider_config,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: Some(false), // Start with non-streaming
        })
    }

    /// Convert message strings to OpenAI message format
    fn convert_messages(
        &self,
        messages: &[String],
    ) -> Result<Vec<OpenRouterMessage>, ProviderError> {
        let mut openai_messages = Vec::new();
        let mut is_user = true; // Start with user message

        for message in messages {
            let role = if is_user { "user" } else { "assistant" };

            openai_messages.push(OpenRouterMessage {
                role: role.to_string(),
                content: message.clone(),
            });

            is_user = !is_user; // Alternate roles
        }

        Ok(openai_messages)
    }

    /// Parse OpenRouter response format (OpenAI-compatible)
    pub fn parse_openrouter_response(
        &self,
        response_json: &str,
    ) -> Result<CompletionResponse, ProviderError> {
        let openrouter_response: OpenRouterResponse =
            serde_json::from_str(response_json).map_err(|e| {
                ProviderError::SerializationError(format!(
                    "Failed to parse OpenRouter response: {}",
                    e
                ))
            })?;

        // Extract content from the first choice
        let content = openrouter_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        let usage = Usage {
            prompt_tokens: openrouter_response.usage.prompt_tokens as usize,
            completion_tokens: openrouter_response.usage.completion_tokens as usize,
            total_tokens: openrouter_response.usage.total_tokens as usize,
        };

        let finish_reason = openrouter_response
            .choices
            .first()
            .and_then(|choice| choice.finish_reason.clone())
            .unwrap_or("stop".to_string());

        Ok(CompletionResponse {
            model: ModelId::new(openrouter_response.model),
            content,
            usage: Some(usage),
            finish_reason,
        })
    }

    /// Get model capabilities based on model name and provider
    fn get_model_capabilities_by_name(&self, model_name: &str) -> ModelCapabilities {
        // Handle provider/model format
        let (provider, model) = if model_name.contains('/') {
            let parts: Vec<&str> = model_name.splitn(2, '/').collect();
            (Some(parts[0]), parts[1])
        } else {
            (None, model_name)
        };

        match (provider, model) {
            // Anthropic models via OpenRouter
            (Some("anthropic") | None, name) if name.contains("claude-3-opus") => {
                ModelCapabilities {
                    max_tokens: 200_000,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    input_cost_per_1k: Some(15.0),
                    output_cost_per_1k: Some(75.0),
                    speed_tier: SpeedTier::Standard,
                    quality_tier: QualityTier::Ultra,
                }
            }
            (Some("anthropic") | None, name) if name.contains("claude-3-sonnet") => {
                ModelCapabilities {
                    max_tokens: 200_000,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    input_cost_per_1k: Some(3.0),
                    output_cost_per_1k: Some(15.0),
                    speed_tier: SpeedTier::Fast,
                    quality_tier: QualityTier::Premium,
                }
            }
            (Some("anthropic") | None, name) if name.contains("claude-3-haiku") => {
                ModelCapabilities {
                    max_tokens: 200_000,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    input_cost_per_1k: Some(0.25),
                    output_cost_per_1k: Some(1.25),
                    speed_tier: SpeedTier::Fast,
                    quality_tier: QualityTier::Standard,
                }
            }
            // OpenAI models via OpenRouter
            (Some("openai") | None, name) if name.contains("gpt-4") => ModelCapabilities {
                max_tokens: if name.contains("turbo") {
                    128_000
                } else {
                    8_192
                },
                supports_tools: true,
                supports_vision: name.contains("vision"),
                supports_streaming: true,
                input_cost_per_1k: Some(10.0),
                output_cost_per_1k: Some(30.0),
                speed_tier: SpeedTier::Standard,
                quality_tier: QualityTier::Ultra,
            },
            (Some("openai") | None, name) if name.contains("gpt-3.5") => ModelCapabilities {
                max_tokens: 4_096,
                supports_tools: true,
                supports_vision: false,
                supports_streaming: true,
                input_cost_per_1k: Some(1.5),
                output_cost_per_1k: Some(2.0),
                speed_tier: SpeedTier::Fast,
                quality_tier: QualityTier::Standard,
            },
            // Google models via OpenRouter
            (Some("google") | None, name) if name.contains("gemini-pro") => ModelCapabilities {
                max_tokens: 32_768,
                supports_tools: true,
                supports_vision: name.contains("vision"),
                supports_streaming: true,
                input_cost_per_1k: Some(0.5),
                output_cost_per_1k: Some(1.5),
                speed_tier: SpeedTier::Fast,
                quality_tier: QualityTier::Premium,
            },
            // Default capabilities for unknown models
            _ => ModelCapabilities::default(),
        }
    }

    /// Check if this model is supported by OpenRouter
    fn is_supported_model(&self, model_name: &str) -> bool {
        // OpenRouter supports a wide variety of models
        // Check for known patterns
        let known_providers = ["anthropic", "openai", "google", "meta", "mistral", "cohere"];
        let known_models = [
            "claude", "gpt", "gemini", "llama", "mistral", "command", "palm", "text-", "ada",
            "babbage", "curie", "davinci",
        ];

        // If it has a provider prefix, check if it's a known provider
        if model_name.contains('/') {
            let provider = model_name.split('/').next().unwrap_or("");
            return known_providers.contains(&provider);
        }

        // Check if it matches known model patterns
        known_models
            .iter()
            .any(|&pattern| model_name.to_lowercase().contains(pattern))
    }
}

/// OpenRouter API request format (OpenAI-compatible with extensions)
#[derive(Debug, Serialize)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<OpenRouterProviderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// OpenRouter provider configuration for routing preferences
#[derive(Debug, Serialize)]
pub struct OpenRouterProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    pub allow_fallbacks: bool,
    pub require_parameters: bool,
}

/// OpenRouter message format (OpenAI-compatible)
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterMessage {
    pub role: String,
    pub content: String,
}

/// OpenRouter API response format (OpenAI-compatible)
#[derive(Debug, Deserialize)]
pub struct OpenRouterResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenRouterChoice>,
    pub usage: OpenRouterUsage,
}

/// OpenRouter choice format
#[derive(Debug, Deserialize)]
pub struct OpenRouterChoice {
    pub index: u32,
    pub message: OpenRouterMessage,
    pub finish_reason: Option<String>,
}

/// OpenRouter usage format
#[derive(Debug, Deserialize)]
pub struct OpenRouterUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenRouter models response format
#[derive(Debug, Deserialize)]
pub struct OpenRouterModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

/// OpenRouter model information
#[derive(Debug, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub pricing: OpenRouterPricing,
    pub context_length: Option<u32>,
    pub architecture: Option<OpenRouterArchitecture>,
    pub top_provider: Option<OpenRouterTopProvider>,
}

/// OpenRouter pricing information
#[derive(Debug, Deserialize)]
pub struct OpenRouterPricing {
    pub prompt: String,
    pub completion: String,
}

/// OpenRouter architecture information
#[derive(Debug, Deserialize)]
pub struct OpenRouterArchitecture {
    pub modality: String,
    pub tokenizer: String,
    pub instruct_type: Option<String>,
}

/// OpenRouter top provider information
#[derive(Debug, Deserialize)]
pub struct OpenRouterTopProvider {
    pub max_completion_tokens: Option<u32>,
    pub is_moderated: Option<bool>,
}

#[async_trait]
impl ModelProvider for OpenRouterProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        let url = format!("{}/models", self.base_url);
        let headers = self.create_headers()?;

        let response = self.client.get(&url).headers(headers).send().await?;

        let models_response: OpenRouterModelsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

        // Convert OpenRouter models to our format
        let models = models_response
            .data
            .into_iter()
            .map(|model| {
                let capabilities = self.get_model_capabilities_by_name(&model.id);
                ModelInfo {
                    id: ModelId::new(model.id.clone()),
                    name: model.id,
                    capabilities,
                }
            })
            .collect();

        Ok(models)
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let headers = self.create_headers()?;

        // Convert to OpenRouter format
        let openrouter_request = self.convert_to_openrouter_format(&request)?;

        // Make the request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&openrouter_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
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

        self.parse_openrouter_response(&response_text)
    }

    async fn stream_completion(
        &self,
        _request: CompletionRequest,
    ) -> ProviderResult<StreamingResponse> {
        // TODO: Implement streaming for OpenRouter provider
        Err(ProviderError::ApiError(
            "Streaming not yet implemented for OpenRouter provider".to_string(),
        ))
    }

    async fn embed(&self, request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        let url = format!("{}/embeddings", self.base_url);
        let headers = self.create_headers()?;

        let embed_request = serde_json::json!({
            "model": request.model.name(),
            "input": request.input,
        });

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&embed_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
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

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

        // Parse OpenAI-format embedding response
        let embeddings: Vec<Vec<f64>> = response_json["data"]
            .as_array()
            .ok_or_else(|| {
                ProviderError::SerializationError("Invalid embeddings response format".to_string())
            })?
            .iter()
            .map(|item| {
                item["embedding"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_f64())
                    .collect()
            })
            .collect();

        let usage = response_json.get("usage").map(|usage_obj| Usage {
            prompt_tokens: usage_obj["prompt_tokens"].as_u64().unwrap_or(0) as usize,
            completion_tokens: 0,
            total_tokens: usage_obj["total_tokens"].as_u64().unwrap_or(0) as usize,
        });

        Ok(EmbeddingResponse {
            embeddings,
            model: request.model,
            usage,
        })
    }

    async fn supports_model(&self, model: &ModelId) -> bool {
        self.is_supported_model(model.name())
    }

    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        if self.supports_model(model).await {
            Some(self.get_model_capabilities_by_name(model.name()))
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "openrouter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = OpenRouterProvider::new("sk-or-test-key");
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_creation_empty_key() {
        let provider = OpenRouterProvider::new("");
        assert!(provider.is_err());
    }

    #[test]
    fn test_message_conversion() {
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();
        let messages = vec!["Hello".to_string(), "Hi there!".to_string()];

        let result = provider.convert_messages(&messages);
        assert!(result.is_ok());

        let openrouter_messages = result.unwrap();
        assert_eq!(openrouter_messages.len(), 2);
        assert_eq!(openrouter_messages[0].role, "user");
        assert_eq!(openrouter_messages[1].role, "assistant");
    }

    #[test]
    fn test_model_capabilities() {
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        let opus_caps = provider.get_model_capabilities_by_name("anthropic/claude-3-opus-20240229");
        assert_eq!(opus_caps.quality_tier, QualityTier::Ultra);

        let gpt_caps = provider.get_model_capabilities_by_name("openai/gpt-4");
        assert_eq!(gpt_caps.quality_tier, QualityTier::Ultra);

        let haiku_caps = provider.get_model_capabilities_by_name("claude-3-haiku-20240307");
        assert_eq!(haiku_caps.speed_tier, SpeedTier::Fast);
    }

    #[test]
    fn test_model_support() {
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        assert!(provider.is_supported_model("anthropic/claude-3-opus"));
        assert!(provider.is_supported_model("openai/gpt-4"));
        assert!(provider.is_supported_model("google/gemini-pro"));
        assert!(provider.is_supported_model("claude-3-sonnet-20240229"));
        assert!(provider.is_supported_model("gpt-3.5-turbo"));

        assert!(!provider.is_supported_model("fake/nonexistent-model-12345"));
    }

    #[test]
    fn test_custom_headers() {
        let provider = OpenRouterProvider::new("sk-or-test-key")
            .unwrap()
            .with_referer("https://myapp.com")
            .with_title("MyApp");

        assert_eq!(provider.referer(), Some("https://myapp.com"));
        assert_eq!(provider.title(), Some("MyApp"));
    }
}
