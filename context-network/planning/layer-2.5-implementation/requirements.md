# Requirements - Layer 2.5 Lifecycle Hook Architecture

## Purpose
Detailed functional and non-functional requirements for lifecycle hook implementation.

## Classification
- **Domain:** Requirements Specification
- **Stability:** Static
- **Abstraction:** Detailed
- **Confidence:** High

---

## Functional Requirements

### FR-1: AgentLifecycle Trait Definition

**Priority**: CRITICAL
**Category**: Core Architecture

**Requirements**:
- FR-1.1: Define `AgentLifecycle` trait with 6 hook methods
- FR-1.2: All hook methods must have default implementations
- FR-1.3: Trait must be `Send + Sync` for async/multithreaded use
- FR-1.4: Use `async_trait` for async methods

**Hook Signatures**:

```rust
#[async_trait]
pub trait AgentLifecycle: Send + Sync {
    // FR-1.1.1: before_agent hook
    async fn before_agent(&self, input: &str) -> Result<String> {
        Ok(input.to_string())
    }

    // FR-1.1.2: before_model hook
    async fn before_model(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
        Ok(messages)
    }

    // FR-1.1.3: wrap_model_call hook
    async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = ProviderResult<ProviderResponse>> + Send,
    {
        f().await
    }

    // FR-1.1.4: after_model hook
    async fn after_model(&self, response: &ProviderResponse) -> Result<HookAction> {
        Ok(HookAction::Continue)
    }

    // FR-1.1.5: wrap_tool_call hook
    async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = ToolResult> + Send,
    {
        f().await
    }

    // FR-1.1.6: after_agent hook
    async fn after_agent(&self, result: &str) -> Result<String> {
        Ok(result.to_string())
    }
}
```

**Acceptance Criteria**:
- [ ] Trait compiles with all 6 methods
- [ ] Default implementations return passthrough values
- [ ] Trait bounds allow use in Agent
- [ ] Rustdoc comments explain each hook's purpose

---

### FR-2: HookAction Enum

**Priority**: CRITICAL
**Category**: Core Types

**Requirements**:
- FR-2.1: Define `HookAction` enum for `after_model` return
- FR-2.2: Support Continue, Approve, Reject, Modify actions
- FR-2.3: Enum must be `Clone` and `Debug`

**Type Definition**:

```rust
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Proceed normally
    Continue,

    /// Explicit approval (for HITL workflows)
    Approve,

    /// Reject with reason
    Reject(String),

    /// Modify the response
    Modify(ProviderResponse),
}
```

**Acceptance Criteria**:
- [ ] Enum compiles with all variants
- [ ] Can be used in hook return types
- [ ] Derives work correctly
- [ ] Rustdoc explains each variant

---

### FR-3: Agent Hook Registration

**Priority**: CRITICAL
**Category**: Builder API

**Requirements**:
- FR-3.1: Add `lifecycle` field to `Agent` struct
- FR-3.2: Implement `.with_lifecycle()` builder method
- FR-3.3: Support multiple hooks (hook chain)
- FR-3.4: Maintain existing builder pattern

**Agent Struct Update**:

```rust
pub struct Agent {
    pub(crate) config: AgentConfig,
    pub(crate) tools: HashMap<String, Arc<dyn Tool>>,
    provider: Option<Box<dyn LLMProvider>>,
    lifecycle: Vec<Arc<dyn AgentLifecycle>>,  // NEW
}
```

**Builder Method**:

```rust
impl Agent {
    pub fn with_lifecycle(mut self, hook: impl AgentLifecycle + 'static) -> Self {
        self.lifecycle.push(Arc::new(hook));
        self
    }
}
```

**Acceptance Criteria**:
- [ ] Can call `.with_lifecycle(hook)` on agent builder
- [ ] Can chain multiple `.with_lifecycle()` calls
- [ ] Existing builder methods still work
- [ ] Rustdoc shows usage examples

---

### FR-4: Hook Integration in Agent::run()

**Priority**: CRITICAL
**Category**: Core Logic

**Requirements**:
- FR-4.1: Call `before_agent` with input before processing
- FR-4.2: Call `before_model` with messages before LLM call
- FR-4.3: Call `wrap_model_call` around provider.complete()
- FR-4.4: Call `after_model` after LLM responds
- FR-4.5: Call `wrap_tool_call` around tool.execute()
- FR-4.6: Call `after_agent` with result before returning
- FR-4.7: Execute hooks in registration order
- FR-4.8: Fast path when no hooks registered (optimization)

**Integration Points**:

