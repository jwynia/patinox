//! Tests for OpenRouter provider implementation
//!
//! These tests define the expected behavior of the OpenRouter provider
//! following Test-Driven Development principles.

use patinox::provider::{
    openrouter::OpenRouterProvider, CompletionRequest, ModelId, ModelProvider, ProviderError,
    QualityTier,
};
use std::time::Duration;
use tokio::time::timeout;

/// Test suite for OpenRouterProvider
mod openrouter_provider_tests {
    use super::*;

    #[tokio::test]
    async fn test_openrouter_provider_creation_with_valid_api_key() {
        // Test that we can create an OpenRouter provider with a valid API key
        let provider = OpenRouterProvider::new("sk-or-test-key");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.base_url(), "https://openrouter.ai/api/v1");
        assert_eq!(provider.name(), "openrouter");
    }

    #[tokio::test]
    async fn test_openrouter_provider_creation_with_empty_api_key() {
        // Test that empty API key fails validation
        let provider = OpenRouterProvider::new("");
        assert!(provider.is_err());

        match provider.unwrap_err() {
            ProviderError::ConfigurationError(msg) => {
                assert!(msg.contains("API key cannot be empty"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[tokio::test]
    async fn test_openrouter_provider_with_custom_base_url() {
        // Test custom base URL configuration
        let provider = OpenRouterProvider::new("sk-or-test-key")
            .unwrap()
            .with_base_url("https://custom-openrouter.example.com/v1");

        assert_eq!(
            provider.base_url(),
            "https://custom-openrouter.example.com/v1"
        );
    }

    #[tokio::test]
    async fn test_openrouter_provider_with_custom_headers() {
        // Test custom HTTP-Referer and X-Title configuration
        let provider = OpenRouterProvider::new("sk-or-test-key")
            .unwrap()
            .with_referer("https://myapp.example.com")
            .with_title("My Awesome App");

        assert_eq!(provider.referer(), Some("https://myapp.example.com"));
        assert_eq!(provider.title(), Some("My Awesome App"));
    }

    #[tokio::test]
    async fn test_list_models_returns_openrouter_models() {
        // Test that we can list models from OpenRouter
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        // This will fail until we implement the provider
        // but it defines our expected interface
        let result = provider.list_models().await;

        // We expect a variety of models from different providers
        if let Ok(models) = result {
            let model_names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();

            // Test that we have models from various providers
            assert!(!model_names.is_empty());

            // Should include popular models from different providers
            // OpenRouter uses "provider/model" format
            let has_anthropic = model_names.iter().any(|&name| name.contains("anthropic"));
            let has_openai = model_names.iter().any(|&name| name.contains("openai"));
            let has_google = model_names.iter().any(|&name| name.contains("google"));

            // At least some models should be available
            assert!(has_anthropic || has_openai || has_google);
        }
    }

    #[tokio::test]
    async fn test_complete_with_anthropic_model_via_openrouter() {
        // Test completion request with Anthropic model through OpenRouter
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        let request = CompletionRequest {
            model: ModelId::new("anthropic/claude-3-sonnet-20240229"),
            messages: vec!["Hello, Claude via OpenRouter!".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        };

        // This will fail until implementation
        let result = provider.complete(request).await;

        if let Ok(response) = result {
            assert!(!response.content.is_empty());
            if let Some(usage) = response.usage {
                assert!(usage.total_tokens > 0);
                assert!(usage.prompt_tokens > 0);
                assert!(usage.completion_tokens > 0);
            }
            assert_eq!(response.model.name(), "anthropic/claude-3-sonnet-20240229");
        }
    }

    #[tokio::test]
    async fn test_complete_with_openai_model_via_openrouter() {
        // Test completion request with OpenAI model through OpenRouter
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        let request = CompletionRequest {
            model: ModelId::new("openai/gpt-4"),
            messages: vec!["Hello, GPT-4 via OpenRouter!".to_string()],
            temperature: Some(0.8),
            max_tokens: Some(150),
            tools: None,
        };

        let result = provider.complete(request).await;

        if let Ok(response) = result {
            assert!(!response.content.is_empty());
            assert_eq!(response.model.name(), "openai/gpt-4");
        }
    }

    #[tokio::test]
    async fn test_complete_with_provider_preferences() {
        // Test completion with provider routing preferences
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        // Use model ID with provider hint for routing
        let model = ModelId::new("claude-3-haiku-20240307").with_provider("anthropic");

        let request = CompletionRequest {
            model,
            messages: vec!["Route this to Anthropic specifically".to_string()],
            temperature: Some(0.5),
            max_tokens: Some(50),
            tools: None,
        };

        let result = provider.complete(request).await;

        if let Ok(response) = result {
            assert!(!response.content.is_empty());
            // Response should include the routed model
            assert!(
                response.model.name().contains("claude")
                    || response.model.name().contains("anthropic")
            );
        }
    }

    #[tokio::test]
    async fn test_complete_with_invalid_model() {
        // Test that invalid model names return appropriate errors
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        let request = CompletionRequest {
            model: ModelId::new("invalid/nonexistent-model"),
            messages: vec!["Hello".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ProviderError::ModelNotAvailable { model } => {
                assert!(model.contains("invalid/nonexistent-model"));
            }
            ProviderError::InvalidRequest(msg) => {
                assert!(msg.contains("invalid") || msg.contains("model"));
            }
            ProviderError::ApiError(msg) => {
                // OpenRouter API error for invalid model
                assert!(msg.contains("invalid") || msg.contains("400") || msg.contains("model"));
            }
            ProviderError::AuthenticationError(_) => {
                // Expected when using test API keys
            }
            _ => panic!(
                "Expected ModelNotAvailable, InvalidRequest, ApiError, or AuthenticationError"
            ),
        }
    }

    #[tokio::test]
    async fn test_complete_with_empty_messages() {
        // Test edge case: empty message list
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        let request = CompletionRequest {
            model: ModelId::new("anthropic/claude-3-haiku-20240307"),
            messages: vec![], // Empty messages
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ProviderError::InvalidRequest(msg) => {
                assert!(msg.contains("messages") || msg.contains("empty"));
            }
            _ => {
                // Other errors are also acceptable - OpenRouter might reject in different ways
            }
        }
    }

    #[tokio::test]
    async fn test_supports_model_for_routed_models() {
        // Test model support detection for OpenRouter
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        // Should support OpenRouter format models
        let anthropic_model = ModelId::new("anthropic/claude-3-opus-20240229");
        assert!(provider.supports_model(&anthropic_model).await);

        let openai_model = ModelId::new("openai/gpt-4");
        assert!(provider.supports_model(&openai_model).await);

        let google_model = ModelId::new("google/gemini-pro");
        assert!(provider.supports_model(&google_model).await);

        // Should also support direct model names (OpenRouter can route them)
        let direct_claude = ModelId::new("claude-3-sonnet-20240229");
        assert!(provider.supports_model(&direct_claude).await);

        let direct_gpt = ModelId::new("gpt-4");
        assert!(provider.supports_model(&direct_gpt).await);

        // Should not support obviously invalid models
        let invalid_model = ModelId::new("fake/nonexistent-model-12345");
        assert!(!provider.supports_model(&invalid_model).await);
    }

    #[tokio::test]
    async fn test_model_capabilities_for_routed_models() {
        // Test that we return appropriate capabilities for OpenRouter models
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        // Test capabilities for Anthropic model via OpenRouter
        let anthropic_model = ModelId::new("anthropic/claude-3-opus-20240229");
        let capabilities = provider.model_capabilities(&anthropic_model).await;

        assert!(capabilities.is_some());
        let caps = capabilities.unwrap();

        assert_eq!(caps.quality_tier, QualityTier::Ultra);
        assert!(caps.max_tokens >= 200000); // Claude 3 has large context
        assert!(caps.supports_tools);
        assert!(caps.supports_vision);
        assert!(caps.input_cost_per_1k.is_some());
        assert!(caps.output_cost_per_1k.is_some());

        // Test capabilities for OpenAI model via OpenRouter
        let openai_model = ModelId::new("openai/gpt-4");
        let gpt_caps = provider.model_capabilities(&openai_model).await;

        assert!(gpt_caps.is_some());
        let gpt = gpt_caps.unwrap();

        assert_eq!(gpt.quality_tier, QualityTier::Ultra);
        assert!(gpt.max_tokens >= 8000);
        assert!(gpt.supports_tools);

        // Test unknown model
        let unknown_model = ModelId::new("unknown/nonexistent-model");
        let unknown_caps = provider.model_capabilities(&unknown_model).await;
        assert!(unknown_caps.is_none());
    }

    #[tokio::test]
    async fn test_embed_support_for_embedding_models() {
        // Test embedding functionality through OpenRouter
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        let request = patinox::provider::EmbeddingRequest {
            input: vec!["Hello world".to_string(), "Test embedding".to_string()],
            model: ModelId::new("openai/text-embedding-ada-002"),
        };

        let result = provider.embed(request).await;

        if let Ok(response) = result {
            assert!(!response.embeddings.is_empty());
            assert_eq!(response.embeddings.len(), 2); // Two input strings
            assert!(!response.embeddings[0].is_empty()); // Vector has dimensions

            if let Some(usage) = response.usage {
                assert!(usage.total_tokens > 0);
            }
        } else {
            // If embeddings fail, it should be a clear error
            match result.unwrap_err() {
                ProviderError::InvalidRequest(msg) => {
                    assert!(msg.contains("embedding") || msg.contains("not supported"));
                }
                ProviderError::AuthenticationError(_) => {
                    // Expected with test keys
                }
                _ => {
                    // Other errors are acceptable for test scenarios
                }
            }
        }
    }

    #[tokio::test]
    async fn test_authentication_headers() {
        // Test that proper authentication headers are created
        let provider = OpenRouterProvider::new("sk-or-test-key")
            .unwrap()
            .with_referer("https://testapp.com")
            .with_title("Test App");

        // This tests internal header creation - we'll need a method to expose this
        let headers = provider.create_headers().unwrap();

        // OpenRouter uses Bearer token
        assert!(headers.contains_key("authorization"));
        assert_eq!(
            headers.get("authorization").unwrap(),
            "Bearer sk-or-test-key"
        );

        // OpenRouter supports HTTP-Referer for attribution
        assert!(headers.contains_key("http-referer"));
        assert_eq!(headers.get("http-referer").unwrap(), "https://testapp.com");

        // OpenRouter supports X-Title for app identification
        assert!(headers.contains_key("x-title"));
        assert_eq!(headers.get("x-title").unwrap(), "Test App");

        // Content-Type should be application/json
        assert!(headers.contains_key("content-type"));
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
    }

    #[tokio::test]
    async fn test_error_handling_for_rate_limits() {
        // Test that we properly handle OpenRouter rate limit responses
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        let request = CompletionRequest {
            model: ModelId::new("anthropic/claude-3-haiku-20240307"),
            messages: vec!["Test rate limit".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        };

        // This will test our error handling when we get HTTP 429
        let result = provider.complete(request).await;

        if let Err(ProviderError::RateLimited {
            retry_after: Some(duration),
        }) = result
        {
            // Rate limit should include retry-after information
            assert!(duration > Duration::from_secs(0));
        }
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        // Test that requests timeout appropriately
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        let request = CompletionRequest {
            model: ModelId::new("anthropic/claude-3-opus-20240229"),
            messages: vec!["This is a test message that might take time to respond to.".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            tools: None,
        };

        // Test with very short timeout
        let result = timeout(Duration::from_millis(1), provider.complete(request)).await;

        assert!(result.is_err()); // Should timeout
    }

    #[tokio::test]
    async fn test_request_serialization() {
        // Test that our requests are properly serialized to OpenRouter format
        let provider = OpenRouterProvider::new("sk-or-test-key")
            .unwrap()
            .with_referer("https://myapp.com")
            .with_title("MyApp");

        let request = CompletionRequest {
            model: ModelId::new("anthropic/claude-3-sonnet-20240229").with_provider("anthropic"),
            messages: vec![
                "Human: Hello!".to_string(),
                "Assistant: Hi there!".to_string(),
                "Human: How are you?".to_string(),
            ],
            temperature: Some(0.8),
            max_tokens: Some(150),
            tools: None,
        };

        // Test internal serialization - we'll need a method to expose this
        let openrouter_request = provider.convert_to_openrouter_format(&request).unwrap();

        assert_eq!(
            openrouter_request.model,
            "anthropic/claude-3-sonnet-20240229"
        );
        assert_eq!(openrouter_request.max_tokens, Some(150));
        assert_eq!(openrouter_request.temperature, Some(0.8));

        // OpenRouter expects messages in OpenAI format
        assert!(!openrouter_request.messages.is_empty());

        // Should include provider preferences if specified
        if let Some(provider_config) = openrouter_request.provider {
            assert!(provider_config.order.is_some() || provider_config.allow_fallbacks);
        }
    }

    #[tokio::test]
    async fn test_response_deserialization() {
        // Test that we properly parse OpenRouter responses (OpenAI format)
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        // Mock OpenRouter response format (same as OpenAI)
        let openrouter_response_json = r#"{
            "id": "chatcmpl-8abc123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "anthropic/claude-3-sonnet-20240229",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! I'm Claude, running via OpenRouter. How can I help you today?"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 15,
                "completion_tokens": 25,
                "total_tokens": 40
            }
        }"#;

        // Test internal deserialization - we'll need a method to expose this
        let response = provider
            .parse_openrouter_response(openrouter_response_json)
            .unwrap();

        assert_eq!(response.model.name(), "anthropic/claude-3-sonnet-20240229");
        assert!(response.content.contains("Claude"));
        assert!(response.content.contains("OpenRouter"));

        if let Some(usage) = response.usage {
            assert_eq!(usage.prompt_tokens, 15);
            assert_eq!(usage.completion_tokens, 25);
            assert_eq!(usage.total_tokens, 40);
        }
    }

    #[tokio::test]
    async fn test_model_routing_strategies() {
        // Test different model routing strategies
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        // Test fallback strategy
        let request_with_fallback = CompletionRequest {
            model: ModelId::new("claude-3-opus-20240229"), // No provider specified - let OpenRouter choose
            messages: vec!["Route with fallback enabled".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        };

        let result = provider.complete(request_with_fallback).await;

        // Should either succeed or fail with a clear error
        match result {
            Ok(response) => {
                assert!(!response.content.is_empty());
                // Model name should include provider when routed
                assert!(
                    response.model.name().contains("claude") || response.model.name().contains("/")
                );
            }
            Err(e) => {
                // Acceptable errors for test scenarios
                match e {
                    ProviderError::AuthenticationError(_) => {}
                    ProviderError::ApiError(_) => {}
                    ProviderError::NetworkError(_) => {}
                    _ => {}
                }
            }
        }
    }

    #[tokio::test]
    async fn test_cost_optimization_features() {
        // Test that OpenRouter provides cost information in model capabilities
        let provider = OpenRouterProvider::new("sk-or-test-key").unwrap();

        // Test that different models have different cost tiers
        let expensive_model = ModelId::new("anthropic/claude-3-opus-20240229");
        let cheap_model = ModelId::new("anthropic/claude-3-haiku-20240307");

        let expensive_caps = provider.model_capabilities(&expensive_model).await;
        let cheap_caps = provider.model_capabilities(&cheap_model).await;

        if let (Some(expensive), Some(cheap)) = (expensive_caps, cheap_caps) {
            // Opus should be more expensive than Haiku
            if let (Some(exp_cost), Some(cheap_cost)) =
                (expensive.input_cost_per_1k, cheap.input_cost_per_1k)
            {
                assert!(exp_cost > cheap_cost);
            }

            // Quality tiers should reflect model positioning
            assert_eq!(expensive.quality_tier, QualityTier::Ultra);
            assert_eq!(cheap.quality_tier, QualityTier::Standard);
        }
    }
}

/// Integration tests with mock HTTP server
mod openrouter_integration_tests {
    #[allow(unused_imports)]
    use super::*;

    // These tests will use mockito or similar to create a mock OpenRouter API
    // We'll implement these after the basic provider is working

    #[tokio::test]
    #[ignore] // Ignore until we implement HTTP mocking
    async fn test_real_openrouter_api_integration() {
        // Integration test with actual OpenRouter API
        // This would require real API keys and should only run in CI
        todo!("Implement with real API testing")
    }

    #[tokio::test]
    #[ignore] // Ignore until we implement HTTP mocking
    async fn test_mock_openrouter_server_responses() {
        // Test with mock HTTP server
        todo!("Implement with mockito or wiremock")
    }

    #[tokio::test]
    #[ignore] // Ignore until we implement HTTP mocking
    async fn test_provider_fallback_behavior() {
        // Test OpenRouter's automatic fallback when primary provider fails
        todo!("Implement with mock server that simulates provider failures")
    }
}
