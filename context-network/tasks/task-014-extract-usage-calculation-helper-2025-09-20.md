# Task-014: Extract Usage Calculation Helper Method

**Created**: 2025-09-20
**Priority**: High
**Effort**: Small (15-30 minutes)
**Type**: Code Quality / Refactoring

## Context

Code review identified duplicate usage calculation logic in the Ollama provider's streaming implementation. The same 15+ lines of usage calculation code appear in both the main streaming loop and the buffer processing section.

## Original Recommendation

> **Code Duplication in Usage Calculation Logic** (Lines 448-457, 493-502 in ollama.rs)
> **Issue**: Identical usage calculation logic duplicated between main loop and buffer processing

## Problem

The usage calculation logic for converting Ollama response data to `Usage` objects is duplicated:

```rust
// DUPLICATED - Same logic appears twice
let usage = if ollama_response.prompt_eval_count.is_some()
    || ollama_response.eval_count.is_some()
{
    Some(crate::provider::types::Usage {
        prompt_tokens: ollama_response.prompt_eval_count.unwrap_or(0) as usize,
        completion_tokens: ollama_response.eval_count.unwrap_or(0) as usize,
        total_tokens: (ollama_response.prompt_eval_count.unwrap_or(0)
            + ollama_response.eval_count.unwrap_or(0))
            as usize,
    })
} else {
    None
};
```

## Solution

Extract the logic into a private helper method on `OllamaProvider`:

```rust
impl OllamaProvider {
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

## Acceptance Criteria

- [ ] Create private `create_usage_from_response()` method on OllamaProvider
- [ ] Replace both instances of duplicated logic with helper method calls
- [ ] Maintain identical behavior and public API
- [ ] All existing provider and streaming tests continue to pass
- [ ] Reduce code duplication by 15+ lines

## Implementation Notes

### Files to Modify
- `src/provider/local/ollama.rs` - Extract helper and replace usage

### Testing
- Run existing streaming tests to ensure no behavioral changes
- Specifically verify `test_ollama_streaming_memory_efficiency_large_response`

### Dependencies
- None - this is an isolated refactoring

## Why Deferred

**Effort**: Small but requires careful testing
**Risk**: Medium (touches core streaming logic)
**Dependencies**: Local (requires understanding of streaming implementation)
**Rationale**: While straightforward, this touches the core streaming logic and needs verification that behavior is exactly preserved.

## Priority Justification

High priority because:
- Eliminates significant code duplication
- Improves maintainability of core functionality
- Low complexity once started
- No external dependencies

## Related Items

- Follow-up to streaming memory optimization implementation
- Part of code quality improvements from recent code review