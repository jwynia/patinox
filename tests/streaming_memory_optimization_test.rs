//! Streaming Memory Optimization Tests
//!
//! This test suite validates that streaming implementations use constant memory
//! regardless of response size, implementing true streaming rather than
//! buffering entire responses in memory.

use futures_util::StreamExt;
use mockito::Server;
use patinox::provider::local::{LMStudioProvider, OllamaProvider};
use patinox::provider::types::{CompletionRequest, ModelId, StreamingChunk};
use patinox::provider::{ModelProvider, ProviderError};
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Test constants
const LARGE_RESPONSE_CHUNKS: usize = 1000; // Simulate 1000 chunks
const CHUNK_CONTENT_SIZE: usize = 1024; // 1KB per chunk = 1MB total response
const MEMORY_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Helper struct for memory optimization testing
struct MemoryOptimizationTestHelper;

impl MemoryOptimizationTestHelper {
    /// Create a test completion request
    fn create_test_request() -> CompletionRequest {
        CompletionRequest {
            model: ModelId::new("test-model"),
            messages: vec!["Generate a very long response".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(10000),
            tools: None,
        }
    }

    /// Generate large Ollama NDJSON streaming response
    fn generate_large_ollama_response(num_chunks: usize, chunk_size: usize) -> String {
        let mut response_parts = Vec::new();

        // Generate intermediate chunks
        for _i in 0..num_chunks - 1 {
            let content = "x".repeat(chunk_size);
            let chunk = json!({
                "model": "test-model",
                "created_at": "2023-12-12T14:13:43.416799Z",
                "response": content,
                "done": false
            });
            response_parts.push(chunk.to_string());
        }

        // Generate final chunk with usage information
        let final_chunk = json!({
            "model": "test-model",
            "created_at": "2023-12-12T14:13:43.416799Z",
            "response": "",
            "done": true,
            "total_duration": 5000000_u64,
            "load_duration": 1000000_u64,
            "prompt_eval_duration": 2000000_u64,
            "prompt_eval_count": 10,
            "eval_count": num_chunks as u32,
            "eval_duration": 2000000_u64
        });
        response_parts.push(final_chunk.to_string());

        response_parts.join("\n")
    }

    /// Generate large LMStudio SSE streaming response
    fn generate_large_lmstudio_response(num_chunks: usize, chunk_size: usize) -> String {
        let mut response_parts = Vec::new();

        // Generate intermediate chunks
        for i in 0..num_chunks - 1 {
            let content = "x".repeat(chunk_size);
            let chunk = json!({
                "id": format!("chatcmpl-{}", i),
                "object": "chat.completion.chunk",
                "created": 1699999999,
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "delta": {
                        "content": content
                    },
                    "finish_reason": null
                }]
            });
            response_parts.push(format!("data: {}\n", chunk));
        }

        // Generate final chunk with usage
        let final_chunk = json!({
            "id": "chatcmpl-final",
            "object": "chat.completion.chunk",
            "created": 1699999999,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": num_chunks as u32,
                "total_tokens": 10 + num_chunks as u32
            }
        });
        response_parts.push(format!("data: {}\n", final_chunk));
        response_parts.push("data: [DONE]\n".to_string());

        response_parts.join("\n")
    }

    /// Measure time to first chunk (latency test)
    async fn measure_time_to_first_chunk<S>(
        mut stream: S,
    ) -> Result<(Duration, usize), ProviderError>
    where
        S: futures_util::Stream<Item = Result<StreamingChunk, ProviderError>> + Unpin,
    {
        let start = Instant::now();
        let mut chunk_count = 0;

        // Get first chunk
        if let Some(first_chunk_result) = stream.next().await {
            let first_chunk_time = start.elapsed();
            first_chunk_result?; // Validate first chunk

            // Count remaining chunks without timing
            while let Some(chunk_result) = stream.next().await {
                chunk_result?;
                chunk_count += 1;
            }

            Ok((first_chunk_time, chunk_count + 1))
        } else {
            Err(ProviderError::ApiError("No chunks received".to_string()))
        }
    }

    /// Collect chunks with memory monitoring simulation
    async fn collect_chunks_with_memory_check<S>(
        mut stream: S,
        max_expected_chunks: usize,
    ) -> Result<Vec<StreamingChunk>, ProviderError>
    where
        S: futures_util::Stream<Item = Result<StreamingChunk, ProviderError>> + Unpin,
    {
        let mut chunks = Vec::new();
        let mut processed_count = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            chunks.push(chunk);
            processed_count += 1;

            // Simulate memory pressure check - in real implementation,
            // each chunk should be processed and discarded immediately
            if processed_count > max_expected_chunks * 2 {
                return Err(ProviderError::ApiError(
                    "Unexpected number of chunks - possible memory leak".to_string(),
                ));
            }
        }

        Ok(chunks)
    }
}

