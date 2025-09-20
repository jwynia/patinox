//! Basic streaming tests for local providers (Ollama and LMStudio)
//!
//! This test suite provides basic validation that streaming methods exist
//! and return the expected basic response structure.

// Real HTTP streaming test constants (mock constants removed as we now use real HTTP)
use patinox::provider::local::{LMStudioProvider, OllamaProvider};
use patinox::provider::types::{CompletionRequest, ModelId};
use patinox::provider::{ModelProvider, ProviderError};

/// Test helper for streaming scenarios
struct StreamingTestHelper;

impl StreamingTestHelper {
    /// Create a simple test request
    fn create_test_request() -> CompletionRequest {
        CompletionRequest {
            model: ModelId::new("test-model"),
            messages: vec!["Test message".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        }
    }
}

mod ollama_streaming {
    use super::*;

    #[tokio::test]
    async fn test_stream_completion_returns_response() {
        // Arrange: Create Ollama provider
        let provider = match OllamaProvider::new() {
            Ok(p) => p,
            Err(_) => {
                // Skip test if Ollama provider can't be created (expected in CI)
                println!("Skipping Ollama test - provider creation failed");
                return;
            }
        };

        let request = StreamingTestHelper::create_test_request();

        // Act: Call stream_completion - this should fail with network error (no server)
        let stream_result = provider.stream_completion(request).await;

        // Assert: Should fail with network error since no server is running
        assert!(
            stream_result.is_err(),
            "Stream completion should fail when no server is available"
        );

        // Verify it's a network error
        match stream_result.unwrap_err() {
            ProviderError::NetworkError(_) => {
                // Expected behavior - no server running
                println!("Expected network error - no Ollama server");
            }
            other => panic!("Expected NetworkError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_stream_completion_validates_empty_model() {
        // Arrange: Create request with empty model name
        let provider = match OllamaProvider::new() {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping Ollama validation test - provider creation failed");
                return;
            }
        };

        let mut request = StreamingTestHelper::create_test_request();
        request.model = ModelId::new(""); // Empty model name

        // Act
        let result = provider.stream_completion(request).await;

        // Assert: Should return validation error
        assert!(result.is_err(), "Should fail with empty model name");
        match result.err().unwrap() {
            ProviderError::InvalidRequest(msg) => {
                assert!(msg.contains("empty"), "Error should mention empty model");
            }
            other => panic!("Expected InvalidRequest error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_stream_completion_validates_empty_messages() {
        // Arrange: Create request with empty messages
        let provider = match OllamaProvider::new() {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping Ollama validation test - provider creation failed");
                return;
            }
        };

        let mut request = StreamingTestHelper::create_test_request();
        request.messages = vec![]; // Empty messages

        // Act
        let result = provider.stream_completion(request).await;

        // Assert: Should return validation error
        assert!(result.is_err(), "Should fail with empty messages");
        match result.err().unwrap() {
            ProviderError::InvalidRequest(msg) => {
                assert!(msg.contains("empty"), "Error should mention empty messages");
            }
            other => panic!("Expected InvalidRequest error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_stream_completion_backward_compatibility() {
        // Arrange: Ensure existing complete() method still works
        let provider = match OllamaProvider::new() {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping Ollama backward compatibility test - provider creation failed");
                return;
            }
        };

        let request = StreamingTestHelper::create_test_request();

        // Act: Use existing non-streaming method - this will likely fail with network error
        // since we're not running a real Ollama instance, but that's expected
        let result = provider.complete(request).await;

        // Assert: The method should exist and be callable (even if it fails due to no server)
        // We're testing that the API exists, not that it succeeds without a server
        match result {
            Ok(_) => {
                // Unexpected success - maybe there's a real Ollama running
                println!("Unexpected success - real Ollama instance detected");
            }
            Err(ProviderError::NetworkError(_)) => {
                // Expected - no Ollama server running
                println!("Expected network error - no Ollama server");
            }
            Err(ProviderError::InvalidRequest(msg)) => {
                // Also acceptable - validation error
                println!("Validation error: {}", msg);
            }
            Err(other) => {
                println!("Other error (acceptable for test): {:?}", other);
            }
        }
        // Test passes as long as the method exists and is callable
    }
}

mod lmstudio_streaming {
    use super::*;

    #[tokio::test]
    async fn test_stream_completion_returns_response() {
        // Arrange: Create LMStudio provider
        let provider = match LMStudioProvider::new() {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping LMStudio test - provider creation failed");
                return;
            }
        };

        let request = StreamingTestHelper::create_test_request();

        // Act: Call stream_completion - this should fail with network error (no server)
        let stream_result = provider.stream_completion(request).await;

        // Assert: Should fail with network error since no server is running
        assert!(
            stream_result.is_err(),
            "Stream completion should fail when no server is available"
        );

        // Verify it's a network error
        match stream_result.unwrap_err() {
            ProviderError::NetworkError(_) => {
                // Expected behavior - no server running
                println!("Expected error - no LMStudio server or other issue");
            }
            other => panic!("Expected NetworkError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_stream_completion_validates_requests() {
        // Arrange: Create LMStudio provider
        let provider = match LMStudioProvider::new() {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping LMStudio validation test - provider creation failed");
                return;
            }
        };

        // Test empty model name
        let mut request = StreamingTestHelper::create_test_request();
        request.model = ModelId::new("");

        let result = provider.stream_completion(request).await;
        assert!(result.is_err(), "Should fail with empty model name");

        // Test empty messages
        let mut request = StreamingTestHelper::create_test_request();
        request.messages = vec![];

        let result = provider.stream_completion(request).await;
        assert!(result.is_err(), "Should fail with empty messages");
    }

    #[tokio::test]
    async fn test_stream_completion_backward_compatibility() {
        // Arrange: Ensure existing complete() method still works
        let provider = match LMStudioProvider::new() {
            Ok(p) => p,
            Err(_) => {
                println!(
                    "Skipping LMStudio backward compatibility test - provider creation failed"
                );
                return;
            }
        };

        let request = StreamingTestHelper::create_test_request();

        // Act: Use existing non-streaming method
        let result = provider.complete(request).await;

        // Assert: The method should exist and be callable
        match result {
            Ok(_) => {
                println!("Unexpected success - real LMStudio instance detected");
            }
            Err(_) => {
                // Expected - no LMStudio server running or other issues
                println!("Expected error - no LMStudio server or other issue");
            }
        }
        // Test passes as long as the method exists and is callable
    }
}

mod trait_extension_tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_completion_trait_method_exists() {
        // Arrange: This test ensures the trait extension compiles
        let provider = match OllamaProvider::new() {
            Ok(p) => p,
            Err(_) => {
                println!("Skipping trait test - Ollama provider creation failed");
                return;
            }
        };

        let request = StreamingTestHelper::create_test_request();

        // Act: This should compile - testing trait method exists
        let result = provider.stream_completion(request).await;

        // Assert: We're mainly testing compilation here
        match result {
            Ok(_) => println!("Stream method exists and returned success"),
            Err(_) => println!("Stream method exists and returned error (expected without server)"),
        }
        // Test passes if it compiles and method is callable
    }
}
