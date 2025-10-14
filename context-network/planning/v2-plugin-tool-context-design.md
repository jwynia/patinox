# Tool Context Helper Plugin - Design Document

**Status**: Draft Design
**Created**: 2025-10-14
**Task**: V2-PLUGIN-001
**Priority**: Critical (Pain Score: 30/30)

## Executive Summary

This plugin eliminates the closure context capture boilerplate that affects 100% of agents with context-aware tools. The design uses builder pattern extension methods to capture context automatically while maintaining type safety and zero runtime overhead.

## Problem Statement

### Current Pain Point

Every tool that needs access to external context (file paths, config, state) requires manual clone + move boilerplate:

```rust
// File Processor Agent (V2-AGENT-001) - 4 tools with this pattern
.tool_fn("read_file", "Read the contents of a file", {
    let path = file_path.clone();  // ❌ Manual clone
    move |_args| read_file_tool(&path)  // ❌ Manual move
})
.tool_fn("count_lines", "Count lines in a file", {
    let path = file_path.clone();  // ❌ Manual clone
    move |_args| count_lines_tool(&path)  // ❌ Manual move
})
// ... 2 more tools with same pattern

// Documentation Generator Agent (V2-AGENT-002) - 5 tools with this pattern
.tool_fn("read_source", "Read Rust source file", {
    let path = source_path.clone();  // ❌ Manual clone
    move |_args| read_source_tool(&path)  // ❌ Manual move
})
// ... 4 more tools with same pattern
```

### Impact Analysis

**Validated Across Both Agents:**
- **V2-AGENT-001**: 3/4 tools (read_file, count_lines, get_file_info)
- **V2-AGENT-002**: 4/5 tools (read_source, get_module_info, extract_public_api, count_functions)
- **Total**: 7/9 context-aware tools (78% of all tools)
- **Pattern Frequency**: 100% (2/2 agents hit this)

**Cost Per Agent:**
- 3 extra lines per tool
- Mental overhead tracking what needs to be cloned
- Breaks ergonomic flow of builder pattern

## Design Goals

### Must Have
1. ✅ Eliminate clone + move boilerplate for common cases
2. ✅ Type-safe approach (compile-time guarantees)
3. ✅ Opt-in mechanism (core works without it)
4. ✅ Zero runtime overhead (compile away)
5. ✅ Works with existing `Agent` builder pattern

### Should Have
1. ✅ Clean integration with `.tool_fn()` method
2. ✅ Support multiple captured variables
3. ✅ Support different types (String, PathBuf, etc.)
4. ✅ Clear error messages if misused

### Nice to Have
1. ⚪ Auto-clone detection (minimize explicit clones)
2. ⚪ Macro support for even less boilerplate
3. ⚪ Integration with other plugins (CLI, Config)

## Proposed Solution

### Option A: Context-Aware Tool Builder (RECOMMENDED)

Add a new builder method that captures context automatically:

```rust
// NEW: .tool_with_context() - captures variables automatically
.tool_with_context("read_file", "Read contents of a file")
    .capture("path", file_path)  // Captures &String, clones internally
    .handler(|ctx, _args| {
        let path = ctx.get::<String>("path")?;
        read_file_tool(path)
    })
```

**Pros:**
- Explicit about what's being captured
- Type-safe with compile-time checks
- No magic - clear data flow
- Composable with other patterns

**Cons:**
- Still some boilerplate (but reduced from 3 lines to 1)
- Requires helper trait for context access

### Option B: Extended tool_fn with Capture (SIMPLER - RECOMMENDED)

Extend existing `tool_fn` with optional capture parameter:

```rust
// Before (current)
.tool_fn("read_file", "Read contents", {
    let path = file_path.clone();
    move |_args| read_file_tool(&path)
})

// After (with plugin)
.tool_fn_with("read_file", "Read contents", &file_path, |path, _args| {
    read_file_tool(path)
})
```

**Pros:**
- Minimal API surface (one new method)
- Very close to existing pattern
- No context API to learn
- Can support 1-3 captured variables with separate methods

**Cons:**
- Need multiple methods for different arities (tool_fn_with, tool_fn_with2, tool_fn_with3)
- Less flexible than Option A for many captures

