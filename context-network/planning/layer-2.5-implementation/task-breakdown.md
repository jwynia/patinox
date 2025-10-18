# Task Breakdown - Layer 2.5 Lifecycle Hook Architecture

## Purpose
Comprehensive breakdown of implementation tasks with estimates, dependencies, and acceptance criteria.

## Classification
- **Domain:** Implementation Plan
- **Stability:** Static
- **Abstraction:** Detailed
- **Confidence:** High

---

## Implementation Timeline

**Total Estimated Effort**: 2-3 days implementation + 1 day testing/docs = 3-4 days
**Target Completion**: October 31, 2025 (Week 4)
**Team Size**: 1 developer

---

## Task Overview

| Task ID | Description | Size | Dependencies | Status |
|---------|-------------|------|--------------|--------|
| ARCH-001 | Create lifecycle.rs with AgentLifecycle trait | M | None | READY |
| ARCH-002 | Implement HookAction enum | S | None | READY |
| ARCH-003 | Add lifecycle field to Agent struct | S | ARCH-001 | READY |
| ARCH-004 | Implement .with_lifecycle() builder | S | ARCH-003 | READY |
| ARCH-005 | Integrate before_agent hook | M | ARCH-004 | READY |
| ARCH-006 | Integrate before_model hook | M | ARCH-005 | READY |
| ARCH-007 | Integrate wrap_model_call hook | L | ARCH-006 | READY |
| ARCH-008 | Integrate after_model hook | M | ARCH-007 | READY |
| ARCH-009 | Integrate wrap_tool_call hook | L | ARCH-008 | READY |
| ARCH-010 | Integrate after_agent hook | M | ARCH-009 | READY |
| TEST-002 | Unit tests for AgentLifecycle defaults | M | ARCH-001 | READY |
| TEST-003 | Integration tests for hook execution | L | ARCH-010 | READY |
| TEST-004 | Regression tests (0 hooks behavior) | M | ARCH-010 | READY |
| PERF-001 | Performance benchmarks | M | ARCH-010 | READY |
| DOCS-002 | Rustdoc for lifecycle.rs | S | ARCH-001 | READY |
| DOCS-003 | Example demonstrating hooks | M | ARCH-010 | READY |
| DOCS-004 | Update architecture documentation | S | ARCH-010 | READY |

**Total Tasks**: 17
**Critical Path**: ARCH-001 → ARCH-003 → ARCH-004 → ARCH-005 → ARCH-006 → ARCH-007 → ARCH-008 → ARCH-009 → ARCH-010 → TEST-003

---

## Detailed Task Specifications

### ARCH-001: Create lifecycle.rs with AgentLifecycle trait

**Priority**: CRITICAL
**Size**: Medium (M)
**Estimated Effort**: 3-4 hours
**Type**: feature

#### Description
Create new file `src/lifecycle.rs` with the `AgentLifecycle` trait defining all 6 hook points with default implementations.

#### Acceptance Criteria
- [ ] File `src/lifecycle.rs` created
- [ ] `AgentLifecycle` trait defined with `Send + Sync` bounds
- [ ] All 6 hook methods present:
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

#### Implementation Notes

**File Location**: `src/lifecycle.rs`

**Trait Structure**:
```rust
use crate::provider::{Message, ProviderResponse, ProviderResult};
use crate::tool::ToolResult;
use async_trait::async_trait;
use std::future::Future;

/// Agent lifecycle hooks for middleware and intervention points
#[async_trait]
pub trait AgentLifecycle: Send + Sync {
    /// Called before agent starts processing input
    async fn before_agent(&self, input: &str) -> crate::Result<String> {
        Ok(input.to_string())
    }

    /// Called before sending messages to LLM
    async fn before_model(&self, messages: Vec<Message>) -> crate::Result<Vec<Message>> {
        Ok(messages)
    }

    /// Wraps the model call (for retry, fallback, etc.)
    async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = ProviderResult<ProviderResponse>> + Send,
    {
        f().await
    }

    /// Called after model responds, before tool execution
    async fn after_model(&self, response: &ProviderResponse) -> crate::Result<HookAction> {
        Ok(HookAction::Continue)
    }

    /// Wraps each tool call (for retry, logging, etc.)
    async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = ToolResult> + Send,
    {
        f().await
    }

    /// Called after agent completes execution
    async fn after_agent(&self, result: &str) -> crate::Result<String> {
        Ok(result.to_string())
    }
}
```

