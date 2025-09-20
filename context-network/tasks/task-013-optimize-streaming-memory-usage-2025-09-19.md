# Task 013: Optimize Streaming Memory Usage

## Overview
**Type**: Performance Enhancement
**Priority**: High
**Effort**: Large
**Created**: 2025-09-19
**Source**: Code Review Recommendation

## Problem Statement
Current streaming implementation loads entire HTTP responses into memory before processing, which could cause memory issues with large responses. This defeats the purpose of streaming for memory efficiency.

**Current Anti-Pattern**:
```rust
// Loads entire response into memory at once
let response_text = response
    .text()
    .await
    .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

// Then processes all lines
response_text.lines().filter(...).map(...)
```

**Impact**:
- Large streaming responses consume significant memory
- Negates memory benefits of streaming
- Could cause issues with very large model outputs
- Not truly "streaming" - more like "batch processing with delays"

## Acceptance Criteria

### Performance Requirements
1. **True Streaming Processing**:
   - Process response data as it arrives, not after complete download
   - Memory usage should remain constant regardless of response size
   - Implement line-by-line processing for both Ollama (NDJSON) and LMStudio (SSE)

2. **Maintain Functionality**:
   - All existing streaming behavior preserved
   - Same error handling patterns
   - Compatible with existing StreamingChunk interface
   - No breaking changes to public APIs

3. **Memory Efficiency**:
   - Peak memory usage independent of response size
   - Proper cleanup of processed chunks
   - No memory leaks during long streaming sessions

### Implementation Options

#### Option A: Async Line Reader (Recommended)
Use `tokio::io::BufReader` with `lines()` for true streaming:
```rust
use tokio::io::{AsyncBufReadExt, BufReader};

let reader = BufReader::new(response.into_reader());
let mut lines = reader.lines();

while let Some(line) = lines.next_line().await? {
    // Process each line as it arrives
    let chunk = parse_line(&line)?;
    yield chunk;
}
```

#### Option B: Bytes Stream with Buffer
Process bytes incrementally and buffer until complete lines:
```rust
use futures_util::TryStreamExt;

let mut buffer = String::new();
let mut bytes_stream = response.bytes_stream();

while let Some(chunk) = bytes_stream.try_next().await? {
    buffer.push_str(&String::from_utf8_lossy(&chunk));

    // Process complete lines and keep remainder
    while let Some(newline_pos) = buffer.find('\n') {
        let line = buffer[..newline_pos].to_string();
        buffer.drain(..=newline_pos);

        let chunk = parse_line(&line)?;
        yield chunk;
    }
}
```

### Testing Requirements
1. **Memory Usage Tests**:
   - Verify memory usage stays constant with increasing response size
   - Test with large mock responses (>10MB)
   - Monitor for memory leaks during extended streaming

2. **Performance Benchmarks**:
   - Compare memory usage: before vs. after optimization
   - Measure latency: time to first chunk
   - Ensure throughput is not negatively impacted

3. **Regression Testing**:
   - All existing streaming tests pass
   - Same chunking behavior and timing
   - Error handling unchanged

## Implementation Guide

### Phase 1: Ollama Provider (NDJSON)
1. Replace `response.text().await` with streaming line reader
2. Process each JSON line as it arrives
3. Update error handling for streaming context
4. Test with large mock responses

### Phase 2: LMStudio Provider (SSE)
1. Implement SSE streaming parser for "data:" lines
2. Handle partial events that span multiple chunks
3. Process "[DONE]" signal correctly
4. Test with SSE-specific edge cases

### Phase 3: Performance Validation
1. Create memory usage benchmarks
2. Test with progressively larger responses
3. Validate no memory regression
4. Document performance improvements

## Files to Modify
- `src/provider/local/ollama.rs` - Replace buffered text processing
- `src/provider/local/lmstudio.rs` - Replace buffered SSE processing
- `tests/real_http_streaming_test.rs` - Add large response tests
- `Cargo.toml` - May need additional async I/O dependencies

## Dependencies
- Current streaming implementation (✅ complete)
- `tokio` async I/O capabilities (✅ available)
- Testing infrastructure for large responses

## Risk Assessment
- **Risk Level**: Medium (changes core streaming behavior)
- **Complexity**: High (async streaming, buffer management, error handling)
- **Testing Requirements**: Extensive (memory, performance, regression)

## Performance Impact
- **Memory**: Significant improvement (constant vs. linear with response size)
- **Latency**: Improvement (first chunk available immediately)
- **Throughput**: Should maintain or improve
- **CPU**: Slight increase (more frequent processing calls)

## Success Metrics
1. **Memory Efficiency**:
   - Memory usage independent of response size
   - No memory leaks during extended streaming
   - Reduction in peak memory usage by 80-90% for large responses

2. **Performance**:
   - Time to first chunk improved by 50%+ for large responses
   - Overall throughput maintained or improved
   - No regression in small response performance

3. **Quality**:
   - All existing tests pass
   - New memory usage tests validate optimization
   - Error handling maintains robustness

## Alternative Approaches

### Defer Implementation
- Current approach works for typical use cases
- Optimization could be delayed until real-world usage patterns are established
- Focus on other high-impact improvements first

### Hybrid Approach
- Keep current approach for small responses (< 1MB)
- Use streaming approach only for large responses
- Adds complexity but reduces risk

## Research Requirements
1. **Benchmark Current Usage**: Measure actual memory usage with various response sizes
2. **Real-World Patterns**: Understand typical streaming response sizes in practice
3. **Library Assessment**: Evaluate tokio-util, futures-util for streaming helpers
4. **Error Handling**: Research async streaming error patterns and recovery

## Notes
This optimization addresses a fundamental architectural issue with the current streaming implementation. While the current approach works for typical usage, it's not truly "streaming" in the memory efficiency sense.

The implementation should prioritize correctness over optimization - ensure all existing behavior is preserved before focusing on performance gains.

Consider implementing this optimization after establishing usage patterns in production to better understand the real-world impact and requirements.