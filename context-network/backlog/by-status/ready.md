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

## Week 6+ (November 2025): V2 Plugin Implementation

### 🎯 PLUGIN-002-B: CLI Plugin Implementation

**One-liner**: Implement CLI argument parsing plugin based on approved design

**Priority**: Critical (Pain Score: 30/30 - affects 100% of CLI-based agents)
**Effort**: 2-3 hours (implementation + tests)
**Branch**: `claude/whats-the-01XRR7Sgm8RWLtG9AybHbfJ9`
**Dependencies**: ✅ PLUGIN-002-A complete (2025-11-13)

**Why This Matters**:
- Design (PLUGIN-002-A) validated to eliminate 70%+ CLI boilerplate
- file_processor: 30 lines → 8 lines (73% reduction)
- doc_generator: 35 lines → 10 lines (71% reduction)
- Pattern affects 100% of CLI-based agents

**Implementation Scope**:
Based on approved design at `context-network/planning/v2-cli-plugin-design.md`:

**Phase 1: Core Types & Builder** (30-45 mins)
- Define `CliArgs`, `ArgSpec`, `ArgBuilder` structs
- Implement builder pattern (`.arg()`, `.flag()`, `.required()`, etc.)
- Implement basic `get()` and `get_optional()` accessors

**Phase 2: Parsing Logic** (45-60 mins)
- Implement `parse()` method
- Handle `--help` / `-h` automatically
- Parse positional arguments
- Parse flags (`--output`, `-o`)
- Validate required arguments
- Apply defaults for optional args

**Phase 3: Help Generation** (20-30 mins)
- Auto-generate help text from arg specs
- Format output cleanly with USAGE, ARGUMENTS, OPTIONS sections
- Include examples (optional)

**Phase 4: Testing** (30-45 mins)
- Unit tests for argument parsing
- Unit tests for validation errors
- Unit tests for help generation
- Integration tests with example scenarios

**Acceptance Criteria**:
- [ ] `src/cli.rs` module created with full implementation
- [ ] All core types implemented (CliArgs, ArgSpec, ArgBuilder, CliError)
- [ ] Builder API works: `.arg().required()`, `.optional()`, `.default()`, `.flag()`
- [ ] Parsing handles positional args and flags correctly
- [ ] `--help` automatically shows generated help text
- [ ] Missing required args produce clear error messages
- [ ] cargo test passes (all new CLI tests)
- [ ] cargo clippy passes (zero warnings)
- [ ] cargo doc builds cleanly

**Files to Create**:
- `src/cli.rs` - Main module file
- `src/cli/args.rs` - CliArgs struct and implementation
- `src/cli/builder.rs` - ArgBuilder for fluent API
- `src/cli/error.rs` - CliError types
- `src/cli/help.rs` - Help text generation (optional - can inline)

**Files to Modify**:
- `src/lib.rs` - Export `pub mod cli;`

**Test Coverage Required**:
- Parse simple positional argument
- Parse multiple positional arguments
- Parse optional argument with default
- Parse flags (`--output`, `-o`)
- Validate missing required argument error
- Validate unknown flag error
- Generate help text correctly
- Handle `--help` flag

**Validation Strategy**:
1. Implement core types and builder
2. Run `cargo build` to verify types compile
3. Implement parsing logic
4. Add unit tests for parsing
5. Implement help generation
6. Add unit tests for help text
7. Run full test suite: `cargo test`
8. Run clippy: `cargo clippy`
9. Build docs: `cargo doc`

**Success Metrics**:
- [ ] All tests pass
- [ ] Zero clippy warnings
- [ ] Documentation builds cleanly
- [ ] API matches design specification
- [ ] Ready for migration (PLUGIN-002-C)

**Unblocks**:
- PLUGIN-002-C: CLI Plugin Documentation (migration of examples)

**Design Reference**:
See [planning/v2-cli-plugin-design.md](../../planning/v2-cli-plugin-design.md) for complete API specification and design decisions.

**See Also**:
- [planning/v2-cli-plugin-design.md](../../planning/v2-cli-plugin-design.md) - Complete design spec
- [records/pain-points-file-processor-2025-10-13.md](../../records/pain-points-file-processor-2025-10-13.md) - Pain Point #2
- [records/pain-points-doc-generator-2025-10-13.md](../../records/pain-points-doc-generator-2025-10-13.md) - Pain Point #2

---

## Metadata

**Last updated**: 2025-11-13 (PLUGIN-002-A complete, PLUGIN-002-B now ready)
**Last updated by**: Post-PLUGIN-002-A completion
**Total ready tasks**: 1 (PLUGIN-002-B ready for implementation)
**V2 Phase**: Layer 3 - CLI Plugin Implementation

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