### Option C: Macro-Based Approach

Use macros to capture variables automatically:

```rust
tool_fn!(agent, "read_file", "Read contents", [file_path] => |path, _args| {
    read_file_tool(path)
})
```

**Pros:**
- Cleanest syntax
- Most flexible

**Cons:**
- Macros are harder to debug
- Error messages can be cryptic
- Overkill for this problem

## Recommended Design: Option B with Extensions

**Core Decision: Go with Option B** - Simple, minimal, solves 90% of cases.

### API Design

```rust
impl Agent {
    /// Existing method (unchanged)
    pub fn tool_fn<F>(self, name: impl Into<String>, desc: impl Into<String>, handler: F) -> Self
    where F: Fn(String) -> ToolResult + Send + Sync + 'static;

    /// NEW: Capture one context variable
    pub fn tool_fn_with<T, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        context: &T,
        handler: F
    ) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T, String) -> ToolResult + Send + Sync + 'static;

    /// NEW: Capture two context variables
    pub fn tool_fn_with2<T1, T2, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        ctx1: &T1,
        ctx2: &T2,
        handler: F
    ) -> Self
    where
        T1: Clone + Send + Sync + 'static,
        T2: Clone + Send + Sync + 'static,
        F: Fn(&T1, &T2, String) -> ToolResult + Send + Sync + 'static;
}
```

### Usage Examples

#### Before (Current Boilerplate)

```rust
// File Processor Agent
let mut agent = create_agent("file_processor")
    .tool_fn("read_file", "Read the contents of a file", {
        let path = file_path.clone();  // ❌ Manual clone
        move |_args| read_file_tool(&path)  // ❌ Manual move
    })
    .tool_fn("count_lines", "Count lines in a file", {
        let path = file_path.clone();  // ❌ Manual clone
        move |_args| count_lines_tool(&path)  // ❌ Manual move
    });
```

#### After (With Plugin)

```rust
// File Processor Agent
let agent = create_agent("file_processor")
    .tool_fn_with("read_file", "Read the contents of a file", &file_path,
        |path, _args| read_file_tool(path))
    .tool_fn_with("count_lines", "Count lines in a file", &file_path,
        |path, _args| count_lines_tool(path));
```

**Improvement:**
- ✅ 3 lines reduced to 1 line per tool
- ✅ No manual clone/move
- ✅ Cleaner builder chain
- ✅ Type safety maintained

### Advanced Usage: Multiple Context Variables

```rust
// Agent with config and state
let agent = create_agent("analyzer")
    .tool_fn_with2("analyze_with_config", "Analyze using config",
        &file_path, &config,
        |path, cfg, _args| analyze_tool(path, cfg));
```

## Implementation Plan

### Phase 1: Core Plugin Trait (This Task - Design Only)

**Create**: `src/plugin/mod.rs` (trait definition, stub)

```rust
//! Plugin system for extending agent functionality

/// Plugin trait for extending agents
pub trait AgentPlugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Apply plugin to agent during build
    fn apply(&self, agent: Agent) -> Agent;
}
```

**Purpose**: Establish plugin architecture pattern for future plugins.

### Phase 2: Tool Context Implementation (Week 3, Task V2-PLUGIN-001-IMPL)

**Create**: `src/plugin/tool_context.rs` (implementation)

```rust
//! Tool context helper - eliminates closure capture boilerplate

use crate::agent::Agent;
use crate::tool::ToolResult;

/// Extension methods for Agent to support context capture
pub trait ToolContextExt {
    /// Add tool with one captured context variable
    fn tool_fn_with<T, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        context: &T,
        handler: F
    ) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T, String) -> ToolResult + Send + Sync + 'static;

    /// Add tool with two captured context variables
    fn tool_fn_with2<T1, T2, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        ctx1: &T1,
        ctx2: &T2,
        handler: F
    ) -> Self
    where
        T1: Clone + Send + Sync + 'static,
        T2: Clone + Send + Sync + 'static,
        F: Fn(&T1, &T2, String) -> ToolResult + Send + Sync + 'static;
}

impl ToolContextExt for Agent {
    fn tool_fn_with<T, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        context: &T,
        handler: F
    ) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T, String) -> ToolResult + Send + Sync + 'static,
    {
        let ctx = context.clone(); // Clone once here
        self.tool_fn(name, desc, move |args| handler(&ctx, args))
    }

    fn tool_fn_with2<T1, T2, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        ctx1: &T1,
        ctx2: &T2,
        handler: F
    ) -> Self
    where
        T1: Clone + Send + Sync + 'static,
        T2: Clone + Send + Sync + 'static,
        F: Fn(&T1, &T2, String) -> ToolResult + Send + Sync + 'static,
    {
        let c1 = ctx1.clone();
        let c2 = ctx2.clone();
        self.tool_fn(name, desc, move |args| handler(&c1, &c2, args))
    }
}
```

