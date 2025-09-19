//! Real HTTP streaming tests for local providers (Ollama and LMStudio)
//!
//! This test suite validates real HTTP streaming implementations using mock servers
//! to simulate Ollama and LMStudio streaming responses.

use futures_util::StreamExt;
use mockito::{Matcher, Server};
use patinox::provider::local::{LMStudioProvider, OllamaProvider};
use patinox::provider::types::{CompletionRequest, ModelId, StreamingChunk};
use patinox::provider::{ModelProvider, ProviderError};
use serde_json::json;

/// Helper struct for testing real HTTP streaming
struct HttpStreamingTestHelper;

impl HttpStreamingTestHelper {
    /// Create a test completion request
    fn create_test_request() -> CompletionRequest {
        CompletionRequest {
            model: ModelId::new("test-model"),
            messages: vec!["Hello, how are you?".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        }
    }

    /// Collect all chunks from a streaming response
    async fn collect_chunks_from_stream<S>(
        mut stream: S,
    ) -> Result<Vec<StreamingChunk>, ProviderError>
    where
        S: futures_util::Stream<Item = Result<StreamingChunk, ProviderError>> + Unpin,
    {
        let mut chunks = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            chunks.push(chunk_result?);
        }
        Ok(chunks)
    }

    /// Validate streaming chunk sequence
    fn validate_chunk_sequence(chunks: &[StreamingChunk]) {
        assert!(!chunks.is_empty(), "Should have at least one chunk");

        // Check that only the last chunk is marked as final
        for (i, chunk) in chunks.iter().enumerate() {
            if i == chunks.len() - 1 {
                assert!(chunk.is_final, "Last chunk should be marked as final");
                assert!(chunk.model.is_some(), "Final chunk should have model info");
                assert!(chunk.finish_reason.is_some(), "Final chunk should have finish reason");
            } else {
                assert!(!chunk.is_final, "Non-final chunk should not be marked as final");
            }
        }

        // Verify content is present
        let combined_content: String = chunks.iter().map(|c| c.content.as_str()).collect();
        assert!(!combined_content.is_empty(), "Combined content should not be empty");
    }
}

mod ollama_real_http_streaming {
    use super::*;

