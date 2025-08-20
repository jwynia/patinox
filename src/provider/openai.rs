//! OpenAI provider implementation

use super::error::{ProviderError, ProviderResult};
use super::secret::SecretString;
use super::types::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelCapabilities,
    ModelId, ModelInfo, QualityTier, SpeedTier, Usage,
};
use super::ModelProvider;
use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

/// OpenAI API provider
#[derive(Debug)]
pub struct OpenAIProvider {
    /// HTTP client for API requests
    client: Client,
    /// API key for authentication
    api_key: SecretString,
    /// Optional organization ID
    organization: Option<String>,
    /// Base URL for API (defaults to OpenAI's)
    base_url: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: impl Into<SecretString>) -> Result<Self, ProviderError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(ProviderError::ConfigurationError(
                "OpenAI API key cannot be empty".to_string(),
            ));
        }

        let client = Client::new();

        Ok(Self {
            client,
            api_key,
            organization: None,
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    /// Set the organization ID
    pub fn with_organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    /// Set a custom base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Create headers for OpenAI API requests
    fn create_headers(&self) -> Result<header::HeaderMap, ProviderError> {
        let mut headers = header::HeaderMap::new();

        // Authorization header
        let auth_value = format!("Bearer {}", self.api_key.expose_secret());
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value).map_err(|e| {
                ProviderError::ConfigurationError(format!("Invalid API key: {}", e))
            })?,
        );

        // Organization header (if provided)
        if let Some(org) = &self.organization {
            headers.insert(
                "OpenAI-Organization",
                header::HeaderValue::from_str(org).map_err(|e| {
                    ProviderError::ConfigurationError(format!("Invalid organization: {}", e))
                })?,
            );
        }

        // Content-Type
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        Ok(headers)
    }
}

/// OpenAI API request structure for completions
#[derive(Debug, Serialize)]
struct OpenAICompletionRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

/// OpenAI message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response structure
#[derive(Debug, Deserialize)]
struct OpenAICompletionResponse {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI choice structure
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    #[allow(dead_code)]
    index: u32,
    message: OpenAIMessage,
    finish_reason: String,
}

/// OpenAI usage statistics
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

/// OpenAI models list response
#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    #[allow(dead_code)]
    object: String,
    data: Vec<OpenAIModel>,
}

/// OpenAI model information
#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    owned_by: String,
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        let url = format!("{}/models", self.base_url);
        let headers = self.create_headers()?;

        let response = self.client.get(&url).headers(headers).send().await?;

        let models_response: OpenAIModelsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

        // Convert OpenAI models to our format
        let models = models_response
            .data
            .into_iter()
            .map(|model| {
                let capabilities = self.get_model_capabilities_by_name(&model.id);
                let name = model.id.clone();
                ModelInfo {
                    id: ModelId::new(model.id),
                    name,
                    capabilities,
                }
            })
            .collect();

        Ok(models)
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let headers = self.create_headers()?;

        // Convert our format to OpenAI format
        let messages: Vec<OpenAIMessage> = request
            .messages
            .into_iter()
            .enumerate()
            .map(|(i, content)| OpenAIMessage {
                role: if i == 0 {
                    "user".to_string()
                } else {
                    "assistant".to_string()
                },
                content,
            })
            .collect();

        let openai_request = OpenAICompletionRequest {
            model: request.model.name().to_string(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools: None, // TODO: Implement tool calling
        };

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let completion_response: OpenAICompletionResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

        // Convert to our format
        let choice = completion_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| ProviderError::ApiError("No choices in response".to_string()))?;

        let usage = completion_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(CompletionResponse {
            model: ModelId::new(completion_response.model),
            content: choice.message.content,
            usage,
            finish_reason: choice.finish_reason,
        })
    }

    async fn embed(&self, request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        let url = format!("{}/embeddings", self.base_url);
        let headers = self.create_headers()?;

        let embed_request = serde_json::json!({
            "model": request.model.name(),
            "input": request.input
        });

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&embed_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(format!(
                "OpenAI embedding API error {}: {}",
                status, error_text
            )));
        }

        let embed_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?;

        // Extract embeddings from response
        let data = embed_response["data"].as_array().ok_or_else(|| {
            ProviderError::ApiError("Invalid embedding response format".to_string())
        })?;

        let embeddings: Result<Vec<Vec<f64>>, ProviderError> = data
            .iter()
            .map(|item| {
                let embedding_vec = item["embedding"]
                    .as_array()
                    .ok_or_else(|| ProviderError::ApiError("Invalid embedding data".to_string()))?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0))
                    .collect::<Vec<f64>>();
                Ok(embedding_vec)
            })
            .collect();

        let embeddings = embeddings?;

        // Extract usage if present
        let usage = embed_response["usage"].as_object().map(|usage_obj| Usage {
            prompt_tokens: usage_obj["prompt_tokens"].as_u64().unwrap_or(0) as usize,
            completion_tokens: usage_obj["completion_tokens"].as_u64().unwrap_or(0) as usize,
            total_tokens: usage_obj["total_tokens"].as_u64().unwrap_or(0) as usize,
        });

        Ok(EmbeddingResponse {
            embeddings,
            model: request.model,
            usage,
        })
    }

    async fn supports_model(&self, model: &ModelId) -> bool {
        // For now, check against a hardcoded list of known OpenAI models
        let known_models = [
            "gpt-4",
            "gpt-4-turbo",
            "gpt-4-turbo-preview",
            "gpt-3.5-turbo",
            "gpt-3.5-turbo-16k",
            "text-embedding-ada-002",
            "text-embedding-3-small",
            "text-embedding-3-large",
        ];

        known_models.contains(&model.name())
    }

    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        Some(self.get_model_capabilities_by_name(model.name()))
    }

    fn name(&self) -> &str {
        "openai"
    }
}

