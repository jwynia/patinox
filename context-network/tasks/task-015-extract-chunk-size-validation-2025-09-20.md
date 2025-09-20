# Task-015: Extract Chunk Size Validation Function

**Created**: 2025-09-20
**Priority**: High
**Effort**: Small (15-30 minutes)
**Type**: Security / Code Quality

## Context

Code review identified identical chunk size validation logic repeated 4 times across both Ollama and LMStudio providers. This validation is critical for preventing memory exhaustion attacks.

## Original Recommendation

> **Code Duplication in Chunk Size Validation** (Multiple locations)
> **Issue**: Identical chunk size validation logic repeated 4 times

## Problem

The chunk size validation logic is duplicated across multiple locations:

```rust
// DUPLICATED - Appears 4 times across both providers
if content.len() > MAX_CHUNK_SIZE {
    return Err(ProviderError::ApiError(format!(
        "Chunk size ({} chars) exceeds limit ({} chars)",
        content.len(),
        MAX_CHUNK_SIZE
    )));
}
```

## Solution

Extract validation into a shared utility function:

```rust
/// Validates that content doesn't exceed maximum chunk size to prevent memory exhaustion
fn validate_chunk_size(content: &str, max_size: usize) -> Result<(), ProviderError> {
    if content.len() > max_size {
        Err(ProviderError::ApiError(format!(
            "Chunk size ({} chars) exceeds limit ({} chars)",
            content.len(),
            max_size
        )))
    } else {
        Ok(())
    }
}
```

## Acceptance Criteria

- [ ] Create shared validation function (decide location: shared module or per-provider)
- [ ] Replace all 4 instances of duplicate validation logic
- [ ] Maintain identical error messages and behavior
- [ ] All existing tests continue to pass
- [ ] Reduce code duplication by 20+ lines
- [ ] Add unit tests for the validation function

## Implementation Options

### Option A: Shared Validation Module
Create `src/provider/local/validation.rs` with shared utilities

### Option B: Per-Provider Helper Methods
Add private validation methods to each provider

**Recommendation**: Option A for better reusability and centralized security logic

## Implementation Notes

### Files to Modify
- `src/provider/local/ollama.rs` - Replace 2 instances with validation call
- `src/provider/local/lmstudio.rs` - Replace 2 instances with validation call
- `src/provider/local/validation.rs` - New shared validation module
- `src/provider/local/mod.rs` - Export validation module

### Testing
- Add unit tests for `validate_chunk_size()` function
- Test valid sizes, boundary conditions, and oversized content
- Run all existing streaming tests to ensure no regression

### Dependencies
- None - this is an isolated refactoring

## Why Deferred

**Effort**: Small but requires deciding on shared module structure
**Risk**: Medium (touches security-critical validation logic)
**Dependencies**: Local (affects both providers)
**Rationale**: While straightforward, this is security-critical code and needs careful consideration of module organization and thorough testing.

## Priority Justification

High priority because:
- Eliminates duplication in security-critical code
- Creates reusable validation utilities
- Improves maintainability of memory protection
- Foundation for other validation utilities

## Security Implications

This validation prevents memory exhaustion attacks by limiting individual chunk sizes. Centralizing this logic:
- Ensures consistent security policy across providers
- Makes security updates easier to apply
- Reduces risk of inconsistent validation

## Related Items

- Follow-up to streaming memory optimization implementation
- Security hardening for local providers
- Foundation for additional validation utilities