```rust
async fn run(&self, input: String) -> Result<String> {
    // FR-4.1: before_agent
    let mut input = input;
    for hook in &self.lifecycle {
        input = hook.before_agent(&input).await?;
    }

    // Build messages...
    let mut messages = vec![...];

    for iteration in 0..max_iterations {
        // FR-4.2: before_model
        for hook in &self.lifecycle {
            messages = hook.before_model(messages).await?;
        }

        // FR-4.3: wrap_model_call
        let response = if self.lifecycle.is_empty() {
            provider.complete(messages.clone(), tools.clone()).await?
        } else {
            self.call_with_model_hooks(|| {
                provider.complete(messages.clone(), tools.clone())
            }).await?
        };

        // FR-4.4: after_model
        let mut response = response;
        for hook in &self.lifecycle {
            match hook.after_model(&response).await? {
                HookAction::Continue | HookAction::Approve => {}
                HookAction::Reject(reason) => return Err(reason.into()),
                HookAction::Modify(new_resp) => response = new_resp,
            }
        }

        // Tool execution with FR-4.5: wrap_tool_call
        // ...

        if let ProviderResponse::Text(text) = response {
            let mut result = text;

            // FR-4.6: after_agent
            for hook in &self.lifecycle {
                result = hook.after_agent(&result).await?;
            }

            return Ok(result);
        }
    }
}
```

**Acceptance Criteria**:
- [ ] All hooks called at correct points
- [ ] Hooks execute in registration order
- [ ] HookAction variants handled correctly
- [ ] Fast path skips hooks when empty
- [ ] Existing behavior preserved with 0 hooks

---

### FR-5: Helper Methods

**Priority**: HIGH
**Category**: Internal Utilities

**Requirements**:
- FR-5.1: Implement helper for chaining wrap_model_call hooks
- FR-5.2: Implement helper for chaining wrap_tool_call hooks
- FR-5.3: Helpers must handle empty lifecycle vec

**Helper Signatures**:

```rust
impl Agent {
    async fn call_with_model_hooks<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse>
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = ProviderResult<ProviderResponse>> + Send,
    {
        // Chain hooks from outer to inner
        // ...
    }

    async fn call_with_tool_hooks<F, Fut>(&self, name: &str, f: F) -> ToolResult
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = ToolResult> + Send,
    {
        // Chain hooks from outer to inner
        // ...
    }
}
```

**Acceptance Criteria**:
- [ ] Helpers correctly chain multiple hooks
- [ ] Empty lifecycle vec handled efficiently
- [ ] Private visibility (implementation detail)

---

## Non-Functional Requirements

### NFR-1: Performance

**Priority**: CRITICAL
**Category**: Performance

**Requirements**:
- NFR-1.1: Zero overhead when no hooks registered
- NFR-1.2: < 5% overhead with 1 hook
- NFR-1.3: < 10% overhead with 5 hooks
- NFR-1.4: No heap allocations if lifecycle vec empty
- NFR-1.5: Minimal Arc clones for hook references

**Measurement**:
- Benchmark `Agent::run()` with 0, 1, 3, 5 hooks
- Compare latency to baseline (no hooks)
- Profile memory allocations

**Acceptance Criteria**:
- [ ] Benchmarks show < 5% overhead (1 hook)
- [ ] Benchmarks show < 10% overhead (5 hooks)
- [ ] No allocations in zero-hook fast path
- [ ] Profiling confirms expectations

---

### NFR-2: Backward Compatibility

**Priority**: CRITICAL
**Category**: Compatibility

**Requirements**:
- NFR-2.1: All existing tests pass unchanged
- NFR-2.2: Existing agent creation code works identically
- NFR-2.3: No breaking changes to public API
- NFR-2.4: New lifecycle field initialized empty by default

**Validation**:
- Run full test suite without modifications
- Check examples compile and run
- Verify builder pattern unchanged

**Acceptance Criteria**:
- [ ] 16 existing tests pass
- [ ] Examples compile without changes
- [ ] No API breakage detected
- [ ] Agent::new() works as before

---

### NFR-3: Documentation

**Priority**: HIGH
**Category**: Documentation

**Requirements**:
- NFR-3.1: Rustdoc for all public types/traits
- NFR-3.2: Code examples in rustdoc
- NFR-3.3: Architecture guide explaining hooks
- NFR-3.4: Migration guide (if needed)
- NFR-3.5: Example showing hook usage

**Documentation Locations**:
- `src/lifecycle.rs` - Trait and type rustdoc
- `examples/lifecycle_hooks.rs` - Working example
- `context-network/planning/` - Architecture docs

**Acceptance Criteria**:
- [ ] `cargo doc` builds without warnings
- [ ] All public items documented
- [ ] Example compiles and runs
- [ ] Architecture guide complete

---

### NFR-4: Testing

**Priority**: CRITICAL
**Category**: Quality

**Requirements**:
- NFR-4.1: Unit tests for AgentLifecycle default implementations
- NFR-4.2: Integration tests for hook execution order
- NFR-4.3: Tests for HookAction handling (Continue, Reject, Modify)
- NFR-4.4: Regression tests (0 hooks = original behavior)
- NFR-4.5: Tests for hook chaining (multiple hooks)
- NFR-4.6: Edge case tests (empty hooks, errors in hooks)