**Module Export**: Add to `src/lib.rs`:
```rust
pub mod lifecycle;
pub use lifecycle::{AgentLifecycle, HookAction};
```

#### Watch Out For
- Ensure `async_trait` is in Cargo.toml
- Generic bounds on wrap_* methods must be correct
- Default implementations must not require self fields

#### Dependencies
- **Blocked by**: None (foundation task)
- **Blocks**: ARCH-003, TEST-002, DOCS-002

---

### ARCH-002: Implement HookAction enum

**Priority**: CRITICAL
**Size**: Small (S)
**Estimated Effort**: 1 hour
**Type**: feature

#### Description
Define `HookAction` enum in `src/lifecycle.rs` for `after_model` hook return values.

#### Acceptance Criteria
- [ ] `HookAction` enum defined in `lifecycle.rs`
- [ ] Four variants: Continue, Approve, Reject, Modify
- [ ] Derives: Debug, Clone
- [ ] Rustdoc for enum and variants
- [ ] Compiles without errors

#### Implementation Notes

**Type Definition**:
```rust
/// Action to take after a lifecycle hook
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Continue processing normally
    Continue,

    /// Explicit approval (for HITL workflows)
    Approve,

    /// Reject with error message
    Reject(String),

    /// Modify the provider response
    Modify(ProviderResponse),
}
```

**Usage Context**:
- Returned by `after_model` hook
- Agent::run() matches on variants to decide next step

#### Dependencies
- **Blocked by**: None (can be done in parallel with ARCH-001)
- **Blocks**: ARCH-008 (after_model integration)

---

### ARCH-003: Add lifecycle field to Agent struct

**Priority**: CRITICAL
**Size**: Small (S)
**Estimated Effort**: 30 minutes
**Type**: feature

#### Description
Add `lifecycle: Vec<Arc<dyn AgentLifecycle>>` field to `Agent` struct and initialize it empty in constructors.

#### Acceptance Criteria
- [ ] Field added to Agent struct (src/agent.rs:60-64)
- [ ] Initialized empty in `Agent::new()`
- [ ] Existing tests still pass
- [ ] No breaking changes

#### Implementation Notes

**Struct Update** (src/agent.rs):
```rust
use std::sync::Arc;
use crate::lifecycle::AgentLifecycle;

pub struct Agent {
    pub(crate) config: AgentConfig,
    pub(crate) tools: HashMap<String, Arc<dyn Tool>>,
    provider: Option<Box<dyn LLMProvider>>,
    lifecycle: Vec<Arc<dyn AgentLifecycle>>,  // NEW
}
```

**Constructor Update**:
```rust
impl Agent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            tools: HashMap::new(),
            provider: None,
            lifecycle: Vec::new(),  // NEW
        }
    }
}
```

#### Watch Out For
- Don't forget to initialize in all constructors
- Existing tests must pass unchanged

#### Dependencies
- **Blocked by**: ARCH-001 (needs AgentLifecycle trait)
- **Blocks**: ARCH-004 (builder method needs field)

---

### ARCH-004: Implement .with_lifecycle() builder

**Priority**: CRITICAL
**Size**: Small (S)
**Estimated Effort**: 1 hour
**Type**: feature

#### Description
Implement `.with_lifecycle()` builder method for Agent to register hooks.

#### Acceptance Criteria
- [ ] Method added to Agent impl
- [ ] Accepts `impl AgentLifecycle + 'static`
- [ ] Pushes hook to lifecycle vec
- [ ] Returns self for chaining
- [ ] Rustdoc with example

#### Implementation Notes

