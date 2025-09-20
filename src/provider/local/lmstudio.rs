//! LMStudio provider implementation
//!
//! This module provides integration with LMStudio, a local LLM server that provides
//! OpenAI-compatible API endpoints for running models locally.
//!
//! ## Features
//!
//! - **OpenAI-Compatible API**: Uses standard OpenAI API format for compatibility
//! - **Local Model Support**: Run models locally without cloud dependencies
//! - **Model Management**: List and query available models
//! - **Text Generation**: Complete text generation with standard parameters
//!
//! ## Usage
//!
//! ```rust,no_run
//! use patinox::provider::local::LMStudioProvider;
//! use patinox::provider::{ModelProvider, types::{CompletionRequest, ModelId}};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create provider with default endpoint (http://localhost:1234)
//! let provider = LMStudioProvider::new()?;
//!
//! // Or specify custom endpoint
//! let provider = LMStudioProvider::with_endpoint("http://localhost:5678".to_string())?;
//!
//! // List available models
//! let models = provider.list_models().await?;
//!
//! // Make completion request
//! let request = CompletionRequest {
//!     model: ModelId::new("gpt-3.5-turbo"),
//!     messages: vec!["Hello, how are you?".to_string()],
//!     temperature: Some(0.7),
//!     max_tokens: Some(100),
//!     tools: None,
//! };
//! let response = provider.complete(request).await?;
//! println!("Response: {}", response.content);
//! # Ok(())
//! # }
//! ```

use crate::provider::types::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelCapabilities,
    ModelId, ModelInfo, QualityTier, SpeedTier, StreamingChunk, StreamingResponse, Usage,
};
use crate::provider::{ModelProvider, ProviderError, ProviderResult};
use async_trait::async_trait;
use futures_util::stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default configuration constants for LMStudio provider
mod defaults {
    /// Default LMStudio endpoint
    pub const ENDPOINT: &str = "http://localhost:1234";
    /// Default HTTP timeout in seconds
    pub const TIMEOUT_SECS: u64 = 30;
    /// Default context window size for LMStudio models
    /// Based on common transformer model defaults. Individual models may have different limits
    /// that can be queried through the LMStudio API, but this provides a reasonable fallback.
    pub const CONTEXT_WINDOW: usize = 4096;
}

/// OpenAI-compatible models response structure for LMStudio
#[derive(Deserialize, Debug)]
struct LMStudioModelsResponse {
    data: Vec<LMStudioModel>,
}

/// OpenAI-compatible model structure for LMStudio  
#[derive(Deserialize, Debug)]
#[allow(dead_code)] // API response fields used for deserialization
struct LMStudioModel {
    id: String,
    #[serde(default)]
    object: String,
    #[serde(default)]
    created: u64,
}

/// OpenAI-compatible chat completion request for LMStudio
#[derive(Serialize, Debug)]
struct LMStudioCompletionRequest {
    model: String,
    messages: Vec<LMStudioMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(default = "default_stream")]
    stream: bool,
}

/// OpenAI-compatible message structure for LMStudio
#[derive(Serialize, Debug)]
struct LMStudioMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible completion response from LMStudio
#[derive(Deserialize, Debug)]
#[allow(dead_code)] // API response fields used for deserialization
struct LMStudioCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<LMStudioChoice>,
    usage: LMStudioUsage,
}

/// OpenAI-compatible choice structure from LMStudio
#[derive(Deserialize, Debug)]
struct LMStudioChoice {
    #[allow(dead_code)]
    index: u32,
    message: LMStudioResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

/// OpenAI-compatible response message structure from LMStudio
#[derive(Deserialize, Debug)]
struct LMStudioResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
}

/// OpenAI-compatible usage statistics from LMStudio
#[derive(Deserialize, Debug)]
struct LMStudioUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI-compatible streaming chunk response from LMStudio
#[derive(Deserialize, Debug)]
#[allow(dead_code)] // API response fields used for deserialization
struct LMStudioStreamingResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<LMStudioStreamingChoice>,
    usage: Option<LMStudioUsage>,
}

