# Task 011: Extract Shared Test Utilities

## Overview
**Type**: Test Quality/Refactoring
**Priority**: Low
**Effort**: Small
**Created**: 2025-09-16
**Source**: Test Review Recommendation

## Problem Statement
Multiple test files contain duplicate test setup code, particularly for creating test requests and common validation patterns. This duplication makes maintenance harder and increases the risk of inconsistencies across test suites.

**Examples of Duplication**:
```rust
// Repeated in multiple test modules
fn create_test_request() -> CompletionRequest {
    CompletionRequest {
        model: ModelId::new("test-model"),
        messages: vec!["Test message".to_string()],
        temperature: Some(0.7),
        max_tokens: Some(100),
        tools: None,
    }
}
```

**Issues**:
- Code duplication across test files
- Inconsistent test data between test suites
- Maintenance burden when changing test structures
- Potential for test divergence over time

## Current State Analysis

The project already has:
- `/tests/utils/mod.rs` - Existing test utilities (367 lines)
- Test builder patterns in some files
- Mock utilities for HTTP testing

**Missing**:
- Standardized completion request builders
- Common assertion helpers
- Shared test data constants
- Provider-agnostic test utilities

## Acceptance Criteria

1. **Extract Common Request Builders**:
   - Centralize completion request creation
   - Provide builder pattern for test customization
   - Support different request types (completion, embedding, etc.)

2. **Create Shared Assertion Helpers**:
   - Common validation functions for responses
   - Error type checking utilities
   - Stream validation helpers

3. **Standardize Test Data**:
   - Common test constants (model names, messages, etc.)
   - Realistic test data that reflects actual usage
   - Configurable test scenarios

4. **Maintain Backward Compatibility**:
   - Existing tests continue to work
   - Optional adoption of new utilities
   - Gradual migration path

## Implementation Plan

### Phase 1: Enhance Existing Test Utilities

Extend `/tests/utils/mod.rs` with:

```rust
/// Builder for creating test completion requests
pub struct CompletionRequestBuilder {
    model: String,
    messages: Vec<String>,
    temperature: Option<f32>,
    max_tokens: Option<usize>,
    tools: Option<Vec<Tool>>,
}

impl CompletionRequestBuilder {
    pub fn new() -> Self {
        Self {
            model: "test-model".to_string(),
            messages: vec!["Test message".to_string()],
            temperature: Some(0.7),
            max_tokens: Some(100),
            tools: None,
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.messages.push(message.to_string());
        self
    }

    pub fn with_messages(mut self, messages: Vec<&str>) -> Self {
        self.messages = messages.into_iter().map(String::from).collect();
        self
    }

    pub fn empty_model(mut self) -> Self {
        self.model = String::new();
        self
    }

    pub fn empty_messages(mut self) -> Self {
        self.messages = vec![];
        self
    }

    pub fn build(self) -> CompletionRequest {
        CompletionRequest {
            model: ModelId::new(self.model),
            messages: self.messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            tools: self.tools,
        }
    }
}
```

### Phase 2: Add Common Assertion Helpers

```rust
/// Assertion helpers for test validation
pub mod assertions {
    use super::*;

    pub fn assert_valid_completion_response(
        response: &CompletionResponse,
        request: &CompletionRequest,
    ) {
        assert!(!response.content.is_empty(), "Response content should not be empty");
        assert_eq!(response.model, request.model, "Response model should match request");
        assert!(!response.finish_reason.is_empty(), "Finish reason should be present");
    }

    pub fn assert_validation_error(result: &Result<T, ProviderError>, expected_msg: &str) {
        match result {
            Err(ProviderError::InvalidRequest(msg)) => {
                assert!(msg.contains(expected_msg),
                       "Error message '{}' should contain '{}'", msg, expected_msg);
            }
            other => panic!("Expected InvalidRequest error, got: {:?}", other),
        }
    }

    pub fn assert_network_error(result: &Result<T, ProviderError>) {
        match result {
            Err(ProviderError::NetworkError(_)) => {
                // Expected
            }
            other => panic!("Expected NetworkError, got: {:?}", other),
        }
    }
}
```

### Phase 3: Create Test Data Constants

```rust
/// Common test data constants
pub mod test_data {
    pub const DEFAULT_MODEL: &str = "test-model";
    pub const VALID_MESSAGE: &str = "This is a valid test message";
    pub const LONG_MESSAGE: &str = "a".repeat(1000);
    pub const SHORT_MESSAGE: &str = "hi";
    pub const EMPTY_MODEL: &str = "";

    pub const OLLAMA_MODELS: &[&str] = &["llama3.2", "mistral", "codellama"];
    pub const LMSTUDIO_MODELS: &[&str] = &["gpt-3.5-turbo", "llama-2-7b"];

    pub const DEFAULT_TEMPERATURE: f32 = 0.7;
    pub const DEFAULT_MAX_TOKENS: usize = 100;
}
```

### Phase 4: Update Existing Tests

Gradually update test files to use shared utilities:

```rust
// BEFORE
fn create_test_request() -> CompletionRequest {
    CompletionRequest {
        model: ModelId::new("test-model"),
        messages: vec!["Test message".to_string()],
        temperature: Some(0.7),
        max_tokens: Some(100),
        tools: None,
    }
}

// AFTER
use utils::{CompletionRequestBuilder, test_data};

fn create_test_request() -> CompletionRequest {
    CompletionRequestBuilder::new().build()
}

fn create_empty_model_request() -> CompletionRequest {
    CompletionRequestBuilder::new()
        .empty_model()
        .build()
}
```

## Files to Modify
- `/tests/utils/mod.rs` - Add new utility functions and builders
- `/tests/local_provider_streaming_test.rs` - Use shared utilities
- `/tests/lmstudio_provider_mock_tests.rs` - Use shared utilities
- Other test files - Gradual adoption

## Migration Strategy

1. **Phase 1**: Add utilities without changing existing tests
2. **Phase 2**: Update one test file at a time
3. **Phase 3**: Remove duplicate code after migration
4. **Phase 4**: Document new patterns for future tests

## Success Metrics
- Reduced code duplication across test files
- Consistent test data and assertions
- Easier test maintenance and updates
- Improved test readability and consistency
- Documentation of standard test patterns

## Dependencies
- None (test utility enhancement)

## Risk Assessment
- **Risk Level**: Low (test utilities only)
- **Breaking Changes**: None (backward compatible)
- **Benefit**: Reduced maintenance burden and improved consistency

## Future Enhancements
- Provider-specific test utilities
- Mock server helpers
- Performance testing utilities
- Integration test patterns