# Upcoming: Post-Layer 2.5 Work

## Purpose
Tasks that will become ready after Layer 2.5 (Lifecycle Hooks) completion.

## Current Status
- **Layer 2.5**: In planning (implementation Week 4: Oct 24-31)
- **Next Phase**: Month 2 - Plugin completion + First hooks
- **Timeline**: November 2025 onwards

---

## Week 5 (Nov 1-7): V2-PLUGIN-001 Implementation

### PLUGIN-001-A: Implement Plugin Trait Foundation

**One-liner**: Create base plugin trait and integration architecture

**Priority**: Critical
**Effort**: 1 day
**Dependencies**: Layer 2.5 complete

**Deliverables**:
- `src/plugin/mod.rs` - Plugin trait definition
- Builder integration points
- Documentation for plugin authors

**Acceptance Criteria**:
- [ ] Plugin trait defined with clear lifecycle
- [ ] Agent can register plugins via `.with_plugin()`
- [ ] Example plugin compiles
- [ ] Rustdoc complete

---

### PLUGIN-001-B: Tool Context Helper Implementation

**One-liner**: Eliminate closure capture boilerplate for tools with context

**Priority**: Critical (Pain Score: 30/30)
**Effort**: 2 days
**Dependencies**: PLUGIN-001-A complete

**Problem**: Currently requires 3 lines of boilerplate per tool:
```rust
let path = file_path.clone();
move |_args| read_file_tool(&path)
```

**Solution**: Plugin provides context automatically:
```rust
.tool_with_context("read_file", "Read file", |ctx, args| {
    read_file_tool(&ctx.file_path, args)
})
```

**Deliverables**:
- `src/plugin/tool_context.rs` - Tool context plugin
- Integration with `.tool_fn()` builder
- Tests for context capture
- Examples (before/after)

**Acceptance Criteria**:
- [ ] Zero boilerplate for context capture
- [ ] Type-safe context access
- [ ] Works with existing tools (backward compatible)
- [ ] Reduces file_processor agent by ~30 lines
- [ ] Reduces doc_generator agent by ~40 lines

**Validation**: Refactor both test agents to use plugin

---

### PLUGIN-001-C: Tool Context Plugin Documentation

**One-liner**: Comprehensive docs and examples for tool context plugin

**Priority**: High
**Effort**: 0.5 days
**Dependencies**: PLUGIN-001-B complete

**Deliverables**:
- Rustdoc for plugin API
- Example: file processor with tool context
- Example: custom context type
- Migration guide (manual → plugin)

**Acceptance Criteria**:
- [ ] `cargo doc` builds cleanly
- [ ] 2+ examples compile and run
- [ ] Migration guide clear
- [ ] Plugin benefits articulated

---

## Week 6 (Nov 8-14): V2-PLUGIN-002 Implementation

### PLUGIN-002-A: CLI Plugin Design

**One-liner**: Design CLI argument handling plugin

**Priority**: Critical (Pain Score: 30/30)
**Effort**: 1 day
**Dependencies**: PLUGIN-001 complete (establishes plugin patterns)

**Problem**: Manual CLI parsing requires ~30 lines per agent:
```rust
let args: Vec<String> = env::args().collect();
if args.len() < 2 {
    eprintln!("Usage: ...");
    return;
}
let file_path = &args[1];
```

**Solution**: Plugin handles CLI automatically:
```rust
create_agent("my-agent")
    .with_cli_arg("file_path", "Path to file", ArgType::String, true)
    .with_cli_flag("verbose", "Verbose output", false)
```

**Deliverables**:
- Design document
- API specification
- Integration approach
- Migration examples

**Acceptance Criteria**:
- [ ] Design covers common CLI patterns
- [ ] Type-safe argument parsing
- [ ] Automatic help generation
- [ ] Backward compatible

---

### PLUGIN-002-B: CLI Plugin Implementation

**One-liner**: Implement CLI argument handling plugin

**Priority**: Critical
**Effort**: 2-3 days
**Dependencies**: PLUGIN-002-A complete

**Deliverables**:
- `src/plugin/cli.rs` - CLI plugin implementation
- Integration with agent builder
- Tests for argument parsing
- Error handling for invalid args