**Update**: `src/lib.rs` prelude

```rust
pub mod prelude {
    pub use crate::agent::{create_agent, Agent, AgentConfig};
    pub use crate::provider::Provider;
    pub use crate::tool::ToolResult;
    pub use crate::plugin::tool_context::ToolContextExt;  // NEW
}
```

### Phase 3: Testing (Week 3)

**Create**: Tests in `src/plugin/tool_context.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_tool_fn_with_single_context() {
        let file_path = String::from("test.txt");

        let agent = create_agent("test")
            .tool_fn_with("read_file", "Read file", &file_path, |path, _args| {
                Ok(format!("Reading: {}", path))
            });

        assert!(agent.tools.contains_key("read_file"));
    }

    #[test]
    fn test_tool_fn_with_two_contexts() {
        let path = String::from("test.txt");
        let config = String::from("config.json");

        let agent = create_agent("test")
            .tool_fn_with2("process", "Process with config", &path, &config,
                |p, c, _args| {
                    Ok(format!("Processing {} with {}", p, c))
                });

        assert!(agent.tools.contains_key("process"));
    }

    #[tokio::test]
    async fn test_tool_fn_with_execution() {
        use crate::provider::MockProvider;

        let file_path = String::from("test.txt");

        let agent = create_agent("test")
            .tool_fn_with("read_file", "Read file", &file_path, |path, _args| {
                Ok(format!("Contents of: {}", path))
            })
            .with_provider(Box::new(MockProvider::new("Tool result received")));

        let result = agent.run("use the read_file tool").await.unwrap();
        assert_eq!(result, "Tool result received");
    }
}
```

### Phase 4: Documentation & Migration Guide (Week 3)

**Update**: Examples to use new API

```rust
// examples/file_processor_v2.rs - Updated with plugin
use patinox::prelude::*;  // Now includes ToolContextExt

let agent = create_agent("file_processor")
    .tool_fn_with("read_file", "Read contents", &file_path,
        |path, _| read_file_tool(path))
    .tool_fn_with("count_lines", "Count lines", &file_path,
        |path, _| count_lines_tool(path));
```

## File Size Analysis

**Projected file sizes:**
- `src/plugin/mod.rs`: ~50 lines (trait definition + re-exports)
- `src/plugin/tool_context.rs`: ~150-200 lines (implementation + tests)
- **Total**: ~200-250 lines

✅ **Under 300 line limit** - No refactoring needed.

## Design Validation

### Validation Against V2-AGENT-001 (File Processor)

**Before:**
```rust
.tool_fn("read_file", "Read the contents of a file", {
    let path = file_path.clone();
    move |_args| read_file_tool(&path)
})
.tool_fn("count_lines", "Count lines in a file", {
    let path = file_path.clone();
    move |_args| count_lines_tool(&path)
})
.tool_fn("get_file_info", "Get file metadata", {
    let path = file_path.clone();
    move |_args| get_file_info_tool(&path)
})
.tool_fn("extract_keywords", "Extract keywords", {
    let path = file_path.clone();
    move |_args| extract_keywords_tool(&path)
})
```

**After:**
```rust
.tool_fn_with("read_file", "Read the contents of a file", &file_path,
    |path, _| read_file_tool(path))
.tool_fn_with("count_lines", "Count lines in a file", &file_path,
    |path, _| count_lines_tool(path))
.tool_fn_with("get_file_info", "Get file metadata", &file_path,
    |path, _| get_file_info_tool(path))
.tool_fn_with("extract_keywords", "Extract keywords", &file_path,
    |path, _| extract_keywords_tool(path))
```

