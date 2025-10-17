# ARCH-001: Create lifecycle.rs with AgentLifecycle trait

**Created**: 2025-10-16
**Status**: ready
**Priority**: Critical
**Size**: Medium
**Effort**: 3-4 hours
**Type**: feature

## Description

Create new file `src/lifecycle.rs` with the `AgentLifecycle` trait defining all 6 hook points with default implementations. This is the foundation task for Layer 2.5 lifecycle hook architecture.

## Context

This task implements the core trait that enables lifecycle hooks throughout the agent execution flow. It's based on the LangChain V1 middleware pattern and validated by external production experience.

**Related Documents**:
- [Architecture Decision](../../decisions/lifecycle-hook-architecture.md)
- [Use Cases Catalog](../../planning/lifecycle-hook-use-cases.md)
- [Task Breakdown](../../planning/layer-2.5-implementation/task-breakdown.md#arch-001-create-lifecyclers-with-agentlifecycle-trait)

## Acceptance Criteria

- [ ] File `src/lifecycle.rs` created
- [ ] `AgentLifecycle` trait defined with `Send + Sync` bounds
- [ ] All 6 hook methods present with correct signatures:
  - [ ] `before_agent`
  - [ ] `before_model`
  - [ ] `wrap_model_call`
  - [ ] `after_model`
  - [ ] `wrap_tool_call`
  - [ ] `after_agent`
- [ ] All methods have default passthrough implementations
- [ ] Uses `#[async_trait]` for async methods
- [ ] Trait compiles without errors
- [ ] Rustdoc comments for trait and each method
- [ ] Exported from `src/lib.rs`

## Implementation Notes

**Files to create/modify**:
- `src/lifecycle.rs` - New file with trait definition
- `src/lib.rs` - Add `pub mod lifecycle;` and re-exports

**Trait Structure**:
```rust
use crate::provider::{Message, ProviderResponse, ProviderResult};
use crate::tool::ToolResult;
use async_trait::async_trait;
use std::future::Future;

/// Agent lifecycle hooks for middleware and intervention points
#[async_trait]
pub trait AgentLifecycle: Send + Sync {
    async fn before_agent(&self, input: &str) -> crate::Result<String> {
        Ok(input.to_string())
    }

    async fn before_model(&self, messages: Vec<Message>) -> crate::Result<Vec<Message>> {
        Ok(messages)
    }

    async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = ProviderResult<ProviderResponse>> + Send,
    {
        f().await
    }

    async fn after_model(&self, response: &ProviderResponse) -> crate::Result<HookAction> {
        Ok(HookAction::Continue)
    }

    async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = ToolResult> + Send,
    {
        f().await
    }

    async fn after_agent(&self, result: &str) -> crate::Result<String> {
        Ok(result.to_string())
    }
}
```

**Approach**:
1. Create `src/lifecycle.rs` file
2. Add imports for types used in signatures
3. Define `AgentLifecycle` trait with `#[async_trait]`
4. Implement each of 6 hook methods with default implementations
5. Add comprehensive rustdoc comments
6. Export from `src/lib.rs`
7. Verify compilation with `cargo check`

**Watch out for**:
- Ensure `async_trait` is in Cargo.toml
- Generic bounds on wrap_* methods must be correct
- Default implementations must not require self fields
- All imports must be valid

## Dependencies

**Blocked by**: None (foundation task)

**Blocks**:
- [ ] ARCH-003 - Add lifecycle field to Agent struct
- [ ] TEST-002 - Unit tests for defaults
- [ ] DOCS-002 - Rustdoc for lifecycle.rs

**Related**: ARCH-002 (HookAction enum) - can be done in parallel

## Testing Strategy

- Unit tests in `src/lifecycle.rs` `#[cfg(test)]` module
- Test default implementations return passthrough values
- Create simple test struct implementing trait
- Verify each hook compiles and executes

## Related Documentation

- **Planning**: [Layer 2.5 Implementation Plan](../../planning/layer-2.5-implementation/README.md)
- **Architecture**: [Lifecycle Hook Architecture Decision](../../decisions/lifecycle-hook-architecture.md)
- **Requirements**: [FR-1: AgentLifecycle Trait](../../planning/layer-2.5-implementation/requirements.md#fr-1-agentlifecycle-trait-definition)

## History

- 2025-10-16: Created task during Layer 2.5 planning session

## Notes

This is the most critical task in Layer 2.5 - everything depends on it. Take time to get the trait signature correct, as changes later will be expensive.

**Estimated Time**: 3-4 hours
**Actual Time**: _To be filled during implementation_
