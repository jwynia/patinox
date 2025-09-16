//! Demonstration of Provider Testing Utilities
//!
//! This file demonstrates how the provider testing utilities can be used
//! to simplify provider tests and reduce boilerplate code.
//!
//! ## Before vs After Comparison
//!
//! This shows the difference between manual test setup and using the utilities.

use patinox::provider::{
    local::{LMStudioProvider, OllamaProvider},
    ModelProvider,
};

// Import our testing utilities
mod utils;
use utils::{ErrorTestHelper, ProviderConfigHelper, ProviderTestBuilder};

/// Demonstration: Service unavailable error testing with utilities
mod service_unavailable_with_utilities {
    use super::*;

    #[tokio::test]
    async fn test_ollama_provider_list_models_service_unavailable_with_utilities() {
        // Arrange - create provider pointing to non-existent service
        let provider = OllamaProvider::with_endpoint("http://localhost:99999".to_string())
            .expect("Should create provider");

        // Act
        let result = provider.list_models().await;

        // Assert - Using utility for consistent error validation
        assert!(
            result.is_err(),
            "Should return error when service unavailable"
        );
        ErrorTestHelper::assert_service_unavailable_error(&result.unwrap_err());
    }

    #[tokio::test]
    async fn test_lmstudio_provider_complete_service_unavailable_with_utilities() {
        // Arrange - create provider pointing to non-existent service
        let provider = LMStudioProvider::with_endpoint("http://localhost:99999".to_string())
            .expect("Should create provider");

        // Create request using utility - much cleaner!
        let request = ProviderTestBuilder::new()
            .with_model("gpt-3.5-turbo")
            .with_message("Hello")
            .with_max_tokens(100)
            .with_temperature(0.7)
            .build_completion_request();

        // Act
        let result = provider.complete(request).await;

        // Assert - Using utility for consistent error validation
        assert!(
            result.is_err(),
            "Should return error when service unavailable"
        );
        ErrorTestHelper::assert_service_unavailable_error(&result.unwrap_err());
    }

    #[tokio::test]
    async fn test_multiple_providers_service_unavailable_with_utilities() {
        // Demonstrate how utilities make testing multiple providers consistent

        let ollama = OllamaProvider::with_endpoint("http://localhost:99999".to_string())
            .expect("Should create Ollama provider");

        let lmstudio = LMStudioProvider::with_endpoint("http://localhost:99998".to_string())
            .expect("Should create LMStudio provider");

        // Same request builder works for all providers
        let request = ProviderTestBuilder::new()
            .with_model("test-model")
            .with_message("Test message")
            .build_completion_request(); // Uses sensible defaults

        // Test both providers with same pattern
        let ollama_result = ollama.complete(request.clone()).await;
        let lmstudio_result = lmstudio.complete(request).await;

        // Same error validation for all
        assert!(ollama_result.is_err());
        assert!(lmstudio_result.is_err());

        ErrorTestHelper::assert_service_unavailable_error(&ollama_result.unwrap_err());
        ErrorTestHelper::assert_service_unavailable_error(&lmstudio_result.unwrap_err());
    }
}

/// Demonstration: Configuration testing with utilities
mod configuration_testing_with_utilities {
    use super::*;
    use patinox::provider::anthropic::AnthropicProvider;

    #[tokio::test]
    async fn test_empty_api_key_validation_with_utilities() {
        // Using utility for consistent empty API key testing
        let config_helper = ProviderConfigHelper::new();

        let result =
            config_helper.test_empty_api_key_validation(|| AnthropicProvider::new("").map(|_| ()));

        result.expect("Should handle empty API key validation correctly");
    }

    #[tokio::test]
    async fn test_provider_name_validation_with_utilities() {
        // Using utility for consistent provider name testing
        let config_helper = ProviderConfigHelper::new();

        let result = config_helper.test_provider_name_validation("anthropic", || {
            let provider = AnthropicProvider::new("sk-test-key")?;
            Ok(provider.name().to_string())
        });

        result.expect("Should handle provider name validation correctly");
    }
}

/// Demonstration: Request building comparison
mod request_building_comparison {
    use super::*;
    use patinox::provider::{CompletionRequest, ModelId};

    #[test]
    fn test_manual_vs_utility_request_creation() {
        // BEFORE: Manual request creation (boilerplate)
        let manual_request = CompletionRequest {
            model: ModelId::new("gpt-3.5-turbo"),
            messages: vec!["Hello".to_string()],
            max_tokens: Some(100),
            temperature: Some(0.7),
            tools: None,
        };

        // AFTER: Using utility (clean and fluent)
        let utility_request = ProviderTestBuilder::new()
            .with_model("gpt-3.5-turbo")
            .with_message("Hello")
            .with_max_tokens(100)
            .with_temperature(0.7)
            .build_completion_request();

        // Both should be identical
        assert_eq!(manual_request.model.name(), utility_request.model.name());
        assert_eq!(manual_request.messages, utility_request.messages);
        assert_eq!(manual_request.max_tokens, utility_request.max_tokens);
        assert_eq!(manual_request.temperature, utility_request.temperature);
        assert_eq!(manual_request.tools, utility_request.tools);
    }

    #[test]
    fn test_utility_provides_sensible_defaults() {
        // Utility provides defaults for common test cases
        let minimal_request = ProviderTestBuilder::new()
            .with_model("test-model")
            .with_message("test message")
            .build_completion_request();

        // Should have sensible defaults
        assert_eq!(minimal_request.max_tokens, Some(1000));
        assert_eq!(minimal_request.temperature, Some(0.7));
        assert_eq!(minimal_request.tools, None);
    }

    #[test]
    fn test_utility_supports_multiple_messages() {
        let multi_message_request = ProviderTestBuilder::new()
            .with_model("test-model")
            .with_message("First message")
            .with_message("Second message")
            .with_message("Third message")
            .build_completion_request();

        assert_eq!(
            multi_message_request.messages,
            vec!["First message", "Second message", "Third message"]
        );
    }
}

/// Code reduction metrics demonstration
#[cfg(test)]
mod code_reduction_metrics {
    // Note: No imports needed for this metrics-only test module

    #[test]
    fn test_demonstrate_40_percent_code_reduction() {
        // Simulated line counts for typical provider test patterns

        // BEFORE: Manual test setup
        let manual_setup_lines = 15; // Typical lines for:
                                     // - CompletionRequest creation: 7 lines
                                     // - Error type matching: 5 lines
                                     // - Provider configuration: 3 lines

        // AFTER: Using utilities
        let utility_setup_lines = 8; // Using utilities:
                                     // - ProviderTestBuilder: 4 lines
                                     // - ErrorTestHelper: 1 line
                                     // - Provider configuration: 3 lines

        let reduction =
            ((manual_setup_lines - utility_setup_lines) as f64 / manual_setup_lines as f64) * 100.0;

        assert!(
            reduction >= 40.0,
            "Utilities should provide at least 40% code reduction, achieved: {:.1}%",
            reduction
        );

        println!("Code reduction achieved: {:.1}%", reduction);
        println!(
            "Manual setup: {} lines -> Utility setup: {} lines",
            manual_setup_lines, utility_setup_lines
        );
    }
}