**Method** (src/agent.rs):
```rust
impl Agent {
    /// Add a lifecycle hook to this agent
    ///
    /// Hooks are executed in registration order.
    ///
    /// # Example
    /// ```ignore
    /// let agent = create_agent("my-agent")
    ///     .with_lifecycle(MyHook::new())
    ///     .with_lifecycle(AnotherHook::new());
    /// ```
    pub fn with_lifecycle(mut self, hook: impl AgentLifecycle + 'static) -> Self {
        self.lifecycle.push(Arc::new(hook));
        self
    }
}
```

#### Watch Out For
- Must work with builder pattern chaining
- Arc::new() is correct (trait object needs Arc)

#### Dependencies
- **Blocked by**: ARCH-003 (needs lifecycle field)
- **Blocks**: ARCH-005 (integration needs registration)

---

### ARCH-005: Integrate before_agent hook

**Priority**: CRITICAL
**Size**: Medium (M)
**Estimated Effort**: 1-2 hours
**Type**: feature

#### Description
Call `before_agent` hook at start of `Agent::run()` method.

#### Acceptance Criteria
- [ ] Hook called before message building
- [ ] Input transformed by hook chain
- [ ] Errors propagate correctly
- [ ] Empty lifecycle vec handled (fast path)

#### Implementation Notes

**Integration Point** (src/agent.rs:105):
```rust
pub async fn run(&self, input: impl Into<String>) -> crate::Result<String> {
    let provider = /* ... */;

    // Hook 1: before_agent
    let mut input = input.into();
    for hook in &self.lifecycle {
        input = hook.before_agent(&input).await?;
    }

    // Rest of method...
}
```

#### Watch Out For
- Input must be mutable to accumulate transformations
- Error propagation with `?` operator

#### Dependencies
- **Blocked by**: ARCH-004 (needs hook registration)
- **Blocks**: ARCH-006

---

### ARCH-006: Integrate before_model hook

**Priority**: CRITICAL
**Size**: Medium (M)
**Estimated Effort**: 1-2 hours
**Type**: feature

#### Description
Call `before_model` hook before each `provider.complete()` call.

#### Acceptance Criteria
- [ ] Hook called in tool-calling loop
- [ ] Messages transformed by hook chain
- [ ] Called on each iteration (not just first)
- [ ] Errors propagate correctly

#### Implementation Notes

**Integration Point** (src/agent.rs:140):
```rust
for iteration in 0..max_iterations {
    // Hook 2: before_model
    for hook in &self.lifecycle {
        messages = hook.before_model(messages).await?;
    }

    let response = provider
        .complete(messages.clone(), tool_defs.clone())
        .await?;
    // ...
}
```

#### Watch Out For
- Messages modified in place, affects next iteration
- Clone messages for provider call (hooks shouldn't see cloned version)

#### Dependencies
- **Blocked by**: ARCH-005
- **Blocks**: ARCH-007

---

### ARCH-007: Integrate wrap_model_call hook

**Priority**: CRITICAL
**Size**: Large (L)
**Estimated Effort**: 3-4 hours
**Type**: feature

#### Description
Wrap `provider.complete()` calls with `wrap_model_call` hook, supporting hook chaining.

#### Acceptance Criteria
- [ ] Hook wraps provider.complete()
- [ ] Multiple hooks chain correctly (outer to inner)
- [ ] Fast path when lifecycle empty
- [ ] Errors propagate correctly

#### Implementation Notes

**Helper Method** (src/agent.rs):
```rust
impl Agent {
    async fn call_with_model_hooks<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse>
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = ProviderResult<ProviderResponse>> + Send,
    {
        if self.lifecycle.is_empty() {
            return f().await;
        }

        // Chain hooks from last to first (so first hook wraps outermost)
        let mut call: Box<dyn Fn() -> _ + Send> = Box::new(f);
        for hook in self.lifecycle.iter().rev() {
            let hook = Arc::clone(hook);
            let prev_call = call;
            call = Box::new(move || {
                let hook = Arc::clone(&hook);
                async move {
                    hook.wrap_model_call(|| prev_call()).await
                }
            });
        }
        call().await
    }
}
```

**Integration** (src/agent.rs:144):
```rust
let response = if self.lifecycle.is_empty() {
    provider.complete(messages.clone(), tool_defs.clone()).await?
} else {
    self.call_with_model_hooks(|| async {
        provider.complete(messages.clone(), tool_defs.clone()).await
    }).await?
};
```

#### Watch Out For
- Hook chaining order matters (first registered = outermost)
- Closure capturing can be tricky
- Fast path optimization critical for performance

#### Dependencies
- **Blocked by**: ARCH-006
- **Blocks**: ARCH-008

---

### ARCH-008: Integrate after_model hook

**Priority**: CRITICAL
**Size**: Medium (M)
**Estimated Effort**: 2 hours
**Type**: feature

#### Description
Call `after_model` hook after LLM response, handle HookAction variants.

#### Acceptance Criteria
- [ ] Hook called after provider.complete()
- [ ] HookAction::Continue proceeds normally
- [ ] HookAction::Reject stops execution with error
- [ ] HookAction::Modify replaces response
- [ ] HookAction::Approve treated as Continue
- [ ] Multiple hooks chain correctly

#### Implementation Notes

**Integration** (src/agent.rs:146):
```rust
let mut response = provider.complete(...).await?;

