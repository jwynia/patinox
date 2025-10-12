# Task: Improve Error Detection Granularity

## Status  
- **Priority**: Medium
- **Complexity**: Medium
- **Effort**: Small-Medium
- **Dependencies**: None

## Context
During code review, overly broad error detection logic was identified in the provider error handling system.

## Problem Statement
In `src/provider/error.rs`, the `is_retriable()` method uses simple string matching for API errors:
```rust
Self::ApiError(msg) => {
    // Some API errors are retriable (5xx status codes)
    msg.contains("5") || msg.contains("timeout") || msg.contains("unavailable")
}
```

This approach is fragile and may incorrectly classify errors.

## Requirements
1. **Structured Error Classification**: Parse HTTP status codes properly
2. **Provider-Specific Logic**: Different providers may have different retry semantics
3. **Error Context**: Preserve more detailed error information
4. **Retry Policies**: Configurable retry behavior per error type
5. **Testing**: Comprehensive test coverage for error classification

## Implementation Approach

### Enhanced Error Types
```rust
#[derive(Error, Debug, Clone)]
pub enum ProviderError {
    /// HTTP-specific error with structured information
    #[error("HTTP error {status_code}: {message}")]
    HttpError {
        status_code: u16,
        message: String,
        headers: Option<HashMap<String, String>>,
    },
    
    /// Provider-specific API error
    #[error("Provider API error: {error_code}")]
    ApiError {
        provider: String,
        error_code: String,
        message: String,
        is_retriable: bool,
    },
    
    // ... other variants
}
```

### Improved Classification Logic
```rust
impl ProviderError {
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::HttpError { status_code, .. } => {
                *status_code >= 500 || *status_code == 429 || *status_code == 408
            },
            Self::ApiError { is_retriable, .. } => *is_retriable,
            // ... specific logic per error type
        }
    }
}
```

## Benefits
1. More accurate retry decisions
2. Better error reporting and debugging
3. Provider-specific error handling
4. Extensible for future error types
5. Reduced false positives in retry logic

## Acceptance Criteria
- [ ] HTTP status codes are parsed and stored properly
- [ ] Error classification is accurate for different scenarios
- [ ] Provider-specific error codes are handled correctly
- [ ] Retry logic is more precise and configurable
- [ ] Comprehensive tests cover edge cases
- [ ] Error messages provide better debugging information

## Files to Modify
- `src/provider/error.rs` - Enhanced error types and classification
- `src/provider/openai.rs` - Use structured error creation
- `tests/provider_integration_test.rs` - Add error classification tests

## Testing Strategy
- Test all HTTP status code ranges
- Mock provider-specific error responses  
- Verify retry vs non-retry classifications
- Test error message formatting and context

## Notes
This change improves reliability by reducing unnecessary retries and providing better error information for debugging. Consider making error classification extensible for future provider additions.

Created: 2025-01-20 (deferred from code review)