**Acceptance Criteria**:
- [ ] Supports strings, integers, booleans, paths
- [ ] Required vs optional arguments
- [ ] Flags (boolean options)
- [ ] Automatic `--help` generation
- [ ] Good error messages
- [ ] Reduces CLI boilerplate from 30 lines to ~5 lines

**Validation**: Refactor both test agents to use CLI plugin

---

### PLUGIN-002-C: CLI Plugin Documentation

**One-liner**: Examples and guides for CLI plugin

**Priority**: High
**Effort**: 0.5 days
**Dependencies**: PLUGIN-002-B complete

**Deliverables**:
- Rustdoc for CLI plugin API
- Example: simple CLI agent
- Example: complex multi-arg agent
- Best practices guide

**Acceptance Criteria**:
- [ ] Documentation complete
- [ ] Examples clear and runnable
- [ ] Common patterns documented

---

## Week 7 (Nov 15-21): Real Usage Wave 2

### USAGE-WAVE-2-A: Build Git Helper Agent

**One-liner**: Agent to analyze git repos and suggest improvements

**Priority**: High
**Effort**: 1-2 days
**Dependencies**: PLUGIN-001, PLUGIN-002 complete

**Tools Needed**:
- git_status - Get repo status
- git_diff - Get uncommitted changes
- git_log - Get commit history
- read_file - Read source files
- write_report - Save analysis report

**Expected Pain**:
- Process execution (git commands)
- Output parsing (git output formats)
- Multi-file analysis (iterate files)

**Success Criteria**:
- [ ] Agent runs against real repos
- [ ] Produces useful suggestions
- [ ] Pain points documented
- [ ] New pain categories identified

---

### USAGE-WAVE-2-B: Build Web Scraper Agent

**One-liner**: Extract structured data from websites

**Priority**: High
**Effort**: 1-2 days
**Dependencies**: PLUGIN-001, PLUGIN-002 complete

**Tools Needed**:
- fetch_url - HTTP GET request
- parse_html - Extract data from HTML
- extract_data - Pattern matching
- save_json - Save structured output

**Expected Pain**:
- HTTP requests (async, error handling)
- HTML parsing (DOM traversal)
- Rate limiting (be nice to servers)

**Success Criteria**:
- [ ] Agent scrapes real websites
- [ ] Extracts clean data
- [ ] Pain points documented
- [ ] HTTP/parsing pain validated

---

### USAGE-WAVE-2-C: Pain Point Analysis

**One-liner**: Analyze pain from Wave 2 agents, prioritize next work

**Priority**: Critical
**Effort**: 1 day
**Dependencies**: USAGE-WAVE-2-A, USAGE-WAVE-2-B complete

**Activities**:
1. Document all pain points
2. Score: Frequency × Severity
3. Compare to existing plugins
4. Identify new categories

**Deliverables**:
- Pain point analysis document
- Updated pain point matrix
- Next plugin recommendation
- OR: First hook recommendation (if hooks more critical)

**Decision**: Continue Layer 2 (more plugins) OR move to Layer 3 (hooks)?

**Success Criteria**:
- [ ] All pain quantified
- [ ] Next priority clear
- [ ] Data-driven decision

---

## Week 9 (Dec 1-7): V2-HOOK-001 Retry Hook

### HOOK-001-A: Retry Hook Design

**One-liner**: Design retry logic for transient API failures

**Priority**: High
**Effort**: 0.5 days
**Dependencies**: Layer 2.5 complete

**Use Cases**:
- 429 Rate limit errors
- 503 Service unavailable
- Network timeouts
- Transient provider issues

**Design Decisions**:
- Max retry attempts (configurable)
- Backoff strategy (exponential, linear, custom)
- Which errors to retry (configurable)
- Logging of retry attempts

**Deliverables**:
- Design document
- API specification
- Configuration approach

---

### HOOK-001-B: Retry Hook Implementation

**One-liner**: Implement retry logic as lifecycle hook

**Priority**: High
**Effort**: 2-3 days
**Dependencies**: HOOK-001-A complete

**Implementation**:
- `src/hooks/retry.rs` - Retry hook
- Implements `wrap_model_call` and `wrap_tool_call`
- Exponential backoff with jitter
- Configurable retry conditions

