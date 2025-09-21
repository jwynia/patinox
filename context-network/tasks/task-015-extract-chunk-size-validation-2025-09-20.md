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

- [x] Create shared validation function (decide location: shared module or per-provider)
- [x] Replace all 6 instances of duplicate validation logic (found 6, not 4)
- [x] Maintain identical error messages and behavior
- [x] All existing tests continue to pass
- [x] Reduce code duplication by 20+ lines (eliminated 48 lines total)
- [x] Add unit tests for the validation function

**STATUS: COMPLETED** ✅

## Completion Summary

**Completed**: 2025-09-20
**Commit**: `35f2f21` - feat: extract chunk size validation to shared utility function
**Branch**: `feat/extract-chunk-validation-0920`
**Files Modified**: 4 files changed, 127 insertions(+), 48 deletions(-)

### What Was Implemented

1. **Created** `/tmp/worktrees/patinox-chunk-validation-0920/src/provider/local/validation.rs`
   - Shared `validate_chunk_size()` function with comprehensive documentation
   - Exported `MAX_CHUNK_SIZE` constant (1MB in characters)
   - 7 comprehensive unit tests covering all edge cases

2. **Updated** `ollama.rs`: Replaced 2 instances of duplicate validation logic
   - Added import for validation utilities
   - Removed local `MAX_CHUNK_SIZE` constant
   - Replaced verbose validation blocks with single function calls

3. **Updated** `lmstudio.rs`: Replaced 4 instances of duplicate validation logic
   - Added import for validation utilities
   - Removed `MAX_CHUNK_SIZE` from defaults module
   - Replaced all validation blocks with function calls

4. **Updated** `mod.rs`: Exported validation module for public access

### Impact

- **Code Reduction**: Eliminated 48 lines of duplicate code
- **Security**: Centralized memory exhaustion protection logic
- **Maintainability**: Single source of truth for chunk validation
- **Testing**: All 166 tests pass, ensuring no behavioral changes
- **Consistency**: Identical error messages maintained across providers

### Implementation Notes

- Found 6 instances instead of originally estimated 4
- Used Option A (shared validation module) for better reusability
- Maintained exact error message format for compatibility
- Added comprehensive test coverage including boundary conditions

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