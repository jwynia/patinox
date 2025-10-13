# REFACTOR-001: Split doc_generator.rs to Meet 300-Line Guideline

## Status
- **Created**: 2025-10-13
- **Priority**: Low
- **Effort**: Medium (30-45 minutes)
- **Type**: Code Quality / Refactoring

## Context

**Source**: PR #21 Code Review
**Current State**: `examples/doc_generator.rs` is 308 lines (8 lines over 300-line guideline per CLAUDE.md)
**Review Note**: "Not blocking, still readable and cohesive"

## Problem Statement

The doc_generator example exceeds the project's 300-line guideline by 8 lines. While not critical, maintaining this guideline helps keep code modular and maintainable.

## Acceptance Criteria

- [ ] Split `examples/doc_generator.rs` into two files
- [ ] Main file (`doc_generator.rs`) stays under 300 lines - contains main(), CLI parsing, agent setup
- [ ] Tool implementations moved to separate module (e.g., `doc_tools.rs` or within the example)
- [ ] All functionality preserved - no behavior changes
- [ ] Tests still pass (cargo test)
- [ ] No clippy warnings
- [ ] Example still runs correctly

## Proposed Approach

**Option A: Helper Module in examples/**
```rust
// examples/doc_generator.rs (~150 lines)
mod doc_tools;  // examples/doc_tools.rs

use doc_tools::*;

fn main() { ... }
```

**Option B: Inline Module**
```rust
// examples/doc_generator.rs
mod tools {
    // Tool implementations
}

fn main() {
    use tools::*;
    // ...
}
```

**Recommendation**: Option A (separate file) for clarity

## Files to Modify

- `examples/doc_generator.rs` - Keep main(), CLI, agent setup (~150 lines)
- `examples/doc_tools.rs` (new) - Move tool implementations (~150 lines):
  - `read_source_tool()`
  - `get_module_info_tool()`
  - `extract_public_api_tool()`
  - `extract_item_name()`
  - `count_functions_tool()`
  - `write_documentation_tool()`

## Testing Plan

1. Verify compilation: `cargo build --example doc_generator`
2. Run clippy: `cargo clippy --example doc_generator`
3. Test help: `cargo run --example doc_generator -- --help`
4. Test functionality (with API key): Document a real file

## Why Deferred (Not Applied Immediately)

1. **Medium Effort**: Requires file split, import updates, testing
2. **Not Urgent**: Only 8 lines over, review said "not blocking"
3. **V2 Phase**: Currently in Week 2 (pain discovery), not refactoring phase
4. **Learning Example**: Keeping it cohesive may be pedagogically better
5. **Future Architecture**: Week 3 plugins may change example structure anyway

## Dependencies

- None (independent refactoring)

## Related Work

- Consider also checking `examples/file_processor.rs` (214 lines - well under limit)
- After Week 3 plugin work, review if examples need updates
- May want to establish patterns for future agent examples

## Notes

- This is a code quality improvement, not a bug fix
- The 300-line guideline is a soft guideline, not a hard rule
- May be superseded by plugin architecture changes in Week 3
- Consider if this refactoring provides value given V2's learning focus

## Recommendation

**Priority**: LOW - Defer until after Week 3 plugin implementation

**Rationale**: Better to focus on plugin development (high-impact) than style refactoring (low-impact) during V2 critical path. Can revisit as part of broader example cleanup later.
