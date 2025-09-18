# Task 006: Extract Streaming Validation Logic

## Overview
**Type**: Refactoring
**Priority**: Medium
**Effort**: Small
**Created**: 2025-09-16
**Source**: Code Review Recommendation

## Problem Statement
Both Ollama and LMStudio providers have nearly identical validation logic in their `stream_completion` methods, leading to code duplication that makes maintenance harder.

```rust
// Duplicate validation in both ollama.rs and lmstudio.rs
if request.model.name().is_empty() {
    return Err(ProviderError::InvalidRequest("Model name cannot be empty".to_string()));
}
if request.messages.is_empty() {
    return Err(ProviderError::InvalidRequest("Messages cannot be empty".to_string()));
}
```

## Acceptance Criteria

1. **Extract Common Validation**:
   - Create shared validation utility function for streaming requests
   - Function should validate model name and messages
   - Return appropriate `ProviderError` for validation failures

2. **Update Provider Implementations**:
   - Replace duplicate validation in `ollama.rs`
   - Replace duplicate validation in `lmstudio.rs`
   - Maintain identical error messages and behavior

3. **Test Coverage**:
   - Ensure existing validation tests continue to pass
   - Add unit tests for the extracted validation function
   - Verify error messages remain consistent

4. **Documentation**:
   - Document the shared validation utility
   - Update provider documentation if needed

## Implementation Notes

**Suggested Location**: `src/provider/local/validation.rs` or `src/provider/validation.rs`

**Function Signature**:
```rust
pub(crate) fn validate_streaming_request(request: &CompletionRequest) -> Result<(), ProviderError>
```

## Files to Modify
- `src/provider/local/ollama.rs` - Remove duplicate validation
- `src/provider/local/lmstudio.rs` - Remove duplicate validation
- `src/provider/local/validation.rs` - Create new validation module
- `src/provider/local/mod.rs` - Export validation module if needed

## Dependencies
- None (independent refactoring)

## Risk Assessment
- **Risk Level**: Low
- **Backward Compatibility**: Maintained (same validation logic)
- **Test Impact**: None (behavior unchanged)

## Success Metrics
- Zero code duplication in validation logic
- All existing tests continue to pass
- Consistent error messages maintained
- New shared validation function has unit tests