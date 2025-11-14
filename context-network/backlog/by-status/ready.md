# Ready for Implementation

Tasks that are fully groomed, have no blockers, and are ready to be worked on immediately.

## What Makes a Task "Ready"?

A task is ready when:
- ✅ Acceptance criteria are clearly defined
- ✅ All dependencies are completed or resolved
- ✅ No open questions or decisions needed
- ✅ Estimated effort is reasonable
- ✅ Implementation approach is documented
- ✅ Someone can start within 5 minutes of reading

## Current Phase: V2 Layer 2 - Real Usage & Pain-Driven Plugins

**Context**:
- ✅ Layer 1 (Minimal Agent) completed October 12, 2025
- ✅ Week 2 Phase 1 (Real Provider) completed October 13, 2025
- ✅ Week 2 Phase 2 (Build Real Agents) completed October 13, 2025
- ✅ Week 2 Phase 3 (Pain Point Analysis) completed October 13, 2025
- ✅ Week 2 Phase 4 (Plugin Design) completed October 14, 2025
- ⏳ **Week 4 - Layer 2.5 (Lifecycle Hooks) - NOW READY**

**See**: [planning/v2-week-2-plan.md](../../planning/v2-week-2-plan.md) for full Week 2 strategy.
**See**: [decisions/lifecycle-hook-architecture.md](../../decisions/lifecycle-hook-architecture.md) for Layer 2.5 rationale.

---

## Week 2 Phase 4: First Plugin Design (Day 7)

### ✅ V2-PLUGIN-001: Design Tool Context Helper Plugin (COMPLETED)

**One-liner**: Design plugin to eliminate tool closure context capture boilerplate

**Priority**: Critical (Pain Score: 30/30 - validated across 100% of agents)
**Effort**: 4-6 hours (design + spec)
**Branch**: `feat/v2-tool-context-plugin`

**Why This Matters**:
- Pain Point #1 from both agents (V2-AGENT-001 and V2-AGENT-002)
- Affects 100% of context-aware tools (9/9 tools in both agents)
- Frequency: 3, Severity: 10, Score: 30 (CRITICAL)
- Universal pattern validated across different agent types

**Problem Statement**:
Every tool that needs access to external context (file paths, config, state) requires manual clone + move boilerplate:
```rust
let path = file_path.clone();
move |_args| read_file_tool(&path)
```
This adds 3 extra lines per tool and breaks the ergonomic flow of agent building.

**Acceptance Criteria**:
- [ ] Plugin trait defined with clear integration points
- [ ] Builder pattern integration designed (opt-in mechanism)
- [ ] API reduces boilerplate to zero for common cases
- [ ] Works with existing agent builder pattern
- [ ] V1 code assessed for import opportunities
- [ ] Implementation plan with test strategy created
- [ ] Design validated against both existing agents

**Data from Pain Point Analysis**:
- **V2-AGENT-001**: Hit on 3/4 tools (read_file, count_lines, get_file_info)
- **V2-AGENT-002**: Hit on 4/5 tools (read_source, get_module_info, extract_public_api, count_functions)
- **Total impact**: 7/9 context-aware tools (78% of all tools)

**Design Considerations**:
1. Should support captured variables without manual cloning
2. Must remain optional (core works without it)
3. Clean integration with `.tool_fn()` builder method
4. Type-safe approach preferred
5. Minimal runtime overhead

**Files to Create**:
- `context-network/planning/v2-plugin-tool-context-design.md` - Design document
- `src/plugin/mod.rs` - Plugin trait definition (stub)
- `src/plugin/tool_context.rs` - Tool context plugin (design spec)

**See Also**:
- [records/pain-points-file-processor-2025-10-13.md](../../records/pain-points-file-processor-2025-10-13.md) - Pain Point #1
- [records/pain-points-doc-generator-2025-10-13.md](../../records/pain-points-doc-generator-2025-10-13.md) - Pain Point #1
- [records/completion-v2-week-2-phase-2-2025-10-13.md](../../records/completion-v2-week-2-phase-2-2025-10-13.md) - Plugin priorities

---

## Week 5+ (November 2025): V2 Plugin Implementation

### 🎯 PLUGIN-001-B: Tool Context Helper Implementation

**One-liner**: Eliminate closure capture boilerplate for tools with context

**Priority**: Critical (Pain Score: 30/30 - validated across 100% of agents)
**Effort**: 2 days
**Branch**: TBD (suggest: `feat/v2-tool-context-helper`)
**Dependencies**: ✅ PLUGIN-001-A complete (2025-11-13)

**Why This Matters**:
- Pain Point #1 from both V2-AGENT-001 and V2-AGENT-002
- Affects 100% of context-aware tools (7/9 tools across both agents)
- Frequency: 3, Severity: 10, Score: 30 (CRITICAL)
- 75% reduction in boilerplate per tool

**Problem Statement**:
Every tool that needs external context (file paths, config, state) requires manual clone + move boilerplate:
```rust
.tool_fn("read_file", "Read contents", {
    let path = file_path.clone();  // ❌ Manual clone
    move |_args| read_file_tool(&path)  // ❌ Manual move
})
```