// Hook 4: after_model
for hook in &self.lifecycle {
    match hook.after_model(&response).await? {
        HookAction::Continue | HookAction::Approve => {
            // Proceed normally
        }
        HookAction::Reject(reason) => {
            return Err(reason.into());
        }
        HookAction::Modify(new_response) => {
            response = new_response;
        }
    }
}

// Continue with response...
```

#### Watch Out For
- Response may be modified multiple times (chain hooks)
- Early return on Reject
- Both Continue and Approve mean "proceed"

#### Dependencies
- **Blocked by**: ARCH-007, ARCH-002 (needs HookAction)
- **Blocks**: ARCH-009

---

### ARCH-009: Integrate wrap_tool_call hook

**Priority**: CRITICAL
**Size**: Large (L)
**Estimated Effort**: 3-4 hours
**Type**: feature

#### Description
Wrap `tool.execute()` calls with `wrap_tool_call` hook, supporting hook chaining.

#### Acceptance Criteria
- [ ] Hook wraps tool.execute()
- [ ] Tool name passed to hook
- [ ] Multiple hooks chain correctly
- [ ] Fast path when lifecycle empty
- [ ] Errors propagate correctly

#### Implementation Notes

**Helper Method** (src/agent.rs):
```rust
impl Agent {
    async fn call_with_tool_hooks<F, Fut>(&self, name: &str, f: F) -> ToolResult
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = ToolResult> + Send,
    {
        if self.lifecycle.is_empty() {
            return f().await;
        }

        // Similar chaining logic as wrap_model_call
        // ...
    }
}
```

**Integration** (src/agent.rs:162):
```rust
let result = if self.lifecycle.is_empty() {
    tool.execute(call.arguments)?
} else {
    self.call_with_tool_hooks(&call.name, || async {
        tool.execute(call.arguments.clone())
    }).await?
};
```

#### Watch Out For
- Tool execution is currently sync, may need async wrapper
- Tool name must be passed to hook
- Clone arguments for closure capture

#### Dependencies
- **Blocked by**: ARCH-008
- **Blocks**: ARCH-010

---

### ARCH-010: Integrate after_agent hook

**Priority**: CRITICAL
**Size**: Medium (M)
**Estimated Effort**: 1-2 hours
**Type**: feature

#### Description
Call `after_agent` hook before returning final result from `Agent::run()`.

#### Acceptance Criteria
- [ ] Hook called before final return
- [ ] Result transformed by hook chain
- [ ] Errors propagate correctly
- [ ] Called only on success path (not on errors)

#### Implementation Notes

**Integration** (src/agent.rs:180):
```rust
match response {
    ProviderResponse::Text(text) => {
        let mut result = text;

        // Hook 6: after_agent
        for hook in &self.lifecycle {
            result = hook.after_agent(&result).await?;
        }

        return Ok(result);
    }
    // ...
}
```

#### Watch Out For
- Only called on success (not when returning error)
- Result accumulated through hook chain

#### Dependencies
- **Blocked by**: ARCH-009
- **Blocks**: TEST-003 (all hooks must be integrated for full testing)

---

### TEST-002: Unit tests for AgentLifecycle defaults

**Priority**: HIGH
**Size**: Medium (M)
**Estimated Effort**: 2 hours
**Type**: test

#### Description
Write unit tests verifying default implementations of AgentLifecycle return passthrough values.

#### Acceptance Criteria
- [ ] Test for each of 6 hook methods
- [ ] Verify passthrough behavior (output = input)
- [ ] Test with various input types
- [ ] All tests pass

#### Implementation Notes

**Test File**: `src/lifecycle.rs` (in #[cfg(test)] mod tests)

**Example Test**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct DefaultHook;

    #[async_trait]
    impl AgentLifecycle for DefaultHook {}

    #[tokio::test]
    async fn test_before_agent_default() {
        let hook = DefaultHook;
        let result = hook.before_agent("test input").await.unwrap();
        assert_eq!(result, "test input");
    }

    // Similar for other hooks...
}
```

