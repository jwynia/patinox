# Task 007: Implement Real HTTP Streaming

## Overview
**Type**: Feature Enhancement
**Priority**: Medium
**Effort**: Large
**Created**: 2025-09-16
**Source**: Code Review Recommendation

## Problem Statement
Current streaming implementation uses hardcoded mock responses instead of real HTTP streaming. This limits the providers to returning fixed test data rather than actual LLM responses.

**Current State**:
```rust
// Mock stream implementation
let stream = stream::iter(vec![
    Ok(StreamingChunk::new("Hello".to_string(), false)),
    Ok(StreamingChunk::new(" world".to_string(), false)),
    // ...
]);
```

## Acceptance Criteria

### Ollama Provider Streaming

1. **HTTP Integration**:
   - Make HTTP POST to `/api/generate` with `stream: true`
   - Handle HTTP response stream using `reqwest::Response::bytes_stream()`
   - Parse newline-delimited JSON responses from Ollama

2. **Response Processing**:
   - Convert `OllamaGenerateResponse` to `StreamingChunk`
   - Handle completion detection via `done` field
   - Accumulate usage statistics for final chunk

3. **Error Handling**:
   - Handle network errors gracefully
   - Parse streaming response errors
   - Implement proper timeout handling

### LMStudio Provider Streaming

1. **HTTP Integration**:
   - Make HTTP POST to `/v1/chat/completions` with `stream: true`
   - Handle Server-Sent Events in OpenAI format
   - Parse `data:` lines and handle `data: [DONE]` completion

2. **Response Processing**:
   - Convert OpenAI delta format to `StreamingChunk`
   - Handle partial content accumulation
   - Process choice finish reasons

3. **Error Handling**:
   - Handle SSE parsing errors
   - Manage connection timeouts
   - Parse streaming error responses

### Testing Requirements

1. **Integration Tests**:
   - Add tests with real HTTP mocking (using `mockito`)
   - Test streaming response parsing
   - Test error conditions and edge cases

2. **Mock Server Tests**:
   - Create mock Ollama server responses
   - Create mock LMStudio SSE responses
   - Test timeout and retry scenarios

3. **Performance Tests**:
   - Verify streaming performance vs non-streaming
   - Test memory usage with large responses
   - Validate backpressure handling

## Implementation Plan

### Phase 1: Ollama Streaming
1. Implement HTTP streaming request
2. Add newline-delimited JSON parser
3. Convert response format
4. Add integration tests

### Phase 2: LMStudio Streaming
1. Implement SSE parser
2. Handle OpenAI delta format
3. Add completion detection
4. Add integration tests

### Phase 3: Error Handling & Polish
1. Comprehensive error handling
2. Timeout configuration
3. Performance optimization
4. Documentation updates

## Technical Considerations

### Dependencies
- May need additional HTTP streaming dependencies
- SSE parsing library for LMStudio
- Enhanced testing infrastructure

### Performance
- Stream processing should be memory-efficient
- Avoid buffering entire responses
- Implement proper backpressure

### Configuration
- Configurable timeout values
- Stream buffer sizes
- Retry policies for streaming requests

## Files to Modify
- `src/provider/local/ollama.rs` - Replace mock with real HTTP streaming
- `src/provider/local/lmstudio.rs` - Replace mock with real SSE streaming
- `tests/local_provider_streaming_test.rs` - Add real HTTP tests
- `Cargo.toml` - Add streaming dependencies if needed

## Dependencies
- Current streaming types and traits (already implemented)
- HTTP client infrastructure (reqwest)
- Potentially new dependencies for SSE parsing

## Risk Assessment
- **Risk Level**: High (network I/O, complex streaming protocols)
- **Complexity**: High (HTTP streaming, SSE parsing, error handling)
- **Testing Requirements**: Extensive (network mocking, edge cases)

## Success Metrics
- Real HTTP streaming responses from both providers
- Memory-efficient stream processing
- Comprehensive error handling
- Performance comparable to non-streaming methods
- 100% test coverage for streaming paths