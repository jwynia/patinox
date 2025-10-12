//! Integration tests for LLM Provider Abstraction
//!
//! These tests verify that the provider abstraction layer works correctly
//! with different providers, configuration cascading, and error handling.

use patinox::provider::{
    CompletionRequest, CompletionResponse, ModelCapabilities, ModelId, ModelInfo, ModelProvider,
    ProviderError, QualityTier, SpeedTier,
};
use std::time::Duration;
use tokio::time::timeout;

/// Mock provider for testing
#[derive(Debug)]
struct MockProvider {
    name: &'static str,
    supported_models: Vec<ModelInfo>,
    should_fail: bool,
    response_delay: Option<Duration>,
}

impl MockProvider {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            supported_models: vec![ModelInfo {
                id: ModelId::new(format!("{}/test-model", name)),
                name: "Test Model".to_string(),
                capabilities: ModelCapabilities {
                    max_tokens: 4096,
                    supports_tools: true,
                    supports_vision: false,
                    supports_streaming: true,
                    input_cost_per_1k: Some(0.01),
                    output_cost_per_1k: Some(0.03),
                    speed_tier: SpeedTier::Fast,
                    quality_tier: QualityTier::Standard,
                },
            }],
            should_fail: false,
            response_delay: None,
        }
    }

    fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = Some(delay);
        self
    }
}

#[async_trait::async_trait]
impl ModelProvider for MockProvider {
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::ApiError("Mock failure".to_string()));
        }
        Ok(self.supported_models.clone())
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        if let Some(delay) = self.response_delay {
            tokio::time::sleep(delay).await;
        }

        if self.should_fail {
            return Err(ProviderError::ApiError(
                "Mock completion failure".to_string(),
            ));
        }

        Ok(CompletionResponse {
            model: request.model.clone(),
            content: format!("Mock response from {}", self.name),
            usage: None,
            finish_reason: "completed".to_string(),
        })
    }

    async fn stream_completion(
        &self,
        _request: patinox::provider::CompletionRequest,
    ) -> Result<patinox::provider::StreamingResponse, ProviderError> {
        // Mock streaming implementation
        Err(ProviderError::ApiError(
            "Streaming not implemented in mock provider".to_string(),
        ))
    }

    async fn embed(
        &self,
        _request: patinox::provider::EmbeddingRequest,
    ) -> Result<patinox::provider::EmbeddingResponse, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::ApiError(
                "Mock embedding failure".to_string(),
            ));
        }

        Ok(patinox::provider::EmbeddingResponse {
            embeddings: vec![vec![0.1, 0.2, 0.3]],
            model: ModelId::new("mock-embedding"),
            usage: None,
        })
    }

    async fn supports_model(&self, model: &ModelId) -> bool {
        self.supported_models.iter().any(|m| m.id == *model)
    }

    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        self.supported_models
            .iter()
            .find(|m| m.id == *model)
            .map(|m| m.capabilities.clone())
    }

    fn name(&self) -> &str {
        self.name
    }
}

#[tokio::test]
async fn test_model_id_creation() {
    let model = ModelId::new("claude-3-opus");
    assert_eq!(model.name(), "claude-3-opus");
    assert!(model.provider_hint().is_none());
}

#[tokio::test]
async fn test_model_id_with_provider() {
    let model = ModelId::new("claude-3-opus").with_provider("anthropic");
    assert_eq!(model.name(), "claude-3-opus");
    assert_eq!(model.provider_hint(), Some("anthropic"));
}

#[tokio::test]
async fn test_mock_provider_list_models() {
    let provider = MockProvider::new("openai");
    let models = provider.list_models().await.unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id.name(), "openai/test-model");
    assert_eq!(models[0].capabilities.max_tokens, 4096);
    assert!(models[0].capabilities.supports_tools);
}

#[tokio::test]
async fn test_mock_provider_complete() {
    let provider = MockProvider::new("openai");
    let request = CompletionRequest {
        model: ModelId::new("test-model"),
        messages: vec!["Hello".to_string()],
        temperature: Some(0.7),
        max_tokens: Some(100),
        tools: None,
    };

    let response = provider.complete(request).await.unwrap();
    assert_eq!(response.content, "Mock response from openai");
    assert_eq!(response.finish_reason, "completed");
}

#[tokio::test]
async fn test_provider_error_propagation() {
    let provider = MockProvider::new("failing").with_failure(true);
    let request = CompletionRequest {
        model: ModelId::new("test-model"),
        messages: vec!["Hello".to_string()],
        temperature: Some(0.7),
        max_tokens: Some(100),
        tools: None,
    };

    let result = provider.complete(request).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ProviderError::ApiError(msg) => assert_eq!(msg, "Mock completion failure"),
        _ => panic!("Expected ApiError"),
    }
}