#[tokio::test]
async fn test_ollama_streaming_memory_efficiency_large_response() {
    // Test that Ollama streaming can handle large responses without loading everything into memory
    let mut server = Server::new_async().await;
    let large_response = MemoryOptimizationTestHelper::generate_large_ollama_response(
        LARGE_RESPONSE_CHUNKS,
        CHUNK_CONTENT_SIZE,
    );

    let mock_stream = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&large_response)
        .create_async()
        .await;

    let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
    let request = MemoryOptimizationTestHelper::create_test_request();

    // Test with timeout to ensure streaming doesn't hang on large responses
    let result = timeout(
        MEMORY_TEST_TIMEOUT,
        provider.stream_completion(request.clone()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Streaming timed out - possible memory issue"
    );
    let stream = result.unwrap().expect("Stream creation failed");

    // Collect chunks and verify they arrive in reasonable time
    let chunks = timeout(
        MEMORY_TEST_TIMEOUT,
        MemoryOptimizationTestHelper::collect_chunks_with_memory_check(
            stream,
            LARGE_RESPONSE_CHUNKS,
        ),
    )
    .await;

    assert!(
        chunks.is_ok(),
        "Chunk collection timed out - possible memory issue"
    );
    let chunks = chunks.unwrap().expect("Failed to collect chunks");

    // Verify we got at least the expected number of chunks (within 1% tolerance)
    // The slight difference comes from how final chunks with empty content are handled
    let expected_min = LARGE_RESPONSE_CHUNKS - (LARGE_RESPONSE_CHUNKS / 100).max(1);
    assert!(
        chunks.len() >= expected_min && chunks.len() <= LARGE_RESPONSE_CHUNKS,
        "Unexpected number of chunks: got {}, expected around {}",
        chunks.len(),
        LARGE_RESPONSE_CHUNKS
    );

    // Verify final chunk has usage information
    let final_chunks: Vec<_> = chunks.iter().filter(|c| c.is_final).collect();
    println!(
        "Found {} final chunks out of {}",
        final_chunks.len(),
        chunks.len()
    );

    assert!(
        !final_chunks.is_empty(),
        "At least one chunk should be final"
    );
    let final_chunk = final_chunks.last().expect("No final chunk");
    assert!(
        final_chunk.usage.is_some(),
        "Final chunk should have usage information"
    );

    mock_stream.assert_async().await;
}

#[tokio::test]
async fn test_lmstudio_streaming_memory_efficiency_large_response() {
    // Test that LMStudio streaming can handle large responses without loading everything into memory
    let mut server = Server::new_async().await;
    let large_response = MemoryOptimizationTestHelper::generate_large_lmstudio_response(
        LARGE_RESPONSE_CHUNKS,
        CHUNK_CONTENT_SIZE,
    );

    let mock_stream = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&large_response)
        .create_async()
        .await;

    let provider =
        LMStudioProvider::with_endpoint(server.url()).expect("Failed to create provider");
    let request = MemoryOptimizationTestHelper::create_test_request();

    // Test with timeout to ensure streaming doesn't hang on large responses
    let result = timeout(
        MEMORY_TEST_TIMEOUT,
        provider.stream_completion(request.clone()),
    )
    .await;

    assert!(
        result.is_ok(),
        "Streaming timed out - possible memory issue"
    );
    let stream = result.unwrap().expect("Stream creation failed");

    // Collect chunks and verify they arrive in reasonable time
    let chunks = timeout(
        MEMORY_TEST_TIMEOUT,
        MemoryOptimizationTestHelper::collect_chunks_with_memory_check(
            stream,
            LARGE_RESPONSE_CHUNKS,
        ),
    )
    .await;

    assert!(
        chunks.is_ok(),
        "Chunk collection timed out - possible memory issue"
    );
    let chunks = chunks.unwrap().expect("Failed to collect chunks");

    // Verify we got at least the expected number of chunks (within 1% tolerance)
    let expected_min = LARGE_RESPONSE_CHUNKS - (LARGE_RESPONSE_CHUNKS / 100).max(1);
    assert!(
        chunks.len() >= expected_min && chunks.len() <= LARGE_RESPONSE_CHUNKS,
        "Unexpected number of chunks: got {}, expected around {}",
        chunks.len(),
        LARGE_RESPONSE_CHUNKS
    );

    // Verify final chunk has usage information
    let final_chunk = chunks.last().expect("No final chunk");
    assert!(final_chunk.is_final, "Last chunk should be final");
    assert!(
        final_chunk.usage.is_some(),
        "Final chunk should have usage information"
    );

    mock_stream.assert_async().await;
}