#### Dependencies
- **Blocked by**: ARCH-001
- **Blocks**: None (can be done in parallel)

---

### TEST-003: Integration tests for hook execution

**Priority**: CRITICAL
**Size**: Large (L)
**Estimated Effort**: 4-5 hours
**Type**: test

#### Description
Comprehensive integration tests verifying hooks execute in correct order and affect agent behavior.

#### Acceptance Criteria
- [ ] Test hook execution order (first registered = first called)
- [ ] Test each hook type in context
- [ ] Test HookAction variants (Continue, Reject, Modify)
- [ ] Test multiple hooks chaining
- [ ] Test error propagation
- [ ] All tests pass

#### Implementation Notes

**Test File**: `src/agent.rs` (in #[cfg(test)] mod tests)

**Example Tests**:
```rust
#[tokio::test]
async fn test_before_agent_hook_transforms_input() {
    struct UppercaseHook;

    #[async_trait]
    impl AgentLifecycle for UppercaseHook {
        async fn before_agent(&self, input: &str) -> Result<String> {
            Ok(input.to_uppercase())
        }
    }

    let agent = create_agent("test")
        .with_provider(Box::new(MockProvider::new("response")))
        .with_lifecycle(UppercaseHook);

    // Test that input was uppercased before processing
    // (would need to capture in mock provider)
}

#[tokio::test]
async fn test_after_model_reject_stops_execution() {
    struct RejectHook;

    #[async_trait]
    impl AgentLifecycle for RejectHook {
        async fn after_model(&self, _: &ProviderResponse) -> Result<HookAction> {
            Ok(HookAction::Reject("Rejected by hook".into()))
        }
    }

    let agent = create_agent("test")
        .with_provider(Box::new(MockProvider::new("response")))
        .with_lifecycle(RejectHook);

    let result = agent.run("test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Rejected"));
}

#[tokio::test]
async fn test_multiple_hooks_chain_correctly() {
    // Test that hooks execute in registration order
    // ...
}
```

#### Dependencies
- **Blocked by**: ARCH-010 (all hooks must be integrated)
- **Blocks**: None

---

### TEST-004: Regression tests (0 hooks behavior)

**Priority**: CRITICAL
**Size**: Medium (M)
**Estimated Effort**: 2 hours
**Type**: test

#### Description
Verify that agents with zero hooks behave identically to pre-lifecycle implementation.

#### Acceptance Criteria
- [ ] All existing tests pass unchanged
- [ ] New tests verify zero-hook behavior
- [ ] Performance unchanged (use fast path)
- [ ] No observable differences

#### Implementation Notes

**Approach**:
1. Run existing test suite (should pass)
2. Add explicit tests for zero-hook case
3. Compare behavior to baseline

**Example Test**:
```rust
#[tokio::test]
async fn test_agent_without_hooks_unchanged() {
    let agent = create_agent("test")
        .with_provider(Box::new(MockProvider::new("response")));

    let result = agent.run("test").await.unwrap();
    assert_eq!(result, "response");

    // Verify this is identical to pre-lifecycle behavior
}
```

#### Dependencies
- **Blocked by**: ARCH-010
- **Blocks**: None

---

### PERF-001: Performance benchmarks

**Priority**: HIGH
**Size**: Medium (M)
**Estimated Effort**: 3 hours
**Type**: perf

#### Description
Create benchmarks measuring overhead of lifecycle hooks at 0, 1, 3, 5 hooks.

#### Acceptance Criteria
- [ ] Benchmark for 0 hooks (baseline)
- [ ] Benchmark for 1 hook (< 5% overhead)
- [ ] Benchmark for 3 hooks (< 8% overhead)
- [ ] Benchmark for 5 hooks (< 10% overhead)
- [ ] Results documented

#### Implementation Notes

**Benchmark File**: `benches/lifecycle_overhead.rs`

**Using criterion**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle");

    // Baseline: 0 hooks
    group.bench_function("no_hooks", |b| {
        b.iter(|| {
            // Run agent with no hooks
        });
    });

    // 1 hook
    group.bench_function("one_hook", |b| {
        b.iter(|| {
            // Run agent with 1 passthrough hook
        });
    });

    // 3 hooks
    group.bench_function("three_hooks", |b| {
        b.iter(|| {
            // Run agent with 3 passthrough hooks
        });
    });

    // 5 hooks
    group.bench_function("five_hooks", |b| {
        b.iter(|| {
            // Run agent with 5 passthrough hooks
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_lifecycle);
criterion_main!(benches);
```

**Add to Cargo.toml**:
```toml
[[bench]]
name = "lifecycle_overhead"
harness = false

[dev-dependencies]
criterion = "0.5"
```

#### Dependencies
- **Blocked by**: ARCH-010
- **Blocks**: None

---

### DOCS-002: Rustdoc for lifecycle.rs

**Priority**: HIGH
**Size**: Small (S)
**Estimated Effort**: 1-2 hours
**Type**: docs

#### Description
Comprehensive rustdoc comments for AgentLifecycle trait and HookAction enum.

#### Acceptance Criteria
- [ ] Module-level doc comment
- [ ] Trait-level doc comment with overview
- [ ] Doc comment for each hook method
- [ ] Doc comment for HookAction and variants
- [ ] Code examples in rustdoc
- [ ] `cargo doc` builds without warnings

#### Implementation Notes

**Module Doc**:
```rust
//! Agent lifecycle hooks for middleware and intervention points
//!
//! The lifecycle system provides 6 hook points where middleware can intercept,
//! modify, or observe agent execution:
//!
//! - `before_agent`: Transform input before processing
//! - `before_model`: Modify messages before LLM call
//! - `wrap_model_call`: Wrap LLM calls (retry, fallback, logging)
//! - `after_model`: Inspect/modify response, HITL approval
//! - `wrap_tool_call`: Wrap tool execution (retry, logging)
//! - `after_agent`: Transform final result
//!
//! # Example
//! ```ignore
//! struct LoggingHook;
//!
//! #[async_trait]
//! impl AgentLifecycle for LoggingHook {
//!     async fn before_agent(&self, input: &str) -> Result<String> {
//!         println!("Input: {}", input);
//!         Ok(input.to_string())
//!     }
//! }
//!
//! let agent = create_agent("my-agent")
//!     .with_lifecycle(LoggingHook);
//! ```
```

#### Dependencies
- **Blocked by**: ARCH-001
- **Blocks**: None

---

### DOCS-003: Example demonstrating hooks

**Priority**: HIGH
**Size**: Medium (M)
**Estimated Effort**: 2-3 hours
**Type**: docs

#### Description
Create `examples/lifecycle_hooks.rs` demonstrating hook usage patterns.

#### Acceptance Criteria
- [ ] Example file created
- [ ] Demonstrates at least 3 different hooks
- [ ] Shows hook chaining (multiple hooks)
- [ ] Compiles and runs successfully
- [ ] Well-commented for learning

#### Implementation Notes

**File**: `examples/lifecycle_hooks.rs`

**Example Content**:
```rust
use patinox::*;
use async_trait::async_trait;

// Example 1: Logging hook
struct LoggingHook;

#[async_trait]
impl AgentLifecycle for LoggingHook {
    async fn before_agent(&self, input: &str) -> Result<String> {
        println!("[LOG] Input: {}", input);
        Ok(input.to_string())
    }

    async fn after_agent(&self, result: &str) -> Result<String> {
        println!("[LOG] Output: {}", result);
        Ok(result.to_string())
    }
}

// Example 2: Uppercase transformer
struct UppercaseHook;

#[async_trait]
impl AgentLifecycle for UppercaseHook {
    async fn before_agent(&self, input: &str) -> Result<String> {
        Ok(input.to_uppercase())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let agent = create_agent("demo")
        .with_provider(Box::new(MockProvider::new("Hello, World!")))
        .with_lifecycle(LoggingHook)
        .with_lifecycle(UppercaseHook);

    let result = agent.run("hello").await?;
    println!("Result: {}", result);

    Ok(())
}
```

#### Dependencies
- **Blocked by**: ARCH-010
- **Blocks**: None

---

### DOCS-004: Update architecture documentation

**Priority**: MEDIUM
**Size**: Small (S)
**Estimated Effort**: 1 hour
**Type**: docs

#### Description
Update context network architecture documents to reflect lifecycle hook addition.

#### Acceptance Criteria
- [ ] Update roadmap.md with Layer 2.5 status
- [ ] Reference lifecycle hooks in relevant sections
- [ ] No broken links
- [ ] Accurate reflection of implementation

#### Implementation Notes

**Files to Update**:
- `context-network/planning/roadmap.md` - Mark Layer 2.5 as complete
- Add completion record

#### Dependencies
- **Blocked by**: ARCH-010
- **Blocks**: None

---

## Implementation Order

### Recommended Sequence

**Day 1: Foundation (4-6 hours)**
1. ARCH-001: Create lifecycle.rs trait
2. ARCH-002: HookAction enum
3. ARCH-003: Add lifecycle field
4. ARCH-004: .with_lifecycle() builder
5. TEST-002: Unit tests for defaults
6. DOCS-002: Rustdoc for lifecycle.rs

**Day 2: Hook Integration (6-8 hours)**
7. ARCH-005: before_agent hook
8. ARCH-006: before_model hook
9. ARCH-007: wrap_model_call hook (complex)
10. ARCH-008: after_model hook
11. ARCH-009: wrap_tool_call hook (complex)
12. ARCH-010: after_agent hook

**Day 3: Testing & Benchmarks (6-8 hours)**
13. TEST-003: Integration tests
14. TEST-004: Regression tests
15. PERF-001: Performance benchmarks

**Day 4: Documentation & Polish (3-4 hours)**
16. DOCS-003: Example code
17. DOCS-004: Architecture docs
18. Final review and cleanup

**Total**: ~20-26 hours (~3-4 days)

---

## Parallel Work Opportunities

### Can Be Done in Parallel
- ARCH-001 and ARCH-002 (independent)
- TEST-002 and DOCS-002 (both depend on ARCH-001)
- TEST-003, TEST-004, PERF-001 (all depend on ARCH-010)
- DOCS-003 and DOCS-004 (both depend on ARCH-010)

### Must Be Sequential
- ARCH-003 requires ARCH-001
- ARCH-004 requires ARCH-003
- ARCH-005 through ARCH-010 must be sequential (build on each other)

---

## Risk Mitigation

### High-Risk Tasks
1. **ARCH-007** (wrap_model_call) - Complex hook chaining
   - Mitigation: Prototype chaining logic separately first
   - Fallback: Simpler single-hook-only version

2. **ARCH-009** (wrap_tool_call) - Similar complexity to ARCH-007
   - Mitigation: Reuse patterns from ARCH-007
   - Fallback: Copy implementation approach

3. **PERF-001** (benchmarks) - May not meet targets
   - Mitigation: Profile to find bottlenecks
   - Fallback: Optimize fast path, accept slightly higher overhead

---

## Completion Criteria

### All Tasks Complete When:
- [ ] All 17 tasks checked off
- [ ] All tests passing (existing + new)
- [ ] Benchmarks meet targets
- [ ] Documentation complete
- [ ] Code review passed
- [ ] Ready for merge

---

## Metadata
- **Created**: 2025-10-16
- **Total Tasks**: 17
- **Estimated Effort**: 20-26 hours (3-4 days)
- **Critical Path**: ARCH-001 → ARCH-010 → TEST-003
- **Status**: READY - Can begin implementation
