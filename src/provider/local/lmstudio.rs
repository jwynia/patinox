//! LMStudio provider implementation

use crate::provider::types::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelCapabilities,
    ModelId, ModelInfo,
};
use crate::provider::{ModelProvider, ProviderError, ProviderResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
        Self::with_endpoint("http://localhost:1234".to_string())
    }

    /// Create with custom endpoint
    pub fn with_endpoint(endpoint: String) -> ProviderResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;

        Ok(Self {
            client,
            base_url: endpoint,
            model_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl ModelProvider for LMStudioProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        // For now, return empty list - will implement in next phase
        Ok(Vec::new())
    }

    async fn complete(&self, _request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        // For now, return error - will implement in next phase
        Err(ProviderError::NetworkError(
            "LMStudio service not available".to_string(),
        ))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        // For now, return error - will implement in next phase
        Err(ProviderError::NetworkError(
            "LMStudio service not available".to_string(),
        ))
    }

    async fn supports_model(&self, _model: &ModelId) -> bool {
        // For now, return false - will implement in next phase
        false
    }

    async fn model_capabilities(&self, _model: &ModelId) -> Option<ModelCapabilities> {
        // For now, return None - will implement in next phase
        None
    }

    fn name(&self) -> &str {
        "lmstudio"
    }
}
