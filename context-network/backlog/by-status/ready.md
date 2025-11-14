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

### 🎯 PLUGIN-002-A: CLI Plugin Design

**One-liner**: Design CLI argument handling plugin

**Priority**: Critical (Pain Score: 30/30 - affects 100% of CLI-based agents)
**Effort**: 1 day
**Branch**: TBD (suggest: `design/v2-cli-plugin`)
**Dependencies**: ✅ PLUGIN-001 series complete (2025-11-13)

**Why This Matters**:
- Pain Point #2 from both V2-AGENT-001 and V2-AGENT-002
- Affects 100% of CLI-based agents
- Manual CLI parsing requires ~30 lines per agent
- Same boilerplate repeated in every example

**Problem Statement**:
Every CLI-based agent requires extensive manual argument parsing:
```rust
let args: Vec<String> = std::env::args().collect();

if args.len() < 2 {
    print_usage(&args[0]);
    std::process::exit(1);
}

// Check for special flags
if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
    print_usage(&args[0]);
    return Ok(());
}

let file_path = &args[1];
let user_query = if args.len() > 2 {
    args[2..].join(" ")
} else {
    format!("Default query...")
};
```

This affects both validated agents:
- **V2-AGENT-001** (file_processor): 30 lines of CLI boilerplate
- **V2-AGENT-002** (doc_generator): 35 lines of CLI boilerplate

**Proposed Solution**:
Plugin-based CLI handling with automatic parsing and help generation:
```rust
use patinox::plugin::cli::CliPlugin;

let agent = create_agent("my-agent")
    .with_plugin(CliPlugin::new()
        .arg("file_path", "Path to file to process")
            .required()
        .arg("query", "Query to run (optional)")
            .optional()
            .default("Analyze this file")
        .flag("verbose", "Enable verbose output")
        .flag("help", "Show this help message")
    );
```

**Design Goals**:
1. **Eliminate boilerplate**: Reduce CLI parsing from ~30 lines to ~5-10 lines
2. **Type safety**: Compile-time argument type checking
3. **Auto help**: Generate `--help` output automatically from arg definitions
4. **Error handling**: Clear error messages for missing/invalid arguments
5. **Backward compatible**: Agents without CLI plugin work unchanged

**Acceptance Criteria**:
- [ ] Design document created with full API specification
- [ ] Covers common CLI patterns (args, flags, optional args, defaults)
- [ ] Type-safe argument parsing approach defined
- [ ] Automatic help generation design
- [ ] Integration with Agent builder pattern
- [ ] Migration examples (before/after)
- [ ] Design validated against both example agents

**Deliverables**:
- Design document (`context-network/planning/v2-cli-plugin-design.md`)
- API specification with examples
- Integration approach (how it works with Agent)
- Migration guide (file_processor and doc_generator examples)

**Key Design Questions to Answer**:
1. **API Shape**: Extension trait vs Plugin trait vs both?
2. **Argument Access**: How does agent/tool code access parsed args?
3. **Type System**: How to support String, PathBuf, i32, bool, etc.?
4. **Error Handling**: How to report missing/invalid arguments?
5. **Help Generation**: How to auto-generate --help output?

**Unblocks**:
- PLUGIN-002-B: CLI Plugin Implementation
- PLUGIN-002-C: CLI Plugin Documentation

**See Also**:
- [records/pain-points-file-processor-2025-10-13.md](../../records/pain-points-file-processor-2025-10-13.md) - Pain Point #2
- [records/pain-points-doc-generator-2025-10-13.md](../../records/pain-points-doc-generator-2025-10-13.md) - Pain Point #2
- [upcoming-post-layer-2.5.md](upcoming-post-layer-2.5.md) - Full PLUGIN-002 series

---

## Metadata

**Last updated**: 2025-11-13 (PLUGIN-001 series complete, PLUGIN-002-A now ready)
**Last updated by**: Post-PLUGIN-001-C completion
**Total ready tasks**: 1 (PLUGIN-002-A ready for design)
**V2 Phase**: Layer 3 - CLI Plugin Design

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