    #[tokio::test]
    async fn test_ollama_real_http_streaming_success() {
        // Create a mock server for Ollama
        let mut server = Server::new_async().await;

        // Mock Ollama streaming response (newline-delimited JSON)
        let mock_response = r#"{"model":"test-model","created_at":"2023-12-12T14:13:43.416799Z","response":"Hello","done":false}
{"model":"test-model","created_at":"2023-12-12T14:13:43.416799Z","response":" there","done":false}
{"model":"test-model","created_at":"2023-12-12T14:13:43.416799Z","response":"!","done":true,"total_duration":5000000,"load_duration":1000000,"prompt_eval_count":10,"prompt_eval_duration":2000000,"eval_count":3,"eval_duration":2000000}
"#;

        let mock = server
            .mock("POST", "/api/generate")
            .match_header("content-type", "application/json")
            .match_body(Matcher::JsonString(json!({
                "model": "test-model",
                "prompt": "Hello, how are you?",
                "stream": true,
                "options": {
                    "temperature": 0.7,
                    "num_predict": 100
                }
            }).to_string()))
            .with_status(200)
            .with_header("content-type", "application/x-ndjson")
            .with_body(mock_response)
            .create_async()
            .await;

        // Create provider with mock server URL
        let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        // Test streaming completion
        let streaming_response = provider
            .stream_completion(request)
            .await
            .expect("Streaming should succeed");

        let chunks = HttpStreamingTestHelper::collect_chunks_from_stream(streaming_response)
            .await
            .expect("Should collect chunks successfully");

        // Validate response
        HttpStreamingTestHelper::validate_chunk_sequence(&chunks);
        assert_eq!(chunks.len(), 3, "Should have 3 chunks");

        // Check specific content
        assert_eq!(chunks[0].content, "Hello");
        assert_eq!(chunks[1].content, " there");
        assert_eq!(chunks[2].content, "!");

        // Check final chunk details
        let final_chunk = &chunks[2];
        assert_eq!(final_chunk.model.as_ref().unwrap().name(), "test-model");
        assert_eq!(final_chunk.finish_reason.as_ref().unwrap(), "stop");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_ollama_http_streaming_network_error() {
        // Create provider with invalid URL to simulate network error
        let provider = OllamaProvider::with_endpoint("http://invalid-host:11434".to_string())
            .expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        // Test that network errors are handled properly
        let result = provider.stream_completion(request).await;
        assert!(result.is_err(), "Should fail with network error");

        match result.unwrap_err() {
            ProviderError::NetworkError(_) => {
                // Expected error type
            }
            other => panic!("Expected NetworkError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_ollama_http_streaming_invalid_json() {
        let mut server = Server::new_async().await;

        // Mock response with invalid JSON
        let mock = server
            .mock("POST", "/api/generate")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/x-ndjson")
            .with_body("invalid json response\n")
            .create_async()
            .await;

        let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        // The error should occur during the initial parsing since we parse immediately
        let result = provider.stream_completion(request).await;
        assert!(result.is_err(), "Should fail to parse invalid JSON");

        match result.unwrap_err() {
            ProviderError::ParseError(_) => {
                // Expected error type
            }
            other => panic!("Expected ParseError, got: {:?}", other),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_ollama_http_streaming_server_error() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("POST", "/api/generate")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        let result = provider.stream_completion(request).await;
        assert!(result.is_err(), "Should fail with server error");

        mock.assert_async().await;
    }
}

mod lmstudio_real_http_streaming {
    use super::*;

    #[tokio::test]
    async fn test_lmstudio_real_http_streaming_success() {
        let mut server = Server::new_async().await;

        // Mock LMStudio SSE streaming response (OpenAI-compatible)
        let mock_response = r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"test-model","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"test-model","choices":[{"index":0,"delta":{"content":" there"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"test-model","choices":[{"index":0,"delta":{"content":"!"},"finish_reason":"stop"}],"usage":{"prompt_tokens":10,"completion_tokens":3,"total_tokens":13}}

data: [DONE]

"#;

        let mock = server
            .mock("POST", "/v1/chat/completions")
            .match_header("content-type", "application/json")
            .match_body(Matcher::JsonString(json!({
                "model": "test-model",
                "messages": [
                    {
                        "role": "user",
                        "content": "Hello, how are you?"
                    }
                ],
                "stream": true,
                "temperature": 0.7,
                "max_tokens": 100
            }).to_string()))
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(mock_response)
            .create_async()
            .await;

        // Create provider with mock server URL
        let provider = LMStudioProvider::with_endpoint(server.url()).expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        // Test streaming completion
        let streaming_response = provider
            .stream_completion(request)
            .await
            .expect("Streaming should succeed");

        let chunks = HttpStreamingTestHelper::collect_chunks_from_stream(streaming_response)
            .await
            .expect("Should collect chunks successfully");

        // Validate response
        HttpStreamingTestHelper::validate_chunk_sequence(&chunks);
        assert_eq!(chunks.len(), 3, "Should have 3 chunks");

        // Check specific content
        assert_eq!(chunks[0].content, "Hello");
        assert_eq!(chunks[1].content, " there");
        assert_eq!(chunks[2].content, "!");

        // Check final chunk details
        let final_chunk = &chunks[2];
        assert_eq!(final_chunk.model.as_ref().unwrap().name(), "test-model");
        assert_eq!(final_chunk.finish_reason.as_ref().unwrap(), "stop");
        assert!(final_chunk.usage.is_some(), "Final chunk should have usage info");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_lmstudio_http_streaming_network_error() {
        // Create provider with invalid URL to simulate network error
        let provider = LMStudioProvider::with_endpoint("http://invalid-host:1234".to_string())
            .expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        // Test that network errors are handled properly
        let result = provider.stream_completion(request).await;
        assert!(result.is_err(), "Should fail with network error");

        match result.unwrap_err() {
            ProviderError::NetworkError(_) => {
                // Expected error type
            }
            other => panic!("Expected NetworkError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_lmstudio_http_streaming_malformed_sse() {
        let mut server = Server::new_async().await;

        // Mock response with malformed SSE
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body("malformed sse data without proper format\n")
            .create_async()
            .await;

        let provider = LMStudioProvider::with_endpoint(server.url()).expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        let streaming_response = provider
            .stream_completion(request)
            .await
            .expect("Initial request should succeed");

        // The malformed SSE should result in an empty stream (graceful handling)
        let chunks = HttpStreamingTestHelper::collect_chunks_from_stream(streaming_response)
            .await
            .expect("Should handle malformed SSE gracefully");

        assert!(chunks.is_empty(), "Malformed SSE should result in empty stream");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_lmstudio_http_streaming_server_error() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(401)
            .with_body("Unauthorized")
            .create_async()
            .await;

        let provider = LMStudioProvider::with_endpoint(server.url()).expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        let result = provider.stream_completion(request).await;
        assert!(result.is_err(), "Should fail with server error");

        mock.assert_async().await;
    }
}

mod streaming_fallback_tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_fallback_to_non_streaming_on_error() {
        // Test that providers can gracefully fall back to non-streaming
        // if streaming fails (this is for future enhancement)

        let mut server = Server::new_async().await;

        // Mock streaming endpoint to fail
        let streaming_mock = server
            .mock("POST", "/api/generate")
            .match_body(Matcher::JsonString(json!({
                "model": "test-model",
                "prompt": "Hello, how are you?",
                "stream": true,
                "options": {
                    "temperature": 0.7,
                    "num_predict": 100
                }
            }).to_string()))
            .with_status(404)
            .create_async()
            .await;

        let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
        let request = HttpStreamingTestHelper::create_test_request();

        // This test documents the current behavior - streaming fails
        // In the future, we might want to implement automatic fallback
        let result = provider.stream_completion(request).await;
        assert!(result.is_err(), "Streaming should fail without fallback (current behavior)");

        streaming_mock.assert_async().await;
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_streaming_requests() {
        let mut server = Server::new_async().await;

        let mock_response = r#"{"model":"test-model","created_at":"2023-12-12T14:13:43.416799Z","response":"Response","done":false}
{"model":"test-model","created_at":"2023-12-12T14:13:43.416799Z","response":" chunk","done":true,"total_duration":5000000}
"#;

        let mock = server
            .mock("POST", "/api/generate")
            .expect_at_least(2)
            .with_status(200)
            .with_header("content-type", "application/x-ndjson")
            .with_body(mock_response)
            .create_async()
            .await;

        let server_url = server.url();

        // Send multiple concurrent streaming requests
        let tasks = (0..2).map(|i| {
            let url = server_url.clone();
            let mut request = HttpStreamingTestHelper::create_test_request();
            request.messages = vec![format!("Request {}", i)];

            tokio::spawn(async move {
                let provider = OllamaProvider::with_endpoint(url).expect("Failed to create provider");
                let response = provider.stream_completion(request).await?;
                HttpStreamingTestHelper::collect_chunks_from_stream(response).await
            })
        });

        let results: Vec<_> = futures::future::join_all(tasks).await;

        for result in results {
            let chunks = result.expect("Task should complete").expect("Should get chunks");
            assert!(!chunks.is_empty(), "Should have chunks");
        }

        mock.assert_async().await;
    }
}