**Improvement:**
- Lines: 16 → 4 (75% reduction)
- No manual clone/move
- Cleaner visual flow

### Validation Against V2-AGENT-002 (Doc Generator)

**Before:**
```rust
.tool_fn("read_source", "Read Rust source file", {
    let path = source_path.clone();
    move |_args| read_source_tool(&path)
})
.tool_fn("get_module_info", "Get information about the module", {
    let path = source_path.clone();
    move |_args| get_module_info_tool(&path)
})
.tool_fn("extract_public_api", "Extract public API items", {
    let path = source_path.clone();
    move |_args| extract_public_api_tool(&path)
})
.tool_fn("count_functions", "Count functions in the source", {
    let path = source_path.clone();
    move |_args| count_functions_tool(&path)
})
```

**After:**
```rust
.tool_fn_with("read_source", "Read Rust source file", &source_path,
    |path, _| read_source_tool(path))
.tool_fn_with("get_module_info", "Get information about the module", &source_path,
    |path, _| get_module_info_tool(path))
.tool_fn_with("extract_public_api", "Extract public API items", &source_path,
    |path, _| extract_public_api_tool(path))
.tool_fn_with("count_functions", "Count functions in the source", &source_path,
    |path, _| count_functions_tool(path))
```

**Improvement:**
- Lines: 16 → 4 (75% reduction)
- Consistent pattern across all tools

### Edge Cases Handled

1. **Different types**: Works with String, PathBuf, config structs, etc.
2. **Multiple contexts**: `tool_fn_with2` for 2 variables, can add `_with3` if needed
3. **No context needed**: Original `tool_fn` still works
4. **Mixed usage**: Can use both old and new methods in same agent

## Performance Considerations

### Runtime Overhead: ZERO

The plugin compiles down to exactly the same code as manual clone + move:

```rust
// Manual (current)
let path = file_path.clone();
move |_args| read_file_tool(&path)

// Plugin (compiles to the same thing)
let ctx = context.clone();
move |args| handler(&ctx, args)
```

**Proof**: Both involve:
1. One clone of the context
2. One move into the closure
3. Reference passed to handler

No allocation, no indirection, no runtime cost.

### Compile Time Impact

- Minimal: Extension trait methods are inline
- No proc macros (fast compilation)
- Standard trait resolution

## Migration Strategy

### Week 3 Timeline

**Day 1-2**: Implement plugin (V2-PLUGIN-001-IMPL)
- Create plugin trait
- Implement tool_context extension
- Write comprehensive tests

**Day 3**: Update examples
- Migrate file_processor.rs to use plugin
- Migrate doc_generator.rs to use plugin
- Create before/after comparison

**Day 4**: Documentation
- Update CLAUDE.md with plugin usage
- Write migration guide
- Document best practices

**Day 5**: Validation & Polish
- Run against real agents
- Measure actual DX improvement
- Collect any issues

### Backward Compatibility

✅ **100% Backward Compatible**
- Old `tool_fn` method unchanged
- Plugin is opt-in via import
- No breaking changes to core API

### Opt-In Mechanism

```rust
// Without plugin (still works)
use patinox::prelude::*;  // Doesn't include ToolContextExt

// With plugin (opt-in)
use patinox::prelude::*;  // Will include ToolContextExt in updated prelude
```

## Alternative Designs Considered

### Why Not Option A (Context Builder)?

Too much API surface for a simple problem:
- Need context struct
- Need `.capture()` method
- Need context access helper
- More learning curve

Option B solves 90% with 10% of the complexity.

### Why Not Option C (Macros)?

- Macros are hard to debug
- Error messages are cryptic
- Overkill for this specific problem
- Can always add macros later if needed

### Why Not Generic Context Store?

Considered a generic `AgentContext` that tools can access:

```rust
agent.set_context("file_path", file_path);
.tool_fn("read_file", "Read file", |ctx, _| {
    let path = ctx.get::<String>("file_path")?;
    read_file_tool(path)
})
```