This affects 7 out of 9 tools across validated agents:
- **V2-AGENT-001**: 3/4 tools (read_file, count_lines, get_file_info)
- **V2-AGENT-002**: 4/5 tools (read_source, get_module_info, extract_public_api, count_functions)

**Solution Design**:
Extension trait providing context capture automatically:
```rust
use patinox::plugin::tool_context::ToolContextExt;

.tool_fn_with("read_file", "Read contents", &file_path,
    |path, _args| read_file_tool(path))  // ✅ Zero boilerplate
```

**Acceptance Criteria**:
- [ ] `ToolContextExt` trait implemented with `tool_fn_with()` and `tool_fn_with2()`
- [ ] Zero boilerplate for single and dual context capture
- [ ] Type-safe context access (compile-time checks)
- [ ] Works with existing tools (100% backward compatible)
- [ ] Reduces file_processor agent by ~30 lines (validated)
- [ ] Reduces doc_generator agent by ~40 lines (validated)
- [ ] Zero runtime overhead (compiles to same code as manual)
- [ ] Comprehensive tests (unit + integration)
- [ ] Example demonstrating usage

**Implementation Plan**:

**Phase 1: Extension Trait** (3-4 hours)
- Implement `ToolContextExt` trait in `src/plugin/tool_context.rs`
- `tool_fn_with<T, F>()` for single context
- `tool_fn_with2<T1, T2, F>()` for dual context
- Generic bounds: `T: Clone + Send + Sync + 'static`

**Phase 2: Integration** (2-3 hours)
- Implement trait for `Agent`
- Closure captures context automatically
- Wraps in FnTool internally

**Phase 3: Testing & Validation** (3-4 hours)
- Unit tests for single/dual context
- Integration tests with MockProvider
- Refactor file_processor.rs to use plugin
- Refactor doc_generator.rs to use plugin
- Performance validation (zero overhead)

**Files to Modify**:
- `src/plugin/tool_context.rs` - IMPLEMENT: ToolContextExt trait (currently design spec)
- `src/plugin/mod.rs` - Export ToolContextExt
- `src/lib.rs` - Add to prelude for easy import
- `examples/file_processor.rs` - REFACTOR: Use tool_fn_with
- `examples/doc_generator.rs` - REFACTOR: Use tool_fn_with

**Success Metrics**:
- [ ] 75% reduction in boilerplate (validated before/after)
- [ ] cargo test passes (all existing + new tests)
- [ ] cargo clippy passes (zero warnings)
- [ ] Both agents compile and run with plugin
- [ ] Performance identical to manual clone + move

**Unblocks**:
- PLUGIN-001-C: Tool Context Plugin Documentation
- PLUGIN-002: CLI Plugin Design
- Adoption across other agents

**See Also**:
- [planning/v2-plugin-tool-context-design.md](../../planning/v2-plugin-tool-context-design.md) - Complete design
- [records/pain-points-file-processor-2025-10-13.md](../../records/pain-points-file-processor-2025-10-13.md) - Pain analysis
- [records/pain-points-doc-generator-2025-10-13.md](../../records/pain-points-doc-generator-2025-10-13.md) - Pain analysis
- [records/completion-v2-plugin-001-design-2025-10-14.md](../../records/completion-v2-plugin-001-design-2025-10-14.md) - Design validation
- [upcoming-post-layer-2.5.md](upcoming-post-layer-2.5.md) - Full PLUGIN-001 series

---

## Metadata

**Last updated**: 2025-11-13 (PLUGIN-001-A completed, PLUGIN-001-B now ready)
**Last updated by**: Plugin foundation completion
**Total ready tasks**: 1 (PLUGIN-001-B ready for implementation)
**V2 Phase**: Layer 3 - Tool Context Helper Implementation

## Grooming Insights

**Project Health**: ✅ Excellent
- Schedule: 75% complete, ahead of plan
- Quality: Clean code reviews, zero conflicts between plan and reality
- Focus: Single critical task ready, clear sequencing for plugins

**V1 Task Archive**: 56 V1 tasks confirmed obsolete (sophisticated-first approach archived per V2 reset)

**Deferred Tasks**: 3 code quality improvements (REFACTOR-001, ARCH-001, TEST-001) - not urgent, revisit after plugin architecture stable

**Next Ready**: V2-PLUGIN-002 (CLI Plugin Design) becomes ready after V2-PLUGIN-001 completes

## Notes

This is a fresh V2 backlog. Previous V1 refinement tasks (streaming optimization, validation improvements, etc.) were archived as part of V2 strategic reset.

**V2 Principle**: Tasks are added **only after pain is felt** through real usage. We don't plan sophistication in advance.

**Pain-Driven Development Working**: V2-PLUGIN-001 emerged from validated pain across 100% of agents (7/9 tools affected, score 30/30)

See [planning/v2-week-2-plan.md](../../planning/v2-week-2-plan.md) for the emergence strategy.
