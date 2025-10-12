//! Provider Testing Utilities Module
//!
//! This module provides reusable testing utilities to reduce boilerplate
//! and ensure consistent testing patterns across all provider implementations.
//!
//! ## Key Utilities
//!
//! - `ProviderTestBuilder`: Fluent builder for creating test requests
//! - `MockHttpBuilder`: Standardized HTTP mock response builder
//! - `ErrorTestHelper`: Consistent error validation patterns
//! - `ProviderConfigHelper`: Configuration testing utilities
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::utils::{ProviderTestBuilder, MockHttpBuilder, ErrorTestHelper};
//!
//! // Create a test completion request
//! let request = ProviderTestBuilder::new()
//!     .with_model("gpt-3.5-turbo")
//!     .with_message("Hello")
//!     .build_completion_request();
//!
//! // Create a mock HTTP response
//! let mock = MockHttpBuilder::new()
//!     .service_unavailable()
//!     .with_endpoint("/v1/models")
//!     .build();
//!
//! // Validate error types consistently
//! ErrorTestHelper::assert_service_unavailable_error(&error);
//! ```

use patinox::provider::{CompletionRequest, ModelId, ProviderError};
use std::time::Duration;

// Constants for default values to improve maintainability
const DEFAULT_MAX_TOKENS: usize = 1000;
const DEFAULT_TEMPERATURE: f32 = 0.7;
#[allow(dead_code)] // Used in future mock response implementations
const MOCK_TIMESTAMP: i64 = 1677610602; // 2023-02-28 - Fixed timestamp for consistent testing

/// Builder for creating test completion requests with sensible defaults
pub struct ProviderTestBuilder {
    model: Option<String>,
    messages: Vec<String>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
}