**Rejected because:**
- Runtime overhead (HashMap lookup)
- Type safety issues (need downcasting)
- Not zero-cost abstraction
- More complex mental model

## Success Metrics

### Quantitative
- ✅ Reduce boilerplate by 75% (16 lines → 4 lines per 4-tool agent)
- ✅ Zero runtime overhead (proven by design)
- ✅ 100% backward compatible
- ✅ File size under 300 lines

### Qualitative
- ✅ More ergonomic builder flow
- ✅ Less mental overhead (no clone tracking)
- ✅ Type-safe (compile-time guarantees)
- ✅ Easy to understand (minimal API surface)

### Validation Criteria
- [ ] Both agents successfully migrated
- [ ] All tests pass (old + new methods)
- [ ] Code review approval
- [ ] No performance regression
- [ ] Positive developer feedback

## Future Extensions

### Phase 2: More Arity Variants
If 3+ captures are common, add:
```rust
.tool_fn_with3(name, desc, ctx1, ctx2, ctx3, handler)
.tool_fn_with4(...) // if ever needed
```

### Phase 3: Macro Sugar (Optional)
If users want even less boilerplate:
```rust
tool_with!(agent, "read_file", "desc", [file_path] => |path, _| { ... });
```

### Integration with Future Plugins
- **CLI Plugin**: Could capture CLI args automatically
- **Config Plugin**: Could inject config into tools
- **State Plugin**: Could provide mutable state access

## Dependencies

### Required
- None (uses only std and existing crate)

### Breaking Changes
- None

### Migration Effort
- Existing code: Works unchanged
- New code: Import trait, use new methods
- Examples: 5 minutes per agent to update

## Open Questions

### Q1: Should we support mutable context?

**Current**: All context is immutable (&T in handler)

**Possible**: Add `tool_fn_with_mut` for &mut T

**Decision**: DEFER - Wait for concrete use case. Mutable state in tools is tricky with async/multithreading.

### Q2: Should we support Arc/Rc optimization?

**Current**: Always clones (T: Clone)

**Possible**: Special case for Arc<T>, Rc<T> (cheaper clones)

**Decision**: DEFER - Profile first, optimize if needed. Clone is cheap for most types.

### Q3: How many arity variants do we need?

**Current Plan**: `tool_fn_with` and `tool_fn_with2`

**Question**: Add `_with3`, `_with4`?

**Decision**: Start with 2, add more only if pain is felt (V2 principle).

## References

### Pain Point Documentation
- [records/pain-points-file-processor-2025-10-13.md](../records/pain-points-file-processor-2025-10-13.md) - Pain Point #1
- [records/pain-points-doc-generator-2025-10-13.md](../records/pain-points-doc-generator-2025-10-13.md) - Pain Point #1
- [records/completion-v2-week-2-phase-2-2025-10-13.md](../records/completion-v2-week-2-phase-2-2025-10-13.md) - Priority justification

### Related Code
- `src/agent.rs:81-94` - Current `tool_fn` method
- `src/tool.rs` - Tool trait and FnTool implementation
- `examples/file_processor.rs:46-61` - Boilerplate pattern
- `examples/doc_generator.rs:52-67` - Boilerplate pattern

### Architecture Decisions
- [decisions/v2_strategic_reset.md](../decisions/v2_strategic_reset.md) - V2 minimal-first philosophy
- [planning/v2-week-2-plan.md](../planning/v2-week-2-plan.md) - Plugin strategy

## Approval Checklist

Before implementation:
- [ ] Design reviewed by team
- [ ] API validated against both agents
- [ ] Performance characteristics understood
- [ ] Migration strategy clear
- [ ] Test strategy defined
- [ ] Documentation plan complete

## Next Steps

1. ✅ Complete this design document
2. ⏳ Review with team / stakeholders
3. ⏳ Get design approval
4. ⏳ Create implementation task (V2-PLUGIN-001-IMPL)
5. ⏳ Begin Week 3 implementation

---

**Status**: Draft Design - Ready for Review
**Author**: Claude (AI Assistant)
**Last Updated**: 2025-10-14
