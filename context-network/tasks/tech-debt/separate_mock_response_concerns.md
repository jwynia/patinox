# Task: Separate Mock Response Field Concerns

## Status
- **Priority**: Low
- **Complexity**: Medium
- **Effort**: Medium (45-60 minutes)
- **Risk**: Medium

## Context

**Original Code Review Recommendation**: "Overlapping Field Usage - `error_message` field is used for both error messages and success response bodies, which is confusing."

**Current Issue**: The `MockHttpBuilder` and `MockHttpResponse` use the `error_message` field to store both actual error messages and successful response bodies, creating conceptual confusion.

## Problem Statement

**Confusing Field Usage**:
```rust
// Used for error messages
self.error_message = Some("Service Temporarily Unavailable".to_string());

// Also used for success response bodies
self.error_message = Some(response_body); // Contains JSON success data
```

This makes the code harder to understand and maintain because the same field serves different purposes depending on the HTTP status code.

## Requirements

1. **Clear Separation**: Separate fields for error messages vs response bodies
2. **Backward Compatibility**: Maintain existing public API behavior
3. **Intuitive Design**: Field names should match their purpose
4. **No Regressions**: All existing tests should continue working

## Implementation Approach

### Current Structure
```rust
pub struct MockHttpBuilder {
    status_code: Option<u16>,
    endpoint: Option<String>,
    error_message: Option<String>,  // Confusing: used for both errors and success bodies
    retry_after: Option<Duration>,
}
```

### Proposed Structure
```rust
pub struct MockHttpBuilder {
    status_code: Option<u16>,
    endpoint: Option<String>,
    response_body: Option<String>,   // For all response content
    retry_after: Option<Duration>,
}

pub struct MockHttpResponse {
    status_code: u16,
    endpoint: String,
    response_body: String,          // Contains the actual response content
    retry_after: Option<Duration>,
}

impl MockHttpResponse {
    pub fn response_body(&self) -> &str {
        &self.response_body
    }

    // For backward compatibility
    pub fn error_message(&self) -> &str {
        &self.response_body  // Delegate to response_body
    }
}
```

## Acceptance Criteria

- [ ] Single `response_body` field used for all response content
- [ ] Backward compatibility maintained via `error_message()` method delegation
- [ ] Code is more intuitive and easier to understand
- [ ] All existing tests pass without modification
- [ ] Documentation updated to reflect the clearer design

## Implementation Steps

1. **Phase 1**: Update internal fields
   - Change `MockHttpBuilder` to use `response_body` instead of `error_message`
   - Update all internal method implementations

2. **Phase 2**: Update `MockHttpResponse`
   - Change internal field to `response_body`
   - Keep `error_message()` method for backward compatibility
   - Update `build()` method accordingly

3. **Phase 3**: Validation
   - Run all tests to ensure no regressions
   - Verify API behavior is unchanged
   - Update internal documentation

## Testing Strategy

- All existing tests should pass without modification
- Add test that verifies both `response_body()` and `error_message()` return same content
- Test various scenarios (success responses, error responses, empty responses)

## Migration Path

```rust
// Internal change - users won't see this
MockHttpBuilder {
    endpoint: self.endpoint,
    status_code: self.status_code,
    response_body: Some(response_data),  // Was: error_message
    retry_after: self.retry_after,
}

// Public API remains the same
impl MockHttpResponse {
    pub fn error_message(&self) -> &str {
        &self.response_body  // Backward compatibility
    }

    pub fn response_body(&self) -> &str {
        &self.response_body  // Clearer semantics
    }
}
```

## Notes

This is primarily a semantic improvement. The functionality doesn't change, but the code becomes more self-documenting and easier to maintain.

**Why Medium Risk**: Changes internal structure of widely-used testing utility. Requires careful validation to ensure no behavioral changes.

**Dependencies**: Could be combined with builder pattern standardization task
**Related Tasks**: `standardize_mock_builder_pattern.md`

Created: 2025-09-15 (Code review follow-up)