impl OpenAIProvider {
    /// Get model capabilities based on model name
    fn get_model_capabilities_by_name(&self, model_name: &str) -> ModelCapabilities {
        match model_name {
            "gpt-4" | "gpt-4-turbo" | "gpt-4-turbo-preview" => ModelCapabilities {
                max_tokens: 128000,
                supports_tools: true,
                supports_vision: model_name.contains("turbo"),
                supports_streaming: true,
                input_cost_per_1k: Some(0.01),
                output_cost_per_1k: Some(0.03),
                speed_tier: SpeedTier::Standard,
                quality_tier: QualityTier::Ultra,
            },
            "gpt-3.5-turbo" | "gpt-3.5-turbo-16k" => ModelCapabilities {
                max_tokens: if model_name.contains("16k") {
                    16384
                } else {
                    4096
                },
                supports_tools: true,
                supports_vision: false,
                supports_streaming: true,
                input_cost_per_1k: Some(0.001),
                output_cost_per_1k: Some(0.002),
                speed_tier: SpeedTier::Fast,
                quality_tier: QualityTier::Standard,
            },
            name if name.starts_with("text-embedding") => ModelCapabilities {
                max_tokens: 8192,
                supports_tools: false,
                supports_vision: false,
                supports_streaming: false,
                input_cost_per_1k: Some(0.0001),
                output_cost_per_1k: None,
                speed_tier: SpeedTier::Fast,
                quality_tier: QualityTier::Standard,
            },
            _ => ModelCapabilities::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation_and_configuration() {
        // Test successful creation with valid key
        let provider = OpenAIProvider::new("sk-test-key");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.base_url, "https://api.openai.com/v1");

        // Test that configuration affects behavior
        let custom_provider = OpenAIProvider::new("sk-another-key")
            .unwrap()
            .with_base_url("https://custom.api.com/v1")
            .with_organization("my-org");

        // Verify configurations are different and affect behavior
        assert_ne!(provider.base_url, custom_provider.base_url);
        assert!(provider.organization.is_none());
        assert!(custom_provider.organization.is_some());

        // Test that both configurations are valid for the same interface
        assert_eq!(provider.name(), custom_provider.name());
    }

    #[test]
    fn test_openai_provider_empty_key() {
        let provider = OpenAIProvider::new("");
        assert!(provider.is_err());

        match provider.unwrap_err() {
            ProviderError::ConfigurationError(_) => {} // Expected
            _ => panic!("Expected configuration error"),
        }
    }

    #[test]
    fn test_openai_provider_with_organization() {
        let provider = OpenAIProvider::new("sk-test-key")
            .unwrap()
            .with_organization("org-test");
        assert_eq!(provider.organization, Some("org-test".to_string()));
    }

    #[test]
    fn test_openai_provider_with_base_url() {
        let provider = OpenAIProvider::new("sk-test-key")
            .unwrap()
            .with_base_url("https://custom-endpoint.com/v1");
        assert_eq!(provider.base_url, "https://custom-endpoint.com/v1");
    }

    #[tokio::test]
    async fn test_openai_supports_model() {
        let provider = OpenAIProvider::new("sk-test-key").unwrap();

        assert!(provider.supports_model(&ModelId::new("gpt-4")).await);
        assert!(
            provider
                .supports_model(&ModelId::new("gpt-3.5-turbo"))
                .await
        );
        assert!(
            !provider
                .supports_model(&ModelId::new("claude-3-opus"))
                .await
        );
    }

    #[tokio::test]
    async fn test_openai_model_capabilities() {
        let provider = OpenAIProvider::new("sk-test-key").unwrap();

        let gpt4_caps = provider.model_capabilities(&ModelId::new("gpt-4")).await;
        assert!(gpt4_caps.is_some());

        let caps = gpt4_caps.unwrap();
        assert_eq!(caps.max_tokens, 128000);
        assert!(caps.supports_tools);
        assert_eq!(caps.quality_tier, QualityTier::Ultra);

        let gpt35_caps = provider
            .model_capabilities(&ModelId::new("gpt-3.5-turbo"))
            .await;
        assert!(gpt35_caps.is_some());

        let caps = gpt35_caps.unwrap();
        assert_eq!(caps.max_tokens, 4096);
        assert_eq!(caps.speed_tier, SpeedTier::Fast);
    }

    #[test]
    fn test_create_headers() {
        let provider = OpenAIProvider::new("sk-test-key").unwrap();
        let headers = provider.create_headers();
        assert!(headers.is_ok());

        let headers = headers.unwrap();
        assert!(headers.contains_key("authorization"));
        assert!(headers.contains_key("content-type"));
    }

    #[test]
    fn test_model_capabilities_by_name() {
        let provider = OpenAIProvider::new("sk-test-key").unwrap();

        let gpt4_caps = provider.get_model_capabilities_by_name("gpt-4-turbo");
        assert!(gpt4_caps.supports_vision);
        assert_eq!(gpt4_caps.quality_tier, QualityTier::Ultra);

        let embedding_caps = provider.get_model_capabilities_by_name("text-embedding-ada-002");
        assert!(!embedding_caps.supports_tools);
        assert!(!embedding_caps.supports_streaming);
    }
}
