# Task 008: Standardize Error Message Formatting

## Overview
**Type**: Code Quality
**Priority**: Low
**Effort**: Trivial
**Created**: 2025-09-16
**Source**: Code Review Recommendation

## Problem Statement
Error messages across the provider system have inconsistent formatting, particularly with capitalization. This creates a less professional user experience and inconsistent API behavior.

**Examples of Inconsistency**:
```rust
"Model name cannot be empty"  // lowercase
"Messages cannot be empty"    // lowercase
"API error: {0}"             // mixed case
"Invalid request: {0}"       // mixed case
```

## Acceptance Criteria

1. **Define Error Message Standards**:
   - Establish consistent capitalization rules
   - Define standard error message patterns
   - Document formatting guidelines

2. **Update Provider Error Messages**:
   - Review all error messages in provider modules
   - Apply consistent formatting
   - Ensure professional tone

3. **Create Guidelines Document**:
   - Document error message formatting standards
   - Provide examples of good/bad messages
   - Include in contribution guidelines

4. **Validation**:
   - All error messages follow consistent format
   - No breaking changes to error semantics
   - Error tests continue to pass

## Proposed Standards

### Capitalization
- **Error messages**: Sentence case (capitalize first word only)
- **Field references**: Use exact field names in backticks when helpful
- **Actions**: Use imperative mood for actionable errors

### Examples
```rust
// CURRENT (inconsistent)
"Model name cannot be empty"
"Invalid request: {0}"

// PROPOSED (consistent)
"Model name cannot be empty"
"Invalid request: {0}"

// OR (alternative approach)
"Model name must not be empty"
"Request validation failed: {0}"
```

## Implementation Tasks

1. **Audit Current Messages**:
   - Scan all `ProviderError` usage
   - Catalog existing error messages
   - Identify inconsistencies

2. **Apply Standards**:
   - Update error messages to follow standards
   - Maintain error semantics (no breaking changes)
   - Update related test assertions

3. **Documentation**:
   - Add error message guidelines to project documentation
   - Include examples in contribution guide

## Files to Review
- `src/provider/error.rs` - Core error definitions
- `src/provider/local/ollama.rs` - Ollama provider errors
- `src/provider/local/lmstudio.rs` - LMStudio provider errors
- `src/provider/anthropic.rs` - Anthropic provider errors
- `src/provider/openai.rs` - OpenAI provider errors
- `src/provider/openrouter.rs` - OpenRouter provider errors
- Related test files - Update error message assertions

## Dependencies
- None (independent formatting task)

## Risk Assessment
- **Risk Level**: Low
- **Breaking Changes**: None (formatting only)
- **Test Impact**: May need to update error message assertions in tests

## Success Metrics
- All error messages follow consistent formatting
- Error message guidelines documented
- No functional changes to error handling
- All tests pass with updated assertions