#[tokio::test]
async fn test_provider_timeout_handling() {
    let provider = MockProvider::new("slow").with_delay(Duration::from_millis(100));
    let request = CompletionRequest {
        model: ModelId::new("test-model"),
        messages: vec!["Hello".to_string()],
        temperature: Some(0.7),
        max_tokens: Some(100),
        tools: None,
    };

    // Test that normal request succeeds
    let result = provider.complete(request.clone()).await;
    assert!(result.is_ok());

    // Test that timeout works
    let timeout_result = timeout(Duration::from_millis(50), provider.complete(request)).await;
    assert!(timeout_result.is_err()); // Should timeout
}

#[tokio::test]
async fn test_model_capabilities() {
    let provider = MockProvider::new("openai");
    let model = ModelId::new("openai/test-model");

    let capabilities = provider.model_capabilities(&model).await;
    assert!(capabilities.is_some());

    let caps = capabilities.unwrap();
    assert_eq!(caps.max_tokens, 4096);
    assert!(caps.supports_tools);
    assert!(!caps.supports_vision);
    assert_eq!(caps.speed_tier, SpeedTier::Fast);
    assert_eq!(caps.quality_tier, QualityTier::Standard);
}

#[tokio::test]
async fn test_model_support_check() {
    let provider = MockProvider::new("openai");

    let supported_model = ModelId::new("openai/test-model");
    assert!(provider.supports_model(&supported_model).await);

    let unsupported_model = ModelId::new("anthropic/claude-3-opus");
    assert!(!provider.supports_model(&unsupported_model).await);
}

#[tokio::test]
async fn test_embedding_functionality() {
    let provider = MockProvider::new("openai");
    let request = patinox::provider::EmbeddingRequest {
        model: ModelId::new("text-embedding-ada-002"),
        input: vec!["Hello world".to_string()],
    };

    let response = provider.embed(request).await.unwrap();
    assert_eq!(response.embeddings.len(), 1);
    assert_eq!(response.embeddings[0].len(), 3);
    assert_eq!(response.model.name(), "mock-embedding");
}

#[tokio::test]
async fn test_provider_error_types() {
    // Test API error
    let api_error = ProviderError::ApiError("API failed".to_string());
    assert!(matches!(api_error, ProviderError::ApiError(_)));

    // Test network error
    let network_error = ProviderError::NetworkError("Connection failed".to_string());
    assert!(matches!(network_error, ProviderError::NetworkError(_)));

    // Test authentication error
    let auth_error = ProviderError::AuthenticationError("Invalid API key".to_string());
    assert!(matches!(auth_error, ProviderError::AuthenticationError(_)));

    // Test rate limit error
    let rate_limit_error = ProviderError::RateLimited {
        retry_after: Some(Duration::from_secs(60)),
    };
    assert!(matches!(
        rate_limit_error,
        ProviderError::RateLimited { .. }
    ));
}

#[tokio::test]
async fn test_quality_and_speed_tiers() {
    // Test quality tier ordering
    assert!(QualityTier::Ultra > QualityTier::Premium);
    assert!(QualityTier::Premium > QualityTier::Standard);
    assert!(QualityTier::Standard > QualityTier::Lite);

    // Test speed tier ordering
    assert!(SpeedTier::Instant < SpeedTier::Fast);
    assert!(SpeedTier::Fast < SpeedTier::Standard);
    assert!(SpeedTier::Standard < SpeedTier::Slow);
}

#[tokio::test]
async fn test_model_info_creation() {
    let model_info = ModelInfo {
        id: ModelId::new("gpt-4-turbo"),
        name: "GPT-4 Turbo".to_string(),
        capabilities: ModelCapabilities {
            max_tokens: 128000,
            supports_tools: true,
            supports_vision: true,
            supports_streaming: true,
            input_cost_per_1k: Some(0.01),
            output_cost_per_1k: Some(0.03),
            speed_tier: SpeedTier::Standard,
            quality_tier: QualityTier::Premium,
        },
    };

    assert_eq!(model_info.id.name(), "gpt-4-turbo");
    assert_eq!(model_info.name, "GPT-4 Turbo");
    assert!(model_info.capabilities.supports_vision);
    assert_eq!(model_info.capabilities.max_tokens, 128000);
}

