# Task: Decompose Large Provider Modules for Better Maintainability

## Overview
**Priority**: Medium
**Effort**: Medium (30-60 minutes)
**Risk**: Medium
**Created**: 2025-08-20
**Source**: Code review recommendations

## Problem Statement

Several provider modules have grown beyond optimal size for maintainability:
- `src/provider/openrouter.rs`: 685 lines
- `src/traits/mod.rs`: 1,010 lines  
- `src/provider/openai.rs`: 513 lines

Large files reduce readability and make the codebase harder to navigate and maintain.

## Acceptance Criteria

### OpenRouter Module Decomposition
- [ ] Split `src/provider/openrouter.rs` into focused submodules:
  - [ ] `client.rs` - HTTP client and request handling
  - [ ] `conversion.rs` - Format conversion logic (OpenAI ↔ OpenRouter)
  - [ ] `types.rs` - OpenRouter-specific request/response types
  - [ ] `mod.rs` - Re-exports and main provider struct
- [ ] Maintain existing public API (no breaking changes)
- [ ] All 20+ OpenRouter tests continue passing
- [ ] Documentation reflects new module structure

### Trait Module Organization  
- [ ] Split `src/traits/mod.rs` into focused modules:
  - [ ] `agent.rs` - Agent trait and related types
  - [ ] `tool.rs` - Tool execution trait
  - [ ] `validator.rs` - Validation trait
  - [ ] `monitor.rs` - Monitoring trait
  - [ ] `mod.rs` - Re-exports and shared types
- [ ] Preserve all public exports
- [ ] All 85+ trait tests continue passing

### Optional: OpenAI Module (if time permits)
- [ ] Consider splitting `src/provider/openai.rs` using similar pattern

## Technical Approach

### Phase 1: OpenRouter Module Split
1. Create `src/provider/openrouter/` directory
2. Move HTTP client logic to `client.rs`
3. Extract conversion functions to `conversion.rs`
4. Move OpenRouter-specific types to `types.rs`
5. Update `mod.rs` with appropriate re-exports
6. Update imports in tests and other modules

### Phase 2: Trait Module Organization
1. Create individual trait files under `src/traits/`
2. Move trait definitions and implementations
3. Ensure proper re-exports in `mod.rs`
4. Update documentation

### Module Structure Example
```rust
// src/provider/openrouter/mod.rs
mod client;
mod conversion;
mod types;

pub use client::OpenRouterProvider;
pub use types::{OpenRouterRequest, OpenRouterResponse, OpenRouterMessage};

// src/traits/mod.rs
mod agent;
mod tool;
mod validator;
mod monitor;

pub use agent::{Agent, AgentConfig, AgentState};
pub use tool::{Tool, ToolCall, ToolResult};
// ... etc
```

## Success Metrics
- File sizes reduced to <400 lines each
- Zero test failures after refactoring
- No breaking changes to public API
- Improved code navigation and discoverability
- Cleaner import statements

## Dependencies
- Requires understanding of existing module structure
- Should coordinate with any ongoing provider development
- Consider impact on IDE navigation and autocomplete

## Risks and Mitigations
- **Risk**: Breaking existing imports
- **Mitigation**: Maintain all public re-exports, run full test suite

- **Risk**: Circular dependencies between modules
- **Mitigation**: Careful design of module boundaries and dependencies

## Implementation Notes
- Follow existing code organization patterns
- Maintain consistent documentation style
- Consider future extensibility when designing module boundaries
- Use `pub(crate)` for internal APIs to maintain encapsulation

## Related Tasks
- None currently

## Validation Checklist
- [ ] All tests pass (175+ tests)
- [ ] No clippy warnings introduced
- [ ] Documentation builds successfully
- [ ] Public API unchanged (no semver breakage)
- [ ] Import statements remain clean and logical