impl ProviderTestBuilder {
    pub fn new() -> Self {
        Self {
            model: None,
            messages: Vec::new(),
            max_tokens: None,
            temperature: None,
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.messages.push(message.to_string());
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn build_completion_request(self) -> CompletionRequest {
        CompletionRequest {
            model: ModelId::new(self.model.expect("ProviderTestBuilder: model must be set via with_model() before calling build_completion_request()")),
            messages: self.messages,
            max_tokens: self.max_tokens.or(Some(DEFAULT_MAX_TOKENS)),
            temperature: self.temperature.or(Some(DEFAULT_TEMPERATURE)),
            tools: None,
        }
    }
}

/// Builder for creating mock HTTP responses
#[allow(dead_code)] // Some methods intended for future use
pub struct MockHttpBuilder {
    status_code: Option<u16>,
    endpoint: Option<String>,
    error_message: Option<String>,
    response_body: Option<String>,
    retry_after: Option<Duration>,
}

#[allow(dead_code)] // Testing utilities for use in provider integration tests
impl MockHttpBuilder {
    pub fn new() -> Self {
        Self {
            status_code: None,
            endpoint: None,
            error_message: None,
            response_body: None,
            retry_after: None,
        }
    }

    pub fn service_unavailable(mut self) -> Self {
        self.status_code = Some(503);
        self.error_message = Some("Service Temporarily Unavailable".to_string());
        self
    }

    pub fn authentication_error(mut self) -> Self {
        self.status_code = Some(401);
        self.error_message = Some("Unauthorized".to_string());
        self
    }

    pub fn success(mut self) -> Self {
        self.status_code = Some(200);
        self
    }

    pub fn rate_limit_error(mut self) -> Self {
        self.status_code = Some(429);
        self.error_message = Some("Too Many Requests".to_string());
        self
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = Some(endpoint.to_string());
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.error_message = Some(message.to_string());
        self
    }

    pub fn with_models_response(self, models: &[&str]) -> Self {
        // Create a JSON response for models list
        let models_json = models
            .iter()
            .map(|model| {
                format!(
                    r#"{{"id":"{}","object":"model","created":{}}}"#,
                    model, MOCK_TIMESTAMP
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let response_body = format!(r#"{{"data":[{}]}}"#, models_json);

        MockHttpBuilder {
            endpoint: self.endpoint,
            status_code: self.status_code,
            error_message: Some(response_body),
            response_body: None,
            retry_after: self.retry_after,
        }
    }

    pub fn with_completion_response(self, response: &str) -> Self {
        // Create a JSON response for completion
        let response_body = format!(
            r#"{{"object":"chat.completion","choices":[{{"message":{{"content":"{}","role":"assistant"}},"finish_reason":"stop"}}]}}"#,
            response
        );

        MockHttpBuilder {
            endpoint: self.endpoint,
            status_code: self.status_code,
            error_message: Some(response_body),
            response_body: None,
            retry_after: self.retry_after,
        }
    }

    pub fn with_retry_after(mut self, duration: Duration) -> Self {
        self.retry_after = Some(duration);
        self
    }

    // Streaming-specific mock methods
    #[allow(dead_code)]
    pub fn with_streaming_response(mut self) -> Self {
        // Set up for streaming response
        self.status_code = Some(200);
        self.response_body = Some("streaming_chunks".to_string()); // Placeholder for streaming
        self
    }

    #[allow(dead_code)]
    pub fn with_chunks(mut self, chunks: Vec<&str>) -> Self {
        // Store chunks as JSON array for test parsing
        let chunks_json: Vec<String> = chunks.iter().map(|c| c.to_string()).collect();
        self.response_body = Some(serde_json::to_string(&chunks_json).unwrap());
        self
    }

    #[allow(dead_code)]
    pub fn with_streaming_chunks(self, chunks: Vec<&str>) -> Self {
        self.with_streaming_response().with_chunks(chunks)
    }

    #[allow(dead_code)]
    pub fn with_openai_streaming_response(mut self) -> Self {
        // Set up for OpenAI-compatible streaming
        self.status_code = Some(200);
        self.response_body = Some("openai_streaming".to_string());
        self
    }

    #[allow(dead_code)]
    pub fn with_openai_streaming_chunks(mut self, chunks: Vec<&str>) -> Self {
        // Format as OpenAI streaming chunks
        let formatted_chunks: Vec<String> = chunks
            .iter()
            .map(|content| {
                format!(
                    r#"data: {{"choices":[{{"delta":{{"content":"{}"}}}}]}}"#,
                    content
                )
            })
            .collect();
        self.response_body = Some(serde_json::to_string(&formatted_chunks).unwrap());
        self
    }

    #[allow(dead_code)]
    pub fn with_connection_error(mut self) -> Self {
        self.status_code = Some(0); // Indicate connection failure
        self.error_message = Some("Connection failed".to_string());
        self
    }

    #[allow(dead_code)]
    pub fn with_partial_streaming_response(mut self, partial_chunks: Vec<&str>) -> Self {
        // Simulate a stream that cuts off
        self.status_code = Some(200);
        self.response_body = Some(format!(
            "partial:{}",
            serde_json::to_string(&partial_chunks).unwrap()
        ));
        self
    }

    #[allow(dead_code)]
    pub fn with_slow_streaming_response(mut self, delay: Duration) -> Self {
        // Simulate slow streaming
        self.status_code = Some(200);
        self.response_body = Some(format!("slow:{}ms", delay.as_millis()));
        self
    }

    #[allow(dead_code)]
    pub fn with_malformed_sse_response(mut self) -> Self {
        // Simulate malformed Server-Sent Events
        self.status_code = Some(200);
        self.response_body = Some("malformed_sse".to_string());
        self
    }

    #[allow(dead_code)]
    pub fn with_openai_completion_response(mut self) -> Self {
        // Standard OpenAI completion response for backward compatibility tests
        self.status_code = Some(200);
        self.response_body =
            Some(r#"{"choices":[{"message":{"content":"Test response"}}]}"#.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn expect_json_body_contains(mut self, expected_fragment: &str) -> Self {
        // Store expectation for request body validation
        self.response_body = Some(format!("expect:{}", expected_fragment));
        self
    }

    pub fn build(self) -> MockHttpResponse {
        let error_message = self.error_message.unwrap_or_default();
        let response_body = self.response_body.unwrap_or_else(|| {
            if error_message.is_empty() {
                "{}".to_string()
            } else {
                error_message.clone()
            }
        });

        MockHttpResponse {
            status_code: self.status_code.unwrap_or(200),
            endpoint: self.endpoint.unwrap_or_else(|| "/".to_string()),
            error_message,
            response_body,
            retry_after: self.retry_after,
        }
    }
}

/// Mock HTTP response for testing
#[allow(dead_code)] // Methods intended for future use
pub struct MockHttpResponse {
    status_code: u16,
    endpoint: String,
    error_message: String,
    response_body: String,
    retry_after: Option<Duration>,
}

#[allow(dead_code)] // Response accessors for use in provider integration tests
impl MockHttpResponse {
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn error_message(&self) -> &str {
        &self.error_message
    }

    pub fn response_body(&self) -> &str {
        &self.response_body
    }

    pub fn retry_after(&self) -> Option<Duration> {
        self.retry_after
    }
}

/// Helper for consistent error testing
pub struct ErrorTestHelper;

impl ErrorTestHelper {
    #[allow(dead_code)] // Methods intended for future provider test usage
    pub fn assert_service_unavailable_error(error: &ProviderError) {
        match error {
            ProviderError::NetworkError(_) => {
                // Service unavailable typically manifests as network errors
            }
            ProviderError::ApiError(msg) if msg.contains("unavailable") || msg.contains("503") => {
                // API errors indicating service unavailability
            }
            _ => panic!("Expected service unavailable error, got: {:?}", error),
        }
    }

    #[allow(dead_code)] // For use in authentication error testing scenarios
    pub fn assert_authentication_error(error: &ProviderError) {
        match error {
            ProviderError::AuthenticationError(_) => {
                // Expected authentication error
            }
            ProviderError::ApiError(msg) if msg.contains("401") || msg.contains("Unauthorized") => {
                // API errors indicating authentication failure
            }
            _ => panic!("Expected authentication error, got: {:?}", error),
        }
    }

    #[allow(dead_code)] // For use in API error validation scenarios
    pub fn assert_api_error(error: &ProviderError, expected_message: &str) {
        match error {
            ProviderError::ApiError(msg) => {
                assert!(
                    msg.contains(expected_message),
                    "Expected API error to contain '{}', got: '{}'",
                    expected_message,
                    msg
                );
            }
            _ => panic!(
                "Expected API error with message '{}', got: {:?}",
                expected_message, error
            ),
        }
    }
}

/// Helper for provider configuration testing
pub struct ProviderConfigHelper;

impl ProviderConfigHelper {
    pub fn new() -> Self {
        Self
    }

    pub fn test_empty_api_key_validation<F>(&self, provider_fn: F) -> Result<(), String>
    where
        F: FnOnce() -> Result<(), ProviderError>,
    {
        match provider_fn() {
            Err(ProviderError::ConfigurationError(msg)) => {
                if msg.contains("API key") && msg.contains("empty") {
                    Ok(())
                } else {
                    Err(format!("Expected empty API key error, got: {}", msg))
                }
            }
            Err(other_error) => Err(format!(
                "Expected ConfigurationError, got: {:?}",
                other_error
            )),
            Ok(()) => Err("Expected error for empty API key, but got success".to_string()),
        }
    }

    #[allow(dead_code)] // For use in base URL configuration testing
    pub fn test_base_url_configuration<F>(
        &self,
        expected_url: &str,
        provider_fn: F,
    ) -> Result<(), String>
    where
        F: FnOnce() -> Result<String, ProviderError>,
    {
        match provider_fn() {
            Ok(actual_url) => {
                if actual_url == expected_url {
                    Ok(())
                } else {
                    Err(format!(
                        "Expected base URL '{}', got '{}'",
                        expected_url, actual_url
                    ))
                }
            }
            Err(error) => Err(format!(
                "Expected successful base URL configuration, got error: {:?}",
                error
            )),
        }
    }

    pub fn test_provider_name_validation<F>(
        &self,
        expected_name: &str,
        provider_fn: F,
    ) -> Result<(), String>
    where
        F: FnOnce() -> Result<String, ProviderError>,
    {
        match provider_fn() {
            Ok(actual_name) => {
                if actual_name == expected_name {
                    Ok(())
                } else {
                    Err(format!(
                        "Expected provider name '{}', got '{}'",
                        expected_name, actual_name
                    ))
                }
            }
            Err(error) => Err(format!(
                "Expected successful provider name validation, got error: {:?}",
                error
            )),
        }
    }

    // Streaming-specific error assertion methods
    #[allow(dead_code)]
    pub fn assert_connection_error(error: &ProviderError) {
        match error {
            ProviderError::NetworkError(_) => {
                // Connection errors are expected to be network errors
            }
            other => panic!(
                "Expected NetworkError for connection failure, got: {:?}",
                other
            ),
        }
    }

    #[allow(dead_code)]
    pub fn assert_incomplete_stream_error(error: &ProviderError) {
        match error {
            ProviderError::StreamError(_) => {
                // Stream errors are expected for incomplete streams
            }
            ProviderError::ApiError(msg)
                if msg.contains("incomplete") || msg.contains("stream") =>
            {
                // API errors can also indicate stream issues
            }
            other => panic!(
                "Expected StreamError for incomplete stream, got: {:?}",
                other
            ),
        }
    }

    #[allow(dead_code)]
    pub fn assert_model_not_found_error(error: &ProviderError) {
        match error {
            ProviderError::ApiError(msg) if msg.contains("not found") || msg.contains("404") => {
                // Model not found typically manifests as API error
            }
            other => panic!("Expected model not found error, got: {:?}", other),
        }
    }

    #[allow(dead_code)]
    pub fn assert_parse_error(error: &ProviderError) {
        match error {
            ProviderError::ParseError(_) => {
                // Parse errors are expected for malformed responses
            }
            ProviderError::ApiError(msg) if msg.contains("parse") || msg.contains("malformed") => {
                // API errors can also indicate parsing issues
            }
            other => panic!(
                "Expected ParseError for malformed response, got: {:?}",
                other
            ),
        }
    }
}
