//! Ollama provider implementation
//!
//! This module provides integration with Ollama, a local LLM runner that allows you to run
//! large language models locally. Ollama supports various models including Llama, Mistral,
//! Code Llama, and many others.
//!
//! ## Features
//!
//! - **Local Model Support**: Run models locally without cloud dependencies
//! - **Model Management**: List and query available models
//! - **Text Generation**: Complete text generation with temperature control
//! - **Automatic Discovery**: Integrate with service discovery for automatic endpoint detection
//!
//! ## Usage
//!
//! ```rust,no_run
//! use patinox::provider::local::OllamaProvider;
//! use patinox::provider::{ModelProvider, types::{CompletionRequest, ModelId}};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create provider with default endpoint (http://localhost:11434)
//! let provider = OllamaProvider::new()?;
//!
//! // Or specify custom endpoint
//! let provider = OllamaProvider::with_endpoint("http://localhost:8080".to_string())?;
//!
//! // List available models
//! let models = provider.list_models().await?;
//!
//! // Make completion request
//! let request = CompletionRequest {
//!     model: ModelId::new("llama3.2"),
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
//!
//! ## API Compatibility
//!
//! This implementation uses Ollama's REST API:
//! - `/api/tags` - List available models
//! - `/api/generate` - Generate completions
//!
//! ## Error Handling
//!
//! The provider returns appropriate `ProviderError` variants:
//! - `NetworkError` - Connection issues or service unavailable
//! - `ApiError` - Ollama API errors or malformed responses
//! - `InvalidRequest` - Request validation failures

use crate::provider::types::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelCapabilities,
    ModelId, ModelInfo, StreamingChunk, StreamingResponse, Usage,
};
use crate::provider::{ModelProvider, ProviderError, ProviderResult};
use async_trait::async_trait;
use futures_util::{stream, TryStreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default endpoint for Ollama service
const DEFAULT_ENDPOINT: &str = "http://localhost:11434";

/// Default timeout for HTTP requests to Ollama service
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default context window size for most Ollama models
const DEFAULT_CONTEXT_WINDOW: usize = 4096;

/// Maximum allowed size for a single streaming chunk (in characters)
/// This prevents memory exhaustion from extremely large responses
const MAX_CHUNK_SIZE: usize = 1024 * 1024; // 1MB in characters

/// Default finish reason for completed streaming responses
const DEFAULT_FINISH_REASON: &str = "stop";

/// Ollama-specific provider implementation
pub struct OllamaProvider {
    /// HTTP client for API requests
    client: reqwest::Client,

    /// Base URL for Ollama API
    base_url: String,

    /// Cached model information
    #[allow(dead_code)] // Future caching implementation planned
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
}

impl OllamaProvider {
    /// Create new Ollama provider with default endpoint
    pub fn new() -> ProviderResult<Self> {
        Self::with_endpoint(DEFAULT_ENDPOINT.to_string())
    }

    /// Create usage information from Ollama response
    fn create_usage_from_response(response: &OllamaGenerateResponse) -> Option<Usage> {
        if response.prompt_eval_count.is_some() || response.eval_count.is_some() {
            Some(Usage {
                prompt_tokens: response.prompt_eval_count.unwrap_or(0) as usize,
                completion_tokens: response.eval_count.unwrap_or(0) as usize,
                total_tokens: (response.prompt_eval_count.unwrap_or(0)
                    + response.eval_count.unwrap_or(0)) as usize,
            })
        } else {
            None
        }
    }

    /// Create with custom endpoint
    pub fn with_endpoint(endpoint: String) -> ProviderResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;

        Ok(Self {
            client,
            base_url: endpoint,
            model_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

impl OllamaProvider {
    /// Internal helper to make HTTP requests to Ollama API
    async fn make_request<T>(&self, path: &str) -> ProviderResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);

        let response = self.client.get(&url).send().await.map_err(|e| {
            ProviderError::NetworkError(format!("Failed to connect to Ollama at {}: {}", url, e))
        })?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "Ollama API returned status: {}",
                response.status()
            )));
        }

        let body: T = response.json().await.map_err(|e| {
            ProviderError::ApiError(format!("Failed to parse Ollama response: {}", e))
        })?;

        Ok(body)
    }

    /// Internal helper for POST requests
    async fn make_post_request<T, B>(&self, path: &str, body: &B) -> ProviderResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| {
                ProviderError::NetworkError(format!(
                    "Failed to connect to Ollama at {}: {}",
                    url, e
                ))
            })?;

        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(format!(
                "Ollama API returned status: {}",
                response.status()
            )));
        }

        let response_body: T = response.json().await.map_err(|e| {
            ProviderError::ApiError(format!("Failed to parse Ollama response: {}", e))
        })?;

        Ok(response_body)
    }
}