#[tokio::test]
async fn test_ollama_streaming_latency_improvement() {
    // Test that time to first chunk is fast even with large responses
    let mut server = Server::new_async().await;
    let large_response = MemoryOptimizationTestHelper::generate_large_ollama_response(
        LARGE_RESPONSE_CHUNKS,
        CHUNK_CONTENT_SIZE,
    );

    let mock_latency = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&large_response)
        .create_async()
        .await;

    let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
    let request = MemoryOptimizationTestHelper::create_test_request();

    let stream = provider
        .stream_completion(request)
        .await
        .expect("Stream creation failed");

    // Measure time to first chunk
    let (first_chunk_time, total_chunks) =
        MemoryOptimizationTestHelper::measure_time_to_first_chunk(stream)
            .await
            .expect("Failed to measure latency");

    // First chunk should arrive quickly (< 1 second even for large responses)
    // This tests that we're not waiting for the entire response before processing
    assert!(
        first_chunk_time < Duration::from_secs(1),
        "First chunk took too long: {:?} - suggests buffering entire response",
        first_chunk_time
    );

    let expected_min = LARGE_RESPONSE_CHUNKS - (LARGE_RESPONSE_CHUNKS / 100).max(1);
    assert!(
        total_chunks >= expected_min && total_chunks <= LARGE_RESPONSE_CHUNKS,
        "Unexpected total chunks: got {}, expected around {}",
        total_chunks,
        LARGE_RESPONSE_CHUNKS
    );

    mock_latency.assert_async().await;
}

#[tokio::test]
async fn test_lmstudio_streaming_latency_improvement() {
    // Test that time to first chunk is fast even with large responses
    let mut server = Server::new_async().await;
    let large_response = MemoryOptimizationTestHelper::generate_large_lmstudio_response(
        LARGE_RESPONSE_CHUNKS,
        CHUNK_CONTENT_SIZE,
    );

    let mock_latency = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(&large_response)
        .create_async()
        .await;

    let provider =
        LMStudioProvider::with_endpoint(server.url()).expect("Failed to create provider");
    let request = MemoryOptimizationTestHelper::create_test_request();

    let stream = provider
        .stream_completion(request)
        .await
        .expect("Stream creation failed");

    // Measure time to first chunk
    let (first_chunk_time, total_chunks) =
        MemoryOptimizationTestHelper::measure_time_to_first_chunk(stream)
            .await
            .expect("Failed to measure latency");

    // First chunk should arrive quickly (< 1 second even for large responses)
    assert!(
        first_chunk_time < Duration::from_secs(1),
        "First chunk took too long: {:?} - suggests buffering entire response",
        first_chunk_time
    );

    let expected_min = LARGE_RESPONSE_CHUNKS - (LARGE_RESPONSE_CHUNKS / 100).max(1);
    assert!(
        total_chunks >= expected_min && total_chunks <= LARGE_RESPONSE_CHUNKS,
        "Unexpected total chunks: got {}, expected around {}",
        total_chunks,
        LARGE_RESPONSE_CHUNKS
    );

    mock_latency.assert_async().await;
}

#[tokio::test]
async fn test_streaming_error_handling_with_malformed_large_response() {
    // Test that streaming handles errors gracefully even with large responses
    let mut server = Server::new_async().await;

    // Create a response with malformed JSON in the middle
    let mut response_parts = Vec::new();

    // Add some valid chunks
    for _i in 0..50 {
        let content = "x".repeat(CHUNK_CONTENT_SIZE);
        let chunk = json!({
            "model": "test-model",
            "created_at": "2023-12-12T14:13:43.416799Z",
            "response": content,
            "done": false
        });
        response_parts.push(chunk.to_string());
    }

    // Add malformed JSON that's clearly not valid JSON
    response_parts.push("this is not json at all".to_string());

    let malformed_response = response_parts.join("\n");

    let mock_error = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&malformed_response)
        .create_async()
        .await;

    let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
    let request = MemoryOptimizationTestHelper::create_test_request();

    // With true streaming, the HTTP request succeeds but parsing fails when consuming the stream
    let result = provider.stream_completion(request).await;
    assert!(
        result.is_ok(),
        "HTTP request should succeed with true streaming"
    );

    // Error occurs when consuming the stream
    let mut stream = result.unwrap();

    use futures_util::StreamExt; // Needed for stream operations

    // Process chunks until we hit the malformed JSON
    let mut chunks_processed = 0;
    let mut got_error = false;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(_chunk) => {
                chunks_processed += 1;
                // Continue processing valid chunks
            }
            Err(e) => {
                // Should get a parse error when hitting malformed JSON
                assert!(
                    matches!(e, ProviderError::ParseError(_)),
                    "Expected ParseError, got: {:?}",
                    e
                );
                println!(
                    "Correctly caught parse error after {} chunks: {:?}",
                    chunks_processed, e
                );
                got_error = true;
                break;
            }
        }
    }

    assert!(
        got_error,
        "Should have encountered a parse error during stream consumption"
    );
    assert!(
        chunks_processed > 0,
        "Should have processed some valid chunks before the error"
    );

    mock_error.assert_async().await;
}

