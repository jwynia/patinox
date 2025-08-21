# Task: Implement HTTP Mocking for Anthropic Provider Tests

## Status
- **Priority**: High
- **Complexity**: Large
- **Effort**: Large (1-2 days)
- **Dependencies**: Testing strategy decision

## Context
The current Anthropic provider tests directly test internal methods (`create_headers`, `convert_to_anthropic_format`, `parse_anthropic_response`) rather than testing behavior through the public interface. This couples tests to implementation details and makes refactoring difficult.

## Problem Statement
From test quality review:
1. Tests in `tests/anthropic_provider_test.rs` lines 293-306, 372-385, 413-424 test internal methods
2. Placeholder integration tests with `todo!()` at lines 440, 447 are ignored
3. Some tests have conditional assertions that may pass without verifying expected behavior
4. Missing proper HTTP-level testing that verifies request/response handling

## Requirements
1. **HTTP Mocking Framework**: Choose and implement `mockito`, `wiremock`, or similar
2. **Behavior-Focused Testing**: Test through the `ModelProvider` trait interface
3. **Request Verification**: Verify correct HTTP requests are sent to Anthropic API
4. **Response Testing**: Test handling of various Anthropic API responses
5. **Error Scenario Testing**: Test specific HTTP error conditions (401, 429, 500, etc.)
6. **Maintain Test Coverage**: Ensure no loss of test coverage during transition

## Implementation Plan

### Phase 1: Research and Choose Mocking Framework
1. Evaluate options:
   - `mockito` - Simple HTTP mocking for Rust
   - `wiremock` - More comprehensive HTTP service mocking
   - `httpmock` - Alternative HTTP mocking library
2. Consider integration with existing test infrastructure
3. Document decision rationale

### Phase 2: Implement Base HTTP Mocking Infrastructure
1. Add chosen mocking dependency to `Cargo.toml`
2. Create test utilities for common mock scenarios
3. Set up mock server lifecycle management in tests

### Phase 3: Replace Internal Method Tests
1. **Headers Testing** (lines 293-306):
   - Mock HTTP server that verifies headers are sent correctly
   - Test authentication header presence and format
   - Test API version header
2. **Request Serialization** (lines 372-385):
   - Mock server that captures and validates request body
   - Test that requests match Anthropic API format
3. **Response Parsing** (lines 413-424):
   - Mock server with controlled responses
   - Test various response formats and edge cases

### Phase 4: Implement Comprehensive HTTP Scenarios
1. **Success Scenarios**:
   - Valid completion requests with different parameters
   - Various model types and capabilities
2. **Error Scenarios**:
   - 401 Authentication errors
   - 429 Rate limiting with retry-after
   - 400 Bad request with various error messages
   - 500 Server errors
   - Network timeouts
   - Malformed JSON responses

### Phase 5: Remove Internal Method Exposure
1. Review whether internal methods need to be public
2. Make methods private if only needed for testing
3. Ensure all behavior is testable through public interface

## Acceptance Criteria
- [ ] HTTP mocking framework integrated and working
- [ ] All internal method tests replaced with behavior tests
- [ ] Request format verification through HTTP mocks
- [ ] Response handling tested with controlled mock responses
- [ ] Error scenarios comprehensively tested
- [ ] Test coverage maintained or improved (>85%)
- [ ] All tests isolated and can run in any order
- [ ] No ignored/placeholder tests remaining
- [ ] Test execution time reasonable (<30s for full suite)

## Files to Modify
- `tests/anthropic_provider_test.rs` - Main test refactoring
- `Cargo.toml` - Add mocking dependencies  
- Consider creating `tests/common/mod.rs` for shared test utilities
- Update `src/provider/anthropic.rs` method visibility if needed

## Testing Strategy
1. **Mock Server Lifecycle**: Start/stop for each test
2. **Request Verification**: Capture and validate all outgoing requests
3. **Response Control**: Return specific responses for different scenarios
4. **Error Injection**: Simulate various failure modes
5. **Performance Testing**: Include timeout and latency simulation

## Risk Assessment
**Medium Risk**: Large refactoring of test suite. Risk of losing test coverage or introducing flaky tests. Requires careful validation that all scenarios are still covered.

## Benefits
1. **Better Isolation**: Tests won't break when internal implementation changes
2. **Behavior Focus**: Tests verify the contract, not implementation
3. **Realistic Testing**: Tests actual HTTP interactions
4. **Error Coverage**: Can test all HTTP error scenarios reliably
5. **Maintainability**: Easier to maintain and extend tests
6. **Documentation**: Tests serve as examples of actual API usage

## Success Metrics
- All tests pass with HTTP mocking
- Test coverage >= 85%
- No internal methods directly tested
- All error scenarios covered
- Test execution time < 30 seconds
- No flaky tests observed

## References
- Current Anthropic provider tests for behavior to preserve
- OpenAI provider tests for patterns to follow
- Anthropic API documentation for request/response formats
- Rust HTTP mocking library documentation

Created: 2025-08-21 (from test quality review)