/// OpenAI-compatible streaming choice for LMStudio
#[derive(Deserialize, Debug)]
#[allow(dead_code)] // API response fields used for deserialization
struct LMStudioStreamingChoice {
    index: u32,
    delta: LMStudioDelta,
    finish_reason: Option<String>,
}

/// OpenAI-compatible delta content for streaming
#[derive(Deserialize, Debug)]
#[allow(dead_code)] // API response fields used for deserialization
struct LMStudioDelta {
    content: Option<String>,
}

#[allow(dead_code)]
fn default_stream() -> bool {
    false
}

/// LMStudio-specific provider implementation
#[allow(dead_code)]
pub struct LMStudioProvider {
    /// HTTP client for API requests
    client: reqwest::Client,

    /// Base URL for LMStudio API
    base_url: String,

    /// Cached model information
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
}

impl LMStudioProvider {
    /// Create new LMStudio provider with default endpoint
    pub fn new() -> ProviderResult<Self> {
        Self::with_endpoint(defaults::ENDPOINT.to_string())
    }

    /// Create with custom endpoint
    pub fn with_endpoint(endpoint: String) -> ProviderResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(defaults::TIMEOUT_SECS))
            .build()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;

        Ok(Self {
            client,
            base_url: endpoint,
            model_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Make HTTP GET request to LMStudio API
    async fn make_request<T>(&self, path: &str) -> ProviderResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);

        let response = self.client.get(&url).send().await.map_err(|e| {
            ProviderError::NetworkError(format!(
                "Failed to GET from LMStudio at {} (check if LMStudio is running): {}",
                url, e
            ))
        })?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "LMStudio API returned status: {}",
                response.status()
            )));
        }

        let body: T = response.json().await.map_err(|e| {
            ProviderError::ApiError(format!("Failed to parse LMStudio response: {}", e))
        })?;

        Ok(body)
    }

    /// Make HTTP POST request to LMStudio API
    async fn make_post_request<T, B>(&self, path: &str, body: &B) -> ProviderResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| {
                ProviderError::NetworkError(format!(
                    "Failed to POST to LMStudio at {} (check if LMStudio is running): {}",
                    url, e
                ))
            })?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "LMStudio API returned status: {}",
                response.status()
            )));
        }

        let response_body: T = response.json().await.map_err(|e| {
            ProviderError::ApiError(format!("Failed to parse LMStudio response: {}", e))
        })?;

        Ok(response_body)
    }
}