/// Ollama API response for /api/tags endpoint
#[derive(Debug, serde::Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

/// Ollama model information from /api/tags
#[derive(Debug, serde::Deserialize)]
struct OllamaModelInfo {
    name: String,
    #[serde(default)]
    #[allow(dead_code)] // Part of API response but not currently used
    size: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)] // Part of API response but not currently used
    digest: Option<String>,
    #[serde(default)]
    #[allow(dead_code)] // Part of API response but not currently used
    details: Option<OllamaModelDetails>,
}

/// Ollama model details
#[derive(Debug, serde::Deserialize)]
struct OllamaModelDetails {
    #[serde(default)]
    #[allow(dead_code)] // Part of API response but not currently used
    parameter_size: Option<String>,
    #[serde(default)]
    #[allow(dead_code)] // Part of API response but not currently used
    quantization_level: Option<String>,
}

/// Ollama generate request
#[derive(Debug, serde::Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaGenerateOptions>,
    stream: bool,
}

/// Ollama generate options
#[derive(Debug, serde::Serialize)]
struct OllamaGenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>, // max_tokens in Ollama
}

/// Ollama generate response
#[derive(Debug, serde::Deserialize)]
struct OllamaGenerateResponse {
    #[serde(default)]
    #[allow(dead_code)] // Part of API response but not currently used
    model: String,
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    #[allow(dead_code)] // Part of API response but not currently used
    total_duration: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)] // Part of API response but not currently used
    load_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        let response: OllamaTagsResponse = self.make_request("/api/tags").await?;

        let models = response
            .models
            .into_iter()
            .map(|ollama_model| {
                // Create default capabilities for Ollama models
                let capabilities = ModelCapabilities {
                    max_tokens: DEFAULT_CONTEXT_WINDOW,
                    supports_tools: false, // Most Ollama models don't support tools
                    supports_vision: false, // Vision support varies by model
                    supports_streaming: true, // Ollama supports streaming
                    input_cost_per_1k: None, // Local models are free
                    output_cost_per_1k: None, // Local models are free
                    speed_tier: crate::provider::types::SpeedTier::Standard,
                    quality_tier: crate::provider::types::QualityTier::Standard,
                };

                ModelInfo {
                    id: ModelId::new(ollama_model.name.clone()).with_provider("ollama"),
                    name: ollama_model.name,
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

        // Convert messages to a single prompt (simplified for Ollama's generate endpoint)
        let prompt = if request.messages.is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Messages cannot be empty".to_string(),
            ));
        } else {
            request.messages.join("\n")
        };

        // Build Ollama request
        let options = OllamaGenerateOptions {
            temperature: request.temperature,
            num_predict: request.max_tokens.map(|t| t as i32),
        };

        let ollama_request = OllamaGenerateRequest {
            model: request.model.name().to_string(),
            prompt,
            options: Some(options),
            stream: false, // For now, only support non-streaming
        };

        let response: OllamaGenerateResponse = self
            .make_post_request("/api/generate", &ollama_request)
            .await?;

        // Convert response
        let usage = Self::create_usage_from_response(&response);

        Ok(CompletionResponse {
            model: request.model,
            content: response.response,
            usage,
            finish_reason: if response.done {
                "stop".to_string()
            } else {
                "incomplete".to_string()
            },
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

        // Convert messages to a single prompt (simplified for Ollama's generate endpoint)
        let prompt = request.messages.join("\n");

        // Build Ollama streaming request
        let options = OllamaGenerateOptions {
            temperature: request.temperature,
            num_predict: request.max_tokens.map(|t| t as i32),
        };

        let _ollama_request = OllamaGenerateRequest {
            model: request.model.name().to_string(),
            prompt,
            options: Some(options),
            stream: true, // Enable streaming
        };

        // Make HTTP POST request with streaming enabled
        let url = format!("{}/api/generate", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&_ollama_request)
            .send()
            .await
            .map_err(|e| {
                ProviderError::NetworkError(format!(
                    "Failed to connect to Ollama streaming at {}: {}",
                    url, e
                ))
            })?;

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

        // Create true streaming implementation that yields chunks as parsed
        // Uses async generator pattern to avoid collecting chunks in memory
        let byte_stream = response.bytes_stream();
        let buffer = String::new();

        let chunk_stream = stream::try_unfold(
            (byte_stream, buffer, false), // (stream, buffer, finished)
            move |(mut stream, mut buffer, finished)| {
                let model_id = model_id.clone();
                async move {
                    if finished {
                        return Ok(None);
                    }

                    loop {
                        // Process any complete lines in buffer first
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].to_string();
                            buffer.drain(..=newline_pos);

                            // Skip empty lines
                            if line.trim().is_empty() {
                                continue;
                            }

                            // Parse JSON line
                            let ollama_response: OllamaGenerateResponse = serde_json::from_str(&line)
                                .map_err(|e| ProviderError::ParseError(e.to_string()))?;

                            // Validate chunk size to prevent memory exhaustion
                            if ollama_response.response.len() > MAX_CHUNK_SIZE {
                                return Err(ProviderError::ApiError(format!(
                                    "Chunk size ({} chars) exceeds limit ({} chars)",
                                    ollama_response.response.len(),
                                    MAX_CHUNK_SIZE
                                )));
                            }

                            if ollama_response.done {
                                // Final chunk with usage information
                                let usage = Self::create_usage_from_response(&ollama_response);
                                let final_chunk = StreamingChunk::final_chunk(
                                    ollama_response.response,
                                    model_id.clone(),
                                    DEFAULT_FINISH_REASON.to_string(),
                                    usage,
                                );
                                return Ok(Some((final_chunk, (stream, buffer, true))));
                            } else {
                                // Regular chunk - yield immediately for true streaming
                                let chunk = StreamingChunk::new(ollama_response.response, false);
                                return Ok(Some((chunk, (stream, buffer, false))));
                            }
                        }

                        // No complete lines, read more bytes
                        match stream.try_next().await {
                            Ok(Some(chunk_bytes)) => {
                                // Convert bytes to string and add to buffer
                                let chunk_str = std::str::from_utf8(&chunk_bytes)
                                    .map_err(|e| ProviderError::ParseError(format!("Invalid UTF-8 in response: {}", e)))?;
                                buffer.push_str(chunk_str);
                                // Continue to process any complete lines
                            },
                            Ok(None) => {
                                // End of stream, process remaining buffer
                                if !buffer.trim().is_empty() {
                                    let ollama_response: OllamaGenerateResponse = serde_json::from_str(&buffer)
                                        .map_err(|e| ProviderError::ParseError(e.to_string()))?;

                                    // Validate chunk size
                                    if ollama_response.response.len() > MAX_CHUNK_SIZE {
                                        return Err(ProviderError::ApiError(format!(
                                            "Chunk size ({} chars) exceeds limit ({} chars)",
                                            ollama_response.response.len(),
                                            MAX_CHUNK_SIZE
                                        )));
                                    }

                                    if ollama_response.done {
                                        let usage = Self::create_usage_from_response(&ollama_response);
                                        let final_chunk = StreamingChunk::final_chunk(
                                            ollama_response.response,
                                            model_id.clone(),
                                            DEFAULT_FINISH_REASON.to_string(),
                                            usage,
                                        );
                                        return Ok(Some((final_chunk, (stream, buffer, true))));
                                    } else {
                                        let chunk = StreamingChunk::new(ollama_response.response, false);
                                        return Ok(Some((chunk, (stream, buffer, true))));
                                    }
                                }
                                return Ok(None);
                            },
                            Err(e) => {
                                return Err(ProviderError::NetworkError(format!("Failed to read Ollama streaming bytes: {}", e)));
                            }
                        }
                    }
                }
            }
        );

        Ok(StreamingResponse::new(chunk_stream))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        // Ollama supports embeddings but with different endpoint - will implement if needed
        Err(ProviderError::ApiError(
            "Embedding not yet implemented for Ollama provider".to_string(),
        ))
    }

    async fn supports_model(&self, model: &ModelId) -> bool {
        // Check if model is in our list of available models
        match self.list_models().await {
            Ok(models) => models.iter().any(|m| m.id.name() == model.name()),
            Err(_) => false, // If we can't list models, assume model is not supported
        }
    }

    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        // Get capabilities by finding the model in our list
        match self.list_models().await {
            Ok(models) => models
                .into_iter()
                .find(|m| m.id.name() == model.name())
                .map(|m| m.capabilities),
            Err(_) => None,
        }
    }

    fn name(&self) -> &str {
        "ollama"
    }
}