/// Test fixture for testing selection strategies
fn create_test_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: ModelId::new("fast-model"),
            name: "Fast Model".to_string(),
            capabilities: ModelCapabilities {
                max_tokens: 4096,
                supports_tools: false,
                supports_vision: false,
                supports_streaming: true,
                input_cost_per_1k: Some(0.001),
                output_cost_per_1k: Some(0.003),
                speed_tier: SpeedTier::Instant,
                quality_tier: QualityTier::Lite,
            },
        },
        ModelInfo {
            id: ModelId::new("balanced-model"),
            name: "Balanced Model".to_string(),
            capabilities: ModelCapabilities {
                max_tokens: 8192,
                supports_tools: true,
                supports_vision: false,
                supports_streaming: true,
                input_cost_per_1k: Some(0.01),
                output_cost_per_1k: Some(0.03),
                speed_tier: SpeedTier::Fast,
                quality_tier: QualityTier::Standard,
            },
        },
        ModelInfo {
            id: ModelId::new("premium-model"),
            name: "Premium Model".to_string(),
            capabilities: ModelCapabilities {
                max_tokens: 128000,
                supports_tools: true,
                supports_vision: true,
                supports_streaming: true,
                input_cost_per_1k: Some(0.1),
                output_cost_per_1k: Some(0.3),
                speed_tier: SpeedTier::Standard,
                quality_tier: QualityTier::Ultra,
            },
        },
    ]
}

#[tokio::test]
async fn test_model_capability_based_selection() {
    let models = create_test_models();

    // Test finding fastest model
    let fastest = models
        .iter()
        .min_by_key(|m| m.capabilities.speed_tier)
        .expect("Should have models");
    assert_eq!(fastest.capabilities.speed_tier, SpeedTier::Instant);

    // Test finding highest quality model
    let highest_quality = models
        .iter()
        .max_by_key(|m| m.capabilities.quality_tier)
        .expect("Should have models");
    assert_eq!(
        highest_quality.capabilities.quality_tier,
        QualityTier::Ultra
    );

    // Test filtering by capabilities
    let tools_capable: Vec<_> = models
        .iter()
        .filter(|m| m.capabilities.supports_tools)
        .collect();
    assert_eq!(tools_capable.len(), 2); // balanced and premium models

    // Test cost-based selection (cheapest input cost)
    let cheapest = models
        .iter()
        .filter_map(|m| m.capabilities.input_cost_per_1k.map(|cost| (m, cost)))
        .min_by(|(_, cost_a), (_, cost_b)| cost_a.partial_cmp(cost_b).unwrap())
        .map(|(model, _)| model)
        .expect("Should find cheapest model");
    assert_eq!(cheapest.id.name(), "fast-model");
}

#[tokio::test]
async fn test_provider_edge_cases() {
    let provider = MockProvider::new("edge-case-test");

    // Test empty message list
    let empty_request = CompletionRequest {
        model: ModelId::new("test-model"),
        messages: vec![], // Empty messages
        temperature: Some(0.7),
        max_tokens: Some(100),
        tools: None,
    };

    let result = provider.complete(empty_request).await;
    assert!(
        result.is_ok(),
        "Provider should handle empty messages gracefully"
    );

    // Test extreme temperature values
    let extreme_temp_request = CompletionRequest {
        model: ModelId::new("test-model"),
        messages: vec!["Test".to_string()],
        temperature: Some(2.0), // Max allowed
        max_tokens: Some(1),    // Minimum tokens
        tools: None,
    };

    let result = provider.complete(extreme_temp_request).await;
    assert!(
        result.is_ok(),
        "Provider should handle extreme parameter values"
    );

    // Test very long model names
    let long_model_name = "a".repeat(1000);
    let long_name_model = ModelId::new(&long_model_name);
    let supports_long_name = provider.supports_model(&long_name_model).await;
    assert!(
        !supports_long_name,
        "Provider should reject very long model names"
    );

    // Test model name with special characters
    let special_char_model = ModelId::new("model/with:special@chars");
    // This tests that the provider handles special characters without crashing
    // We don't care about the result, just that it doesn't panic
    let _supports_special = provider.supports_model(&special_char_model).await;
}

#[tokio::test]
async fn test_model_capabilities_edge_cases() {
    let provider = MockProvider::new("capability-test");

    // Test non-existent model capabilities
    let fake_model = ModelId::new("non-existent-model-12345");
    let capabilities = provider.model_capabilities(&fake_model).await;
    assert!(
        capabilities.is_none(),
        "Non-existent models should return None capabilities"
    );

    // Test model capabilities with empty name
    let empty_model = ModelId::new("");
    let empty_capabilities = provider.model_capabilities(&empty_model).await;
    assert!(
        empty_capabilities.is_none(),
        "Empty model names should return None capabilities"
    );

    // Test model list when provider fails
    let failing_provider = MockProvider::new("failing-list").with_failure(true);
    let models_result = failing_provider.list_models().await;
    assert!(
        models_result.is_err(),
        "Failed provider should return error for list_models"
    );

    match models_result.unwrap_err() {
        ProviderError::ApiError(msg) => assert_eq!(msg, "Mock failure"),
        _ => panic!("Expected ApiError for failing provider"),
    }
}