#[async_trait]
impl ModelProvider for LMStudioProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        let response: LMStudioModelsResponse = self.make_request("/v1/models").await?;

        let models = response
            .data
            .into_iter()
            .map(|lmstudio_model| {
                // Create default capabilities for LMStudio models
                let capabilities = ModelCapabilities {
                    max_tokens: defaults::CONTEXT_WINDOW,
                    supports_tools: false, // LMStudio typically doesn't support tool calling
                    supports_vision: false, // LMStudio typically doesn't support vision
                    supports_streaming: true, // LMStudio can support streaming
                    input_cost_per_1k: None, // Local model - no cost
                    output_cost_per_1k: None, // Local model - no cost
                    quality_tier: QualityTier::Standard,
                    speed_tier: SpeedTier::Standard,
                };

                let model_name = lmstudio_model.id;
                ModelInfo {
                    id: ModelId::new(model_name.clone()).with_provider("lmstudio"),
                    name: model_name,
                    capabilities,
                }
            })
            .collect();

        Ok(models)
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        // Validate request
        if request.model.name().is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Model name cannot be empty".to_string(),
            ));
        }

        // Convert messages to OpenAI format (simplified for now)
        let prompt = if request.messages.is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Messages cannot be empty".to_string(),
            ));
        } else {
            // For simplicity, join messages. Production version would handle conversation format properly
            request.messages.join("\n")
        };

        // Create LMStudio completion request (OpenAI-compatible)
        let lmstudio_request = LMStudioCompletionRequest {
            model: request.model.name().to_string(),
            messages: vec![LMStudioMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: request.max_tokens.map(|t| t as u32),
            temperature: request.temperature,
            stream: false, // For now, only support non-streaming
        };

        let response: LMStudioCompletionResponse = self
            .make_post_request("/v1/chat/completions", &lmstudio_request)
            .await?;

        // Convert response to our format with validation
        let content = response
            .choices
            .first()
            .ok_or_else(|| ProviderError::ApiError("No choices in LMStudio response".to_string()))?
            .message
            .content
            .clone();

        let usage = Usage {
            prompt_tokens: response.usage.prompt_tokens as usize,
            completion_tokens: response.usage.completion_tokens as usize,
            total_tokens: response.usage.total_tokens as usize,
        };

        Ok(CompletionResponse {
            model: request.model,
            content,
            usage: Some(usage),
            finish_reason: response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone())
                .unwrap_or_else(|| "stop".to_string()),
        })
    }

    async fn stream_completion(
        &self,
        request: CompletionRequest,
    ) -> ProviderResult<StreamingResponse> {
        // Validate request
        if request.model.name().is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Model name cannot be empty".to_string(),
            ));
        }

        if request.messages.is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Messages cannot be empty".to_string(),
            ));
        }

        // Convert messages to OpenAI format
        let prompt = request.messages.join("\n");

        // Create LMStudio streaming request (OpenAI-compatible)
        let _lmstudio_request = LMStudioCompletionRequest {
            model: request.model.name().to_string(),
            messages: vec![LMStudioMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: request.max_tokens.map(|t| t as u32),
            temperature: request.temperature,
            stream: true, // Enable streaming
        };

        // Make HTTP POST request with streaming enabled
        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&_lmstudio_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(ProviderError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let model_id = request.model.clone();

        // Convert the response to text and parse SSE format
        let response_text = response
            .text()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        // Parse Server-Sent Events format
        let chunks: Result<Vec<StreamingChunk>, ProviderError> = response_text
            .lines()
            .filter(|line| line.starts_with("data: ") && !line.trim().is_empty())
            .map(|line| {
                // Remove "data: " prefix
                let data = &line[6..];

                // Check for completion signal
                if data.trim() == "[DONE]" {
                    return Ok(None); // Skip DONE marker
                }

                // Parse JSON data
                let streaming_response: LMStudioStreamingResponse = serde_json::from_str(data)
                    .map_err(|e| ProviderError::ParseError(e.to_string()))?;

                // Extract content from first choice
                if let Some(choice) = streaming_response.choices.first() {
                    if let Some(finish_reason) = &choice.finish_reason {
                        // Final chunk with usage information
                        let usage =
                            streaming_response
                                .usage
                                .map(|u| crate::provider::types::Usage {
                                    prompt_tokens: u.prompt_tokens as usize,
                                    completion_tokens: u.completion_tokens as usize,
                                    total_tokens: u.total_tokens as usize,
                                });

                        let content = choice.delta.content.clone().unwrap_or_default();
                        Ok(Some(StreamingChunk::final_chunk(
                            content,
                            model_id.clone(),
                            finish_reason.clone(),
                            usage,
                        )))
                    } else {
                        // Regular chunk
                        let content = choice.delta.content.clone().unwrap_or_default();
                        Ok(Some(StreamingChunk::new(content, false)))
                    }
                } else {
                    // No choices, skip this chunk
                    Ok(None)
                }
            })
            .filter_map(|result| match result {
                Ok(Some(chunk)) => Some(Ok(chunk)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .collect();

        let stream = stream::iter(chunks?.into_iter().map(Ok));
        Ok(StreamingResponse::new(stream))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        // LMStudio doesn't typically support embeddings, return appropriate error
        Err(ProviderError::InvalidRequest(
            "LMStudio provider does not support embeddings".to_string(),
        ))
    }

    async fn supports_model(&self, model: &ModelId) -> bool {
        // Check if model exists in available models list
        match self.list_models().await {
            Ok(models) => models.iter().any(|m| m.id.name() == model.name()),
            Err(_) => false, // If we can't get models list, assume not supported
        }
    }

    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        // Get capabilities from models list
        match self.list_models().await {
            Ok(models) => models
                .into_iter()
                .find(|m| m.id.name() == model.name())
                .map(|m| m.capabilities),
            Err(_) => None,
        }
    }

    fn name(&self) -> &str {
        "lmstudio"
    }
}
