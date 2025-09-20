//! Local Model Provider Module
//!
//! This module provides support for local AI model services including:
//! - Ollama (http://localhost:11434)
//! - LMStudio (http://localhost:1234)  
//!
//! ## Design Principles
//!
//! - **Service Discovery**: Automatically discover available local services
//! - **Fallback Support**: Route requests to available services with fallback
//! - **Health Monitoring**: Continuous health checking of local services
//! - **Performance**: Optimized connection pooling and caching
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use patinox::provider::local::LocalProvider;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = LocalProvider::new().await?;
//!     // Provider will auto-discover available local services
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod discovery;
pub mod error;
pub mod lmstudio;
pub mod ollama;
pub mod types;
pub mod validation;

// Re-export main types for convenient access
pub use config::{DiscoveryConfig, HealthCheckConfig, LocalProviderConfig};
pub use discovery::{ServiceDiscovery, ServiceInfo, ServiceStatus, ServiceType};
pub use error::{LocalProviderError, LocalProviderResult};
pub use lmstudio::LMStudioProvider;
pub use ollama::OllamaProvider;
pub use types::{LocalService, ServiceMetrics};

use crate::provider::types::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelCapabilities,
    ModelId, ModelInfo, StreamingResponse,
};
use crate::provider::{ModelProvider, ProviderError, ProviderResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main coordinator for local model providers
///
/// LocalProvider discovers and coordinates access to local AI services
/// like Ollama and LMStudio, providing unified access through the
/// standard ModelProvider interface.
#[allow(dead_code)]
pub struct LocalProvider {
    /// Service discovery and health monitoring
    discovery: Arc<ServiceDiscovery>,

    /// Ollama provider client (if available)
    ollama: Option<Arc<OllamaProvider>>,

    /// LMStudio provider client (if available)
    lmstudio: Option<Arc<LMStudioProvider>>,

    /// Provider configuration
    config: LocalProviderConfig,

    /// Shared HTTP client for connection pooling
    http_client: reqwest::Client,

    /// Cached model information across all providers
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
}

impl LocalProvider {
    /// Create new local provider with auto-discovery
    pub async fn new() -> ProviderResult<Self> {
        let config = LocalProviderConfig::default();
        Self::with_config(config).await
    }

    /// Create with custom configuration
    pub async fn with_config(config: LocalProviderConfig) -> ProviderResult<Self> {
        // Validate configuration first
        config
            .validate()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;

        // Create HTTP client with connection pooling
        let http_client = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .pool_max_idle_per_host(config.connection_pool_size)
            .build()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;

        let discovery = ServiceDiscovery::new(config.discovery.clone())
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;

        let provider = Self {
            discovery: Arc::new(discovery),
            ollama: None,
            lmstudio: None,
            config,
            http_client,
            model_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        Ok(provider)
    }

    /// Get list of available local services
    pub async fn available_services(&self) -> Vec<ServiceType> {
        self.discovery.available_services().await
    }

    /// Manually refresh available services
    pub async fn refresh_services(&mut self) -> ProviderResult<()> {
        self.discovery
            .discover_services()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl ModelProvider for LocalProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        // For now, return empty list until services are implemented
        Ok(Vec::new())
    }

    async fn complete(&self, _request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        // For now, return error until services are implemented
        Err(ProviderError::NetworkError(
            "No local services available".to_string(),
        ))
    }

    async fn stream_completion(
        &self,
        _request: CompletionRequest,
    ) -> ProviderResult<StreamingResponse> {
        // For now, return error until services are implemented
        Err(ProviderError::NetworkError(
            "No local services available".to_string(),
        ))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        // For now, return error until services are implemented
        Err(ProviderError::NetworkError(
            "No local services available".to_string(),
        ))
    }

    async fn supports_model(&self, _model: &ModelId) -> bool {
        // For now, return false until services are implemented
        false
    }

    async fn model_capabilities(&self, _model: &ModelId) -> Option<ModelCapabilities> {
        // For now, return None until services are implemented
        None
    }

    fn name(&self) -> &str {
        "local"
    }
}
