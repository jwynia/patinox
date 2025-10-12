//! Proper mocked tests for LMStudio provider
//!
//! These tests replace the tautological assertions with meaningful mocked behavior testing.
//! They test the provider's business logic for handling different HTTP responses.

use patinox::provider::local::LMStudioProvider;
use patinox::provider::types::{CompletionRequest, ModelId};
use patinox::provider::{ModelProvider, ProviderError};
use serde_json::json;

/// Test module with proper mocking for LMStudio provider
mod lmstudio_provider_mock_tests {
    use super::*;

    // Helper to create test completion request
    fn create_test_completion_request() -> CompletionRequest {
        CompletionRequest {
            model: ModelId::new("gpt-3.5-turbo"),
            messages: vec!["Hello".to_string()],
            max_tokens: Some(100),
            temperature: Some(0.7),
            tools: None,
        }
    }

    #[tokio::test]
    async fn test_list_models_success_with_mock_response() {
        // Arrange - Setup mock server with successful models response
        let mut mock_server = mockito::Server::new_async().await;
        let models_mock = mock_server
            .mock("GET", "/v1/models")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": "llama-2-7b",
                            "object": "model",
                            "created": 1677610602
                        },
                        {
                            "id": "code-llama-13b",
                            "object": "model",
                            "created": 1677610602
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let provider = LMStudioProvider::with_endpoint(mock_server.url())
            .expect("Should create provider with mock endpoint");

        // Act
        let result = provider.list_models().await;

        // Assert - Test that provider correctly processes successful response
        assert!(result.is_ok(), "Should successfully parse models response");
        let models = result.unwrap();

        assert_eq!(models.len(), 2, "Should parse both models from response");
        assert_eq!(models[0].name, "llama-2-7b");
        assert_eq!(models[1].name, "code-llama-13b");

        // Verify provider adds correct metadata
        assert_eq!(models[0].id.provider_hint(), Some("lmstudio"));
        assert!(models[0].capabilities.max_tokens > 0);

        models_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_models_handles_http_404_error() {
        // Arrange - Setup mock server to return 404
        let mut mock_server = mockito::Server::new_async().await;
        let error_mock = mock_server
            .mock("GET", "/v1/models")
            .with_status(404)
            .with_body("Not Found")
            .create_async()
            .await;

        let provider =
            LMStudioProvider::with_endpoint(mock_server.url()).expect("Should create provider");

        // Act
        let result = provider.list_models().await;

        // Assert - Test that provider correctly handles HTTP errors
        assert!(result.is_err(), "Should return error for 404 response");
        match result.unwrap_err() {
            ProviderError::NetworkError(msg) => {
                assert!(msg.contains("LMStudio API returned status: 404"));
            }
            other => panic!("Expected NetworkError, got {:?}", other),
        }

        error_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_models_handles_invalid_json_response() {
        // Arrange - Setup mock server with malformed JSON
        let mut mock_server = mockito::Server::new_async().await;
        let invalid_json_mock = mock_server
            .mock("GET", "/v1/models")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{ invalid json }")
            .create_async()
            .await;

        let provider =
            LMStudioProvider::with_endpoint(mock_server.url()).expect("Should create provider");

        // Act
        let result = provider.list_models().await;

        // Assert - Test that provider handles JSON parsing errors
        assert!(result.is_err(), "Should return error for invalid JSON");
        match result.unwrap_err() {
            ProviderError::ApiError(msg) => {
                assert!(msg.contains("Failed to parse LMStudio response"));
            }
            other => panic!("Expected ApiError, got {:?}", other),
        }

        invalid_json_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_success_with_mock_response() {
        // Arrange - Setup mock server with successful completion response
        let mut mock_server = mockito::Server::new_async().await;
        let completion_mock = mock_server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "id": "chatcmpl-123",
                    "object": "chat.completion",
                    "created": 1677610602,
                    "model": "gpt-3.5-turbo",
                    "choices": [
                        {
                            "index": 0,
                            "message": {
                                "role": "assistant",
                                "content": "Hello! How can I help you today?"
                            },
                            "finish_reason": "stop"
                        }
                    ],
                    "usage": {
                        "prompt_tokens": 10,
                        "completion_tokens": 20,
                        "total_tokens": 30
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let provider =
            LMStudioProvider::with_endpoint(mock_server.url()).expect("Should create provider");

        // Act
        let result = provider.complete(create_test_completion_request()).await;

        // Assert - Test that provider correctly processes completion response
        assert!(result.is_ok(), "Should successfully process completion");
        let response = result.unwrap();

        assert_eq!(response.content, "Hello! How can I help you today?");
        assert_eq!(response.model.name(), "gpt-3.5-turbo");
        assert_eq!(response.finish_reason, "stop");

        // Test usage information is correctly converted
        assert!(response.usage.is_some());
        if let Some(usage) = response.usage {
            assert_eq!(usage.prompt_tokens, 10);
            assert_eq!(usage.completion_tokens, 20);
            assert_eq!(usage.total_tokens, 30);
        }

        completion_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_complete_handles_empty_model_name_validation() {
        // Arrange - No mock needed, testing input validation
        let provider = LMStudioProvider::with_endpoint("http://unused".to_string())
            .expect("Should create provider");

        let invalid_request = CompletionRequest {
            model: ModelId::new(""), // Empty model name
            messages: vec!["Hello".to_string()],
            max_tokens: Some(100),
            temperature: Some(0.7),
            tools: None,
        };

        // Act
        let result = provider.complete(invalid_request).await;

        // Assert - Test input validation logic
        assert!(result.is_err(), "Should reject empty model name");
        match result.unwrap_err() {
            ProviderError::InvalidRequest(msg) => {
                assert!(msg.contains("Model name cannot be empty"));
            }
            other => panic!("Expected InvalidRequest, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_complete_handles_empty_messages_validation() {
        // Arrange - No mock needed, testing input validation
        let provider = LMStudioProvider::with_endpoint("http://unused".to_string())
            .expect("Should create provider");

        let invalid_request = CompletionRequest {
            model: ModelId::new("gpt-3.5-turbo"),
            messages: vec![], // Empty messages
            max_tokens: Some(100),
            temperature: Some(0.7),
            tools: None,
        };

        // Act
        let result = provider.complete(invalid_request).await;

        // Assert - Test input validation logic
        assert!(result.is_err(), "Should reject empty messages");
        match result.unwrap_err() {
            ProviderError::InvalidRequest(msg) => {
                assert!(msg.contains("Messages cannot be empty"));
            }
            other => panic!("Expected InvalidRequest, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_complete_handles_http_500_error() {
        // Arrange - Setup mock server to return server error
        let mut mock_server = mockito::Server::new_async().await;
        let error_mock = mock_server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let provider =
            LMStudioProvider::with_endpoint(mock_server.url()).expect("Should create provider");

        // Act
        let result = provider.complete(create_test_completion_request()).await;

        // Assert - Test that provider correctly handles server errors
        assert!(result.is_err(), "Should return error for 500 response");
        match result.unwrap_err() {
            ProviderError::NetworkError(msg) => {
                assert!(msg.contains("LMStudio API returned status: 500"));
            }
            other => panic!("Expected NetworkError, got {:?}", other),
        }

        error_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_supports_model_returns_false_when_model_not_in_list() {
        // Arrange - Setup mock with models that don't include the test model
        let mut mock_server = mockito::Server::new_async().await;
        let models_mock = mock_server
            .mock("GET", "/v1/models")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": "llama-2-7b",
                            "object": "model",
                            "created": 1677610602
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let provider =
            LMStudioProvider::with_endpoint(mock_server.url()).expect("Should create provider");

        // Act
        let supports = provider.supports_model(&ModelId::new("gpt-4")).await;

        // Assert - Test that provider correctly checks model availability
        assert!(
            !supports,
            "Should return false when model not in available models"
        );

        models_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_supports_model_returns_true_when_model_in_list() {
        // Arrange - Setup mock with models that include the test model
        let mut mock_server = mockito::Server::new_async().await;
        let models_mock = mock_server
            .mock("GET", "/v1/models")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": "llama-2-7b",
                            "object": "model",
                            "created": 1677610602
                        },
                        {
                            "id": "gpt-3.5-turbo",
                            "object": "model",
                            "created": 1677610602
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let provider =
            LMStudioProvider::with_endpoint(mock_server.url()).expect("Should create provider");

        // Act
        let supports = provider
            .supports_model(&ModelId::new("gpt-3.5-turbo"))
            .await;

        // Assert - Test that provider correctly finds available model
        assert!(
            supports,
            "Should return true when model is in available models"
        );

        models_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_model_capabilities_returns_none_when_list_models_fails() {
        // Arrange - Setup mock server to fail on models request
        let mut mock_server = mockito::Server::new_async().await;
        let error_mock = mock_server
            .mock("GET", "/v1/models")
            .with_status(503)
            .with_body("Service Unavailable")
            .create_async()
            .await;

        let provider =
            LMStudioProvider::with_endpoint(mock_server.url()).expect("Should create provider");

        // Act
        let capabilities = provider
            .model_capabilities(&ModelId::new("gpt-3.5-turbo"))
            .await;

        // Assert - Test that provider handles API failures gracefully
        assert!(
            capabilities.is_none(),
            "Should return None when unable to fetch model list"
        );

        error_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_model_capabilities_returns_some_when_model_found() {
        // Arrange - Setup mock with models including the test model
        let mut mock_server = mockito::Server::new_async().await;
        let models_mock = mock_server
            .mock("GET", "/v1/models")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": "gpt-3.5-turbo",
                            "object": "model",
                            "created": 1677610602
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let provider =
            LMStudioProvider::with_endpoint(mock_server.url()).expect("Should create provider");

        // Act
        let capabilities = provider
            .model_capabilities(&ModelId::new("gpt-3.5-turbo"))
            .await;

        // Assert - Test that provider returns capabilities for available model
        assert!(
            capabilities.is_some(),
            "Should return capabilities when model is available"
        );

        let caps = capabilities.unwrap();
        assert!(caps.max_tokens > 0, "Should have valid max_tokens");
        assert!(
            !caps.supports_tools,
            "LMStudio typically doesn't support tools"
        );
        assert!(
            !caps.supports_vision,
            "LMStudio typically doesn't support vision"
        );
        assert!(caps.supports_streaming, "LMStudio supports streaming");

        models_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_embed_returns_not_supported_error() {
        // Arrange - No mock needed, testing unsupported operation
        let provider = LMStudioProvider::with_endpoint("http://unused".to_string())
            .expect("Should create provider");

        let embed_request = patinox::provider::types::EmbeddingRequest {
            model: ModelId::new("text-embedding-ada-002"),
            input: vec!["test text".to_string()],
        };

        // Act
        let result = provider.embed(embed_request).await;

        // Assert - Test that provider correctly rejects unsupported operations
        assert!(result.is_err(), "Should reject embedding requests");
        match result.unwrap_err() {
            ProviderError::InvalidRequest(msg) => {
                assert!(msg.contains("LMStudio provider does not support embeddings"));
            }
            other => panic!("Expected InvalidRequest, got {:?}", other),
        }
    }
}
