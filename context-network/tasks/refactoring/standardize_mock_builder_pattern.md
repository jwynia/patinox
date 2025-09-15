# Task: Standardize MockHttpBuilder Pattern Consistency

## Status
- **Priority**: Low
- **Complexity**: Medium
- **Effort**: Medium (45-60 minutes)
- **Risk**: Medium

## Context

**Original Code Review Recommendation**: "Inconsistent Builder Pattern - `with_models_response` and `with_completion_response` methods don't follow the same pattern as other builder methods."

**Current Issue**: The `MockHttpBuilder` has inconsistent patterns between methods:
- Most methods follow `mut self` pattern and return self
- `with_models_response` and `with_completion_response` create new struct instances

This inconsistency makes the API harder to understand and maintain.

## Problem Statement

**Inconsistent Pattern**:
```rust
// Consistent pattern (most methods)
pub fn with_endpoint(mut self, endpoint: &str) -> Self {
    self.endpoint = Some(endpoint.to_string());
    self
}

// Inconsistent pattern (2 methods)
pub fn with_models_response(self, models: &[&str]) -> Self {
    // Creates new MockHttpBuilder instance instead of modifying self
    MockHttpBuilder {
        endpoint: self.endpoint,
        status_code: self.status_code,
        error_message: Some(response_body),
        retry_after: self.retry_after,
    }
}
```

## Requirements

1. **Consistent API**: All builder methods should follow the same `mut self` pattern
2. **Backward Compatibility**: Maintain the same public interface
3. **Clear Separation**: Address the underlying issue of field overlap (error_message used for both errors and success responses)
4. **Maintainability**: Make the code easier to understand and extend

## Implementation Approach

### Option 1: Fix Pattern Only (Quick)
```rust
pub fn with_models_response(mut self, models: &[&str]) -> Self {
    let response_body = create_models_json(models);
    self.error_message = Some(response_body);
    self
}
```

### Option 2: Improve Field Separation (Better)
```rust
pub struct MockHttpBuilder {
    status_code: Option<u16>,
    endpoint: Option<String>,
    response_body: Option<String>,  // Separate from error handling
    retry_after: Option<Duration>,
}
```

## Acceptance Criteria

- [ ] All builder methods follow consistent `mut self` pattern
- [ ] Existing tests continue to pass without modification
- [ ] Internal field usage is clearer and less confusing
- [ ] No breaking changes to public API
- [ ] Code is easier to understand and maintain

## Implementation Steps

1. **Phase 1**: Fix immediate pattern inconsistency
   - Update `with_models_response` to use `mut self`
   - Update `with_completion_response` to use `mut self`
   - Run tests to ensure no regressions

2. **Phase 2** (Optional): Improve field separation
   - Consider separating response_body from error_message
   - Evaluate impact on existing API
   - Update if beneficial without breaking changes

## Testing Strategy

- Existing tests should continue to pass
- Add tests that chain all builder methods to verify consistency
- Verify that the fluent interface works smoothly

## Notes

This is primarily a code quality improvement. The inconsistency doesn't break functionality but makes the API less intuitive.

**Risk Assessment**: Medium risk because it touches the builder pattern that's used throughout tests. Changes need careful validation.

**Dependencies**: Should be done after JSON construction safety improvements
**Related Tasks**: Field separation could be done as separate task if complexity warrants

Created: 2025-09-15 (Code review follow-up)