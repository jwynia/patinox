# ARCH-001: Design Custom Error Types for Better Categorization

## Status
- **Created**: 2025-10-13
- **Priority**: Medium
- **Effort**: Large (60+ minutes)
- **Type**: Architecture / Error Handling

## Context

**Source**: PR #21 Code Review
**Current State**: Using generic error types with string messages
**Suggestion**: Consider custom error types for better error categorization

## Problem Statement

The current error handling strategy uses generic error types:
```rust
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
```

This works but provides limited:
- Error categorization (no types to match on)
- Contextual information (just string messages)
- Recovery strategies (can't discriminate error types)
- Debugging support (stack traces unclear)

## Acceptance Criteria

- [ ] Design error taxonomy for patinox framework
- [ ] Implement custom error types (enum-based)
- [ ] Update all error callsites to use new types
- [ ] Maintain backwards compatibility where possible
- [ ] Add error conversion implementations
- [ ] Document error handling patterns
- [ ] Update examples to use new error types
- [ ] All tests pass with new error system

## Proposed Approach

### Option A: Thiserror-Based Error Enum

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatinoxError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type Result<T> = std::result::Result<T, PatinoxError>;
```

### Option B: Module-Specific Error Types

```rust
// provider/error.rs
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("API key not found")]
    ApiKeyMissing,

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

// Similar for tool::Error, agent::Error, etc.
```

### Recommendation

Start with Option A (unified error type) for V2 simplicity. Can add module-specific types later if needed.

## Design Questions to Answer

1. **Error Hierarchy**: Flat enum vs. nested error types?
2. **Recovery**: Which errors should be recoverable? How?
3. **Context**: How much context should errors carry?
4. **Serialization**: Should errors be serializable (for logging, APIs)?
5. **User vs. Developer**: Separate error types for different audiences?

## Files to Modify

**Core:**
- `src/lib.rs` - Update Result type
- `src/error.rs` (new) - Define error types

**Modules:**
- `src/agent.rs` - Update error handling
- `src/provider/*.rs` - Update provider errors
- `src/tool.rs` - Update tool errors
- `src/cli.rs` - Update CLI error handling

**Examples:**
- `examples/file_processor.rs` - Update to new error types
- `examples/doc_generator.rs` - Update to new error types

**Tests:**
- Update all error handling tests
- Add error type matching tests

## Testing Plan

1. **Compile Check**: Ensure all error conversions work
2. **Existing Tests**: All current tests should pass
3. **New Tests**: Add tests for error type matching
4. **Error Messages**: Verify user-facing messages are clear
5. **Examples**: Run both agents to verify error handling

## Why Deferred (Not Applied Immediately)

1. **Large Effort**: Requires design decisions + multiple file changes
2. **System-Wide Impact**: Touches most modules in the codebase
3. **Architectural Decision**: Needs broader discussion about error strategy
4. **No Current Pain**: V2 agents work fine with current generic errors
5. **V2 Philosophy**: Don't add sophistication until pain is felt
6. **Timing**: Better to do after Week 3 plugins stabilize

## Dependencies

- None (but should be done before major v1.0 release)
- Consider: Would plugins need specific error types?

## Related Decisions

- See: `context-network/decisions/` for any error handling ADRs
- Consider: Error handling patterns in V1 archive
- Future: Error recovery strategies plugin?

## Research Needed

1. Review V1 archive error handling patterns
2. Survey error handling in similar Rust projects (tokio, actix, etc.)
3. Identify most common error scenarios in V2 agents
4. Determine error recovery strategies needed
5. Design error context propagation strategy

## Success Metrics

- Clear error categorization enables specific error handling
- Error messages are more helpful for debugging
- Error recovery strategies are possible (match on error types)
- Test coverage for error scenarios improves
- User-facing errors are actionable

## Notes

**Current Assessment**: The generic error approach is working fine for V2 minimal phase. This is a "nice to have" rather than "must have".

**V2 Principle Alignment**: This recommendation goes against V2's minimal-first approach. We should wait until error categorization becomes a real pain point (e.g., need to handle provider errors differently from tool errors).

**Recommendation**: DEFER until Week 4+ or when concrete need arises.

## Alternative: Do Nothing

**Valid Option**: Keep generic errors for V2 minimal phase. The current approach:
- ✅ Works fine for simple agents
- ✅ Low cognitive overhead
- ✅ Easy to understand
- ✅ Doesn't block any features

**Trade-off**: Accept less sophisticated error handling in exchange for faster iteration and simpler codebase.

This aligns with V2 "minimal first, sophistication when validated" philosophy.
