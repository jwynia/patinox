//! LLM Provider Abstraction Layer
//!
//! This module provides a unified interface for interacting with different LLM providers
//! while supporting cascading configuration, fallback strategies, and provider-agnostic
//! model selection.
//!
//! ## Design Principles
//!
//! - **Provider Agnostic**: Works with OpenAI, Anthropic, OpenRouter, local models, etc.
//! - **Cascading Configuration**: Global → Agent → Request level overrides
//! - **Zero Required Config**: Sensible defaults work out of the box
//! - **Capability Awareness**: Models advertise their capabilities for smart routing
//! - **Resilient**: Automatic fallbacks and retry strategies
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use patinox::provider::{ModelId, CompletionRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Simple usage with defaults
//!     let provider = patinox::provider::create_default_provider().await?;
//!     
//!     let request = CompletionRequest {
//!         model: ModelId::new("claude-3-sonnet"),
//!         messages: vec!["Hello, world!".to_string()],
//!         temperature: Some(0.7),
//!         max_tokens: Some(100),
//!         tools: None,
//!     };
//!     
//!     let response = provider.complete(request).await?;
//!     println!("Response: {}", response.content);
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod openai;
pub mod secret;
pub mod types;

// Re-export main types for convenient access
pub use config::{
    AgentModelConfig, GlobalModelConfig, ModelConfigLoader, Provider, RequestConfig,
    SelectionStrategy,
};
pub use error::{ProviderError, ProviderResult};
pub use secret::SecretString;
pub use types::{
    CompletionMessage, CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse,
    ModelCapabilities, ModelId, ModelInfo, QualityTier, SpeedTier, ToolCall, Usage,
};

/// Core trait that all LLM providers must implement
#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    /// Get available models from this provider
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;

    /// Create a text completion
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError>;

    /// Create embeddings for text
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, ProviderError>;

    /// Check if provider supports a specific model
    async fn supports_model(&self, model: &ModelId) -> bool;

    /// Get model capabilities
    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities>;

    /// Provider name for debugging and telemetry
    fn name(&self) -> &str;
}

/// Create a default provider based on environment configuration
///
/// This function reads environment variables and configuration files to
/// create an appropriate provider instance with sensible defaults.
pub async fn create_default_provider() -> Result<Box<dyn ModelProvider>, ProviderError> {
    use crate::provider::openai::OpenAIProvider;
    
    // Try to load configuration from environment
    let config_loader = ModelConfigLoader::new();
    let config = config_loader.load().await?;
    
    match &config.default_provider {
        Provider::OpenAI { api_key, organization, base_url } => {
            let mut provider = OpenAIProvider::new(api_key.clone())?;
            
            if let Some(org) = organization {
                provider = provider.with_organization(org.clone());
            }
            
            if let Some(url) = base_url {
                provider = provider.with_base_url(url.clone());
            }
            
            Ok(Box::new(provider))
        }
        _ => {
            // For now, fallback to OpenAI if we have the key in environment
            if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
                let provider = OpenAIProvider::new(api_key)?;
                Ok(Box::new(provider))
            } else {
                Err(ProviderError::ConfigurationError(
                    "No valid provider configuration found. Set OPENAI_API_KEY environment variable.".to_string(),
                ))
            }
        }
    }
}

/// Create a multi-provider setup with fallbacks
///
/// This creates a provider that tries multiple backends in order,
/// falling back if the primary fails.
pub async fn create_fallback_provider(
    providers: Vec<Box<dyn ModelProvider>>,
) -> Result<Box<dyn ModelProvider>, ProviderError> {
    if providers.is_empty() {
        return Err(ProviderError::ConfigurationError(
            "Cannot create fallback provider with empty provider list".to_string(),
        ));
    }
    
    // For now, just return the first provider
    // TODO: Implement proper fallback provider wrapper
    let mut providers = providers;
    Ok(providers.remove(0))
}
