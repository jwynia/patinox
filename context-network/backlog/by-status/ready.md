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

## Week 4: Layer 2.5 - Lifecycle Hook Architecture

### 📅 V2-ARCH-001: Implement Lifecycle Hook Infrastructure

**One-liner**: Add 6-hook lifecycle architecture to enable future middleware without premature implementation

**Priority**: High (prevents architectural regret, validated by external experience)
**Effort**: 2-3 days (trait definition + integration + tests + examples)
**Branch**: `feat/v2-lifecycle-hooks`
**Timeline**: Week 4 (October 24-31, 2025)

**Why This Matters**:
- Project lead has validated pain from external agent framework production use
- LangChain V1 middleware validates industry need for 6 hook points
- Adding now prevents costly refactoring later (current `run()` is monolithic)
- Trait-only approach aligns with minimal-first (no implementations yet)
- Zero runtime cost if unused (opt-in, default passthroughs)

**Problem Statement**:
Agent execution has 6 natural intervention points where middleware is needed:
1. `before_agent` - Input validation, rate limiting, context loading
2. `before_model` - Context window management, prompt injection
3. `wrap_model_call` - Retry logic, fallback providers, telemetry
4. `after_model` - HITL approval, safety validation
5. `wrap_tool_call` - Tool retry, audit logging, permissions
6. `after_agent` - Result persistence, metrics, notifications

Without hook infrastructure, adding these later requires refactoring core `Agent::run()` method.

**Acceptance Criteria**:
- [ ] `AgentLifecycle` trait defined with all 6 hooks (default passthroughs)
- [ ] Agent supports hook registration via `.with_lifecycle(hook)`
- [ ] `run()` method calls hooks when present (fast path if empty)
- [ ] All existing tests pass (zero regression)
- [ ] Hook execution order validated (integration test)
- [ ] Performance benchmarks: < 5% overhead with 1 hook, < 10% with 5 hooks
- [ ] Example showing hook usage pattern
- [ ] Rustdoc complete for all hooks

**Design Decisions**:
See [decisions/lifecycle-hook-architecture.md](../../decisions/lifecycle-hook-architecture.md) for full design.

**Files to Create/Modify**:
- `src/lifecycle.rs` - NEW: AgentLifecycle trait + HookAction enum (~150 lines)
- `src/agent.rs` - MODIFY: Add lifecycle vec, with_lifecycle(), hook calling (~50 lines added)
- `src/lib.rs` - MODIFY: Export lifecycle module
- `examples/lifecycle_hooks.rs` - NEW: Example hook implementations (~100 lines)
- `benches/hook_overhead.rs` - NEW: Performance benchmarks

**Implementation Plan**:

**Day 1-2: Trait Definition & Integration**
1. Define `AgentLifecycle` trait with all 6 hooks
2. Add `HookAction` enum (Continue, Approve, Reject, Modify)
3. Update `Agent` struct with `lifecycle: Vec<Arc<dyn AgentLifecycle>>`
4. Implement `.with_lifecycle()` builder method
5. Add helper methods for calling hook chains

**Day 2-3: Hook Calling in run() Method**
1. Add `before_agent` hook call before processing
2. Add `before_model` hook call before provider
3. Add `wrap_model_call` wrapper with hook chain
4. Add `after_model` hook call after response
5. Add `wrap_tool_call` wrapper in tool execution
6. Add `after_agent` hook call before return
7. Optimize fast path (empty lifecycle vec)

**Day 3: Testing & Examples**
1. Unit tests for each hook's default implementation
2. Integration test for hook execution order
3. Regression tests (all existing tests pass)
4. Performance benchmarks (overhead targets)
5. Example: logging hook, retry hook, HITL mock

**V1 Import Path**:
- V1 Tower middleware patterns can be imported as `AgentLifecycle` implementations
- V1 MAPE-K monitoring becomes hook suite in Layer 4
- V1 async HITL becomes `after_model` hook in Layer 4

**Validation**:
- External production experience confirms need for all 6 hooks
- LangChain V1 chose identical hook points (industry validation)
- Adding trait now is cheap insurance against refactoring later

**Concrete Hooks Deferred to Layer 3**:
- Retry logic (when API reliability becomes pain)
- HITL approval (when safety becomes requirement)
- Context trimming (when token limits hit)
- Telemetry (when debugging becomes painful)

**See Also**:
- [decisions/lifecycle-hook-architecture.md](../../decisions/lifecycle-hook-architecture.md) - Full design
- [planning/lifecycle-hook-use-cases.md](../../planning/lifecycle-hook-use-cases.md) - 30+ use cases cataloged
- [planning/roadmap.md](../../planning/roadmap.md) - Layer 2.5 timeline
- [planning/layer-2.5-implementation/task-breakdown.md](../../planning/layer-2.5-implementation/task-breakdown.md) - Complete 17-task implementation breakdown

---

## Metadata

**Last updated**: 2025-10-16 (V2-ARCH-001 added for Layer 2.5)
**Last updated by**: Lifecycle Hook Architecture Planning
**Total ready tasks**: 1 (V2-ARCH-001 ready for Week 4)
**V2 Phase**: Layer 2.5 - Lifecycle Hook Architecture

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