**Test Coverage**:
- Default implementations return passthroughs
- Hooks execute in registration order
- HookAction::Reject stops execution
- HookAction::Modify changes response
- Multiple hooks chain correctly
- Errors propagate properly

**Acceptance Criteria**:
- [ ] All new code covered by tests
- [ ] Integration tests verify hook behavior
- [ ] Edge cases handled
- [ ] Test suite passes (0 failures)

---

### NFR-5: Code Quality

**Priority**: HIGH
**Category**: Quality

**Requirements**:
- NFR-5.1: `cargo clippy` passes (no warnings)
- NFR-5.2: `cargo fmt` applied (consistent style)
- NFR-5.3: Follow Rust API guidelines
- NFR-5.4: Idiomatic Rust patterns
- NFR-5.5: Clear error messages

**Quality Checks**:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

**Acceptance Criteria**:
- [ ] Clippy clean (0 warnings)
- [ ] Formatting consistent
- [ ] Idiomatic Rust code
- [ ] Clear error messages

---

## Constraints

### Technical Constraints

**TC-1: Async Trait Requirement**
- Must use `async_trait` crate for async trait methods
- Alternative (GATs) not stable yet

**TC-2: Send + Sync Requirement**
- Hooks must be Send + Sync for use in Agent
- Limits closures to thread-safe types

**TC-3: Type Erasure**
- Vec<Arc<dyn AgentLifecycle>> requires trait object
- Can't use generic type parameter (would infect all of Agent)

### Project Constraints

**PC-1: Minimal-First Philosophy**
- No concrete hook implementations
- Infrastructure only, defer usage to Layer 3+

**PC-2: Week 4 Timeline**
- Must complete by October 31, 2025
- 2-3 days implementation + 1 day testing/docs

**PC-3: No External Dependencies**
- Only use existing dependencies (async_trait, tokio)
- No new crates for this feature

### Architecture Constraints

**AC-1: Builder Pattern**
- Must fit existing `.with_X()` builder style
- No breaking changes to Agent API

**AC-2: Zero-Cost Abstraction**
- Must have fast path when no hooks
- Minimal overhead when hooks present

---

## Dependencies

### Internal Dependencies
- `src/agent.rs` - Main integration point
- `src/provider/mod.rs` - Provider types (Message, ProviderResponse)
- `src/tool.rs` - Tool types (ToolResult)
- `crate::Result` - Error type

### External Dependencies
- `async_trait` - For async trait methods (already in Cargo.toml)
- `tokio` - Async runtime (already in Cargo.toml)

### No New Dependencies
- All required crates already present
- No additional dependencies needed

---

## Acceptance Criteria Summary

### Must Have (CRITICAL)
- [ ] AgentLifecycle trait defined with 6 hooks
- [ ] HookAction enum implemented
- [ ] Agent.lifecycle field added
- [ ] .with_lifecycle() builder method works
- [ ] All 6 hooks integrated in Agent::run()
- [ ] Default implementations passthrough
- [ ] All existing tests pass (16/16)
- [ ] < 5% overhead with 1 hook
- [ ] Rustdoc for all public APIs

### Should Have (HIGH)
- [ ] Example demonstrating hook usage
- [ ] Integration tests for hook behavior
- [ ] Benchmarks comparing overhead
- [ ] Architecture guide updated

### Nice to Have (MEDIUM)
- [ ] Multiple example hooks
- [ ] Performance profiling data
- [ ] Comparison to alternatives

---

## Verification Plan

### Phase 1: Code Review
1. Trait definition matches specification
2. Agent integration correct
3. Default implementations appropriate
4. Code follows Rust guidelines

### Phase 2: Testing
1. Run existing test suite (must pass)
2. Run new hook tests (must pass)
3. Run integration tests (must pass)
4. Run benchmarks (meet targets)

### Phase 3: Documentation Review
1. Rustdoc builds cleanly
2. Examples compile and run
3. Architecture guide clear
4. No missing documentation

### Phase 4: Integration Check
1. Build example agents with hooks
2. Verify zero-hook behavior unchanged
3. Test hook chaining
4. Validate error handling

---

## Related Documents

**Architecture**: [decisions/lifecycle-hook-architecture.md](../../decisions/lifecycle-hook-architecture.md)
**Use Cases**: [planning/lifecycle-hook-use-cases.md](../lifecycle-hook-use-cases.md)
**Problem**: [problem-definition.md](./problem-definition.md)

## Metadata
- **Created**: 2025-10-16
- **Type**: Requirements Specification
- **Status**: COMPLETE - Ready for task breakdown
- **Priority**: CRITICAL (Layer 2.5 foundation)