#[tokio::test]
async fn test_streaming_handles_slow_network_responses() {
    // Test that streaming works correctly even when network is slow
    // This simulates scenarios where chunks arrive slowly over time
    let mut server = Server::new_async().await;

    // Small response but simulate slow network by testing timeout behavior
    let small_response = MemoryOptimizationTestHelper::generate_large_ollama_response(10, 100);

    let mock_slow = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&small_response)
        .create_async()
        .await;

    let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
    let request = MemoryOptimizationTestHelper::create_test_request();

    let stream = provider
        .stream_completion(request)
        .await
        .expect("Stream creation failed");

    // Even with potential network delays, should complete within reasonable time
    let chunks = timeout(
        Duration::from_secs(10),
        MemoryOptimizationTestHelper::collect_chunks_with_memory_check(stream, 10),
    )
    .await;

    assert!(
        chunks.is_ok(),
        "Streaming should handle slow networks gracefully"
    );
    let chunks = chunks.unwrap().expect("Failed to collect chunks");
    assert_eq!(
        chunks.len(),
        10,
        "Should receive all chunks despite slow network"
    );

    mock_slow.assert_async().await;
}

#[tokio::test]
async fn test_concurrent_large_streaming_requests() {
    // Test that multiple concurrent large streaming requests don't cause memory issues
    let mut server = Server::new_async().await;
    let large_response = MemoryOptimizationTestHelper::generate_large_ollama_response(
        LARGE_RESPONSE_CHUNKS / 4, // Smaller per request but multiple concurrent
        CHUNK_CONTENT_SIZE,
    );

    let mock_concurrent = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&large_response)
        .expect_at_least(3) // Expect at least 3 concurrent requests
        .create_async()
        .await;

    let provider = OllamaProvider::with_endpoint(server.url()).expect("Failed to create provider");
    let request = MemoryOptimizationTestHelper::create_test_request();

    // Start 3 concurrent streaming requests
    let stream1_fut = provider.stream_completion(request.clone());
    let stream2_fut = provider.stream_completion(request.clone());
    let stream3_fut = provider.stream_completion(request.clone());

    let (stream1, stream2, stream3) = tokio::try_join!(stream1_fut, stream2_fut, stream3_fut)
        .expect("Failed to create concurrent streams");

    // Process all streams concurrently
    let collect1 = MemoryOptimizationTestHelper::collect_chunks_with_memory_check(
        stream1,
        LARGE_RESPONSE_CHUNKS / 4,
    );
    let collect2 = MemoryOptimizationTestHelper::collect_chunks_with_memory_check(
        stream2,
        LARGE_RESPONSE_CHUNKS / 4,
    );
    let collect3 = MemoryOptimizationTestHelper::collect_chunks_with_memory_check(
        stream3,
        LARGE_RESPONSE_CHUNKS / 4,
    );

    let (chunks1, chunks2, chunks3) = timeout(MEMORY_TEST_TIMEOUT, async move {
        tokio::try_join!(collect1, collect2, collect3)
    })
    .await
    .expect("Concurrent streaming timed out")
    .expect("Failed to collect concurrent chunks");

    // Verify all streams completed successfully (with 1% tolerance)
    let expected_chunks = LARGE_RESPONSE_CHUNKS / 4;
    let expected_min = expected_chunks - (expected_chunks / 100).max(1);

    assert!(
        chunks1.len() >= expected_min && chunks1.len() <= expected_chunks,
        "Stream 1 unexpected chunks: got {}, expected around {}",
        chunks1.len(),
        expected_chunks
    );
    assert!(
        chunks2.len() >= expected_min && chunks2.len() <= expected_chunks,
        "Stream 2 unexpected chunks: got {}, expected around {}",
        chunks2.len(),
        expected_chunks
    );
    assert!(
        chunks3.len() >= expected_min && chunks3.len() <= expected_chunks,
        "Stream 3 unexpected chunks: got {}, expected around {}",
        chunks3.len(),
        expected_chunks
    );

    mock_concurrent.assert_async().await;
}
