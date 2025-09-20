# Task 012: Extract Usage Calculation Utility

## Overview
**Type**: Refactoring
**Priority**: Medium
**Effort**: Small
**Created**: 2025-09-19
**Source**: Code Review Recommendation

## Problem Statement
Code duplication exists in the Ollama provider where usage calculation logic is repeated in both the regular `complete()` method and the new `stream_completion()` method. This violates DRY principles and creates maintenance overhead.

**Current Duplication**:
```rust
// Appears in both complete() and stream_completion()
let usage = if ollama_response.prompt_eval_count.is_some() || ollama_response.eval_count.is_some() {
    Some(crate::provider::types::Usage {
        prompt_tokens: ollama_response.prompt_eval_count.unwrap_or(0) as usize,
        completion_tokens: ollama_response.eval_count.unwrap_or(0) as usize,
        total_tokens: (ollama_response.prompt_eval_count.unwrap_or(0)
            + ollama_response.eval_count.unwrap_or(0)) as usize,
    })
} else {
    None
};
```

## Acceptance Criteria

### Implementation Requirements
1. **Extract Helper Method**:
   - Create `create_usage_from_response()` method on `OllamaProvider`
   - Method should take `&OllamaGenerateResponse` and return `Option<Usage>`
   - Make method private (no need to expose publicly)

2. **Update Both Call Sites**:
   - Replace duplicated logic in `complete()` method (around line 313-316)
   - Replace duplicated logic in `stream_completion()` method (around line 411-414)
   - Ensure identical behavior is preserved

3. **Maintain Compatibility**:
   - No changes to public API
   - Identical behavior for both streaming and non-streaming completions
   - All existing tests must continue to pass

### Testing Requirements
1. **Verify No Regression**:
   - All existing provider tests pass unchanged
   - All streaming tests pass unchanged
   - Usage statistics are identical before and after refactoring

2. **Test Coverage**:
   - No additional tests needed (existing coverage validates behavior)
   - Consider adding unit test for the helper method if desired

## Implementation Guide

### Step 1: Create Helper Method
```rust
impl OllamaProvider {
    /// Create usage statistics from Ollama response data
    fn create_usage_from_response(response: &OllamaGenerateResponse) -> Option<Usage> {
        if response.prompt_eval_count.is_some() || response.eval_count.is_some() {
            Some(Usage {
                prompt_tokens: response.prompt_eval_count.unwrap_or(0) as usize,
                completion_tokens: response.eval_count.unwrap_or(0) as usize,
                total_tokens: (response.prompt_eval_count.unwrap_or(0)
                    + response.eval_count.unwrap_or(0)) as usize,
            })
        } else {
            None
        }
    }
}
```

### Step 2: Update complete() Method
Replace the usage calculation with:
```rust
let usage = Self::create_usage_from_response(&response);
```

### Step 3: Update stream_completion() Method
Replace the usage calculation with:
```rust
let usage = Self::create_usage_from_response(&ollama_response);
```

## Files to Modify
- `src/provider/local/ollama.rs` - Extract method and update both call sites

## Dependencies
- Current provider implementations (✅ complete)
- Usage type definitions (✅ complete)

## Risk Assessment
- **Risk Level**: Low (isolated refactoring within single file)
- **Complexity**: Low (simple extraction of existing logic)
- **Testing Requirements**: Regression testing (existing tests validate behavior)

## Success Metrics
- Code duplication reduced by 8-10 lines
- Single source of truth for usage calculation logic
- All existing tests continue to pass
- No change in public API or behavior

## Notes
This refactoring was identified during the real HTTP streaming implementation code review. The duplication was introduced when streaming support was added to maintain consistency with the existing complete() method. Now that both methods are stable, the common logic can be safely extracted.

The helper method should be kept simple and focused solely on the conversion logic. Consider whether similar patterns exist in other providers that could benefit from the same approach.