**Acceptance Criteria**:
- [ ] Retries transient failures (429, 503)
- [ ] Exponential backoff working
- [ ] Max attempts respected
- [ ] Logs retry attempts
- [ ] Zero overhead when not used
- [ ] Example showing retry in action

**Validation**: Test with real provider (trigger rate limits)

---

### HOOK-001-C: Retry Hook Documentation

**One-liner**: Document retry hook usage and configuration

**Priority**: Medium
**Effort**: 0.5 days
**Dependencies**: HOOK-001-B complete

**Deliverables**:
- Rustdoc for retry hook
- Configuration guide
- Example: retry on rate limits
- Best practices (backoff strategies)

---

## Month 4-5 (Jan-Feb 2026): Layer 3 Bicameral Mind

### PATTERN-001-A: Bicameral Critic Hook Design

**One-liner**: Design creator-critic separation for quality improvement

**Priority**: Critical
**Effort**: 3 days
**Dependencies**: Hooks proven in production

**Pattern**: Separate creator (generates) from critic (evaluates)
- Creator model produces output
- Critic model evaluates with fresh perspective
- Critic can Continue, Reject, or Modify

**Use Cases**:
- Code review (creator writes code, critic reviews)
- Content writing (creator drafts, critic edits)
- Decision making (creator proposes, critic evaluates)

**Deliverables**:
- Design document
- API specification
- Validation approach

---

### PATTERN-001-B: Bicameral Critic Implementation

**One-liner**: Implement creator-critic hook

**Priority**: Critical
**Effort**: 5-7 days
**Dependencies**: PATTERN-001-A complete

**Implementation**:
- `src/patterns/bicameral.rs` - Bicameral mind pattern
- Simple critic hook (one-shot evaluation)
- Iterative refinement hook (multi-round)
- Configurable critic prompts

**Acceptance Criteria**:
- [ ] Critic evaluates creator output
- [ ] Separate model/context for critic
- [ ] Continue/Reject/Modify actions work
- [ ] Measurable quality improvement
- [ ] Examples: code review, content editing

**Validation**: A/B test creator-only vs bicameral on same tasks

---

### PATTERN-001-C: Bicameral Refinement Loop

**One-liner**: Multi-round creator→critic→refine loop

**Priority**: High
**Effort**: 3-5 days
**Dependencies**: PATTERN-001-B complete

**Implementation**:
- Implements `wrap_model_call` for iterative refinement
- Creator → Critic → Refine loop
- Configurable max rounds
- Early termination on approval

**Acceptance Criteria**:
- [ ] Multiple rounds work
- [ ] Converges to approval
- [ ] Quality improves each round
- [ ] Terminates correctly

---

## Month 6+ (Mar 2026): Layer 4 V1 Imports

### IMPORT-001: V1 Testing Utilities

**One-liner**: Import proven testing patterns from V1

**Priority**: Medium
**Effort**: 1-2 weeks
**Dependencies**: Layer 3 proven

**What to Import**:
- Mock builders
- Test fixtures
- HTTP mocking patterns
- Assertion helpers

**Approach**:
- Simplify during import
- Adapt to V2 architecture
- Comprehensive tests

---

### IMPORT-002: Tower Patterns as Hooks

**One-liner**: Import V1 Tower middleware as AgentLifecycle implementations

**Priority**: High
**Effort**: 3-4 weeks
**Dependencies**: IMPORT-001 complete

**What to Import**:
- Tower validation layers
- Middleware composition
- Circuit breaker patterns
- Rate limiting

**Mapping**:
- Tower layers → AgentLifecycle hooks
- Middleware stack → Hook chain
- Service trait → Provider trait

---

## Metadata

**Created**: 2025-10-16
**Scope**: November 2025 - March 2026
**Total Tasks**: 20+ (scoped to upcoming work)
**Next Review**: After Layer 2.5 completion
**Status**: PLANNED - Becomes ready incrementally

---

## Notes

**Pain-Driven Approach**: These tasks are projected based on current understanding. Reality may differ:
- Some tasks may not be needed (pain doesn't materialize)
- New tasks may emerge (unexpected pain discovered)
- Priorities may shift (usage reveals different critical paths)

**Flexibility**: This is a guide, not a contract. Each decision point allows pivots based on real data.

**Migration from ready.md**: As Layer 2.5 completes, specific tasks move from this file to ready.md with updated status.
