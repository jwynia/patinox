# Coding Standards - Patinox

## Purpose
Document the coding standards and quality expectations for the Patinox codebase. These standards emerged from real development experience and are designed to keep code maintainable, testable, and understandable.

## Classification
- **Domain:** Meta/Process
- **Stability:** Evolving (updated as we learn)
- **Abstraction:** Standards
- **Confidence:** High (based on practical experience)

## Core Principles

### 1. File Size as a Code Smell

**Standard:** Keep files focused and manageable.

**Limits:**
- **Target:** 100-200 lines per file
- **Maximum:** 300 lines (including tests)
- **Breaking point:** 500+ lines indicates poor separation of concerns

**Why it matters:**
- Large files are hard to navigate and understand
- Indicates multiple responsibilities (SRP violation)
- Makes code review difficult
- Suggests missing abstractions

**When to split:**
```
WARNING SIGNS:
- File > 300 lines
- Multiple distinct concepts in one file
- Scrolling required to understand structure
- Tests are larger than implementation
- Adding features requires extensive context
```

**How to split:**
1. Identify distinct concepts/responsibilities
2. Extract to separate modules
3. Create clear public API via mod.rs
4. Move tests with their implementations

**Example (from provider.rs refactoring):**
```
BEFORE: provider.rs (567 lines)
  - 235 lines implementation
  - 332 lines tests

AFTER: provider/ module
  - mod.rs (160 lines) - Public API
  - openai.rs (168 lines) - Implementation
  - mock.rs (38 lines) - Test utilities
```

### 2. Module Organization (Rust-Specific)

**Standard:** Use Rust's module system to enforce boundaries.

**Structure patterns:**

**Single-file module (simple):**
```rust
src/
  simple_feature.rs  // < 200 lines, self-contained
```

**Multi-file module (complex):**
```rust
src/
  complex_feature/
    mod.rs           // Public API, re-exports
    core.rs          // Core implementation
    utils.rs         // Helper functions
    tests.rs         // Unit tests (optional)
```

**With provider pattern:**
```rust
src/
  feature/
    mod.rs           // Trait definition, public types
    provider_a.rs    // Implementation A
    provider_b.rs    // Implementation B
    mock.rs          // Test utilities
```

**Guidelines:**
- One concept per file
- Public API defined in mod.rs
- Implementation details in separate files
- Tests co-located with code (inline or separate tests.rs)

### 3. Test Organization

**Standard:** Tests should be focused and maintainable.

**Where tests go:**

**Small test suites (< 50 lines):**
```rust
// Inline with implementation
#[cfg(test)]
mod tests {
    use super::*;
    // Tests here
}
```

**Medium test suites (50-100 lines):**
```rust
// Separate module file
// src/feature/tests.rs
use super::*;
// Tests here
```

**Large test suites (100+ lines):**
```rust
// Integration tests directory
// tests/feature_tests.rs
use patinox::feature::*;
// Tests here
```

**When tests outgrow the file:**
- Extract to tests.rs in same module
- Split by test category (creation_tests.rs, validation_tests.rs)
- Consider if implementation needs refactoring too

See [testing-philosophy.md](./testing-philosophy.md) for test quality standards.

### 4. Code Duplication

**Standard:** Don't Repeat Yourself (with judgment).

**When to extract:**
- Logic appears 3+ times
- Pattern is stable and unlikely to diverge
- Abstraction makes code clearer

**When NOT to extract:**
- Only 2 instances (wait for third)
- Logic might diverge in future
- Abstraction obscures intent
- Premature optimization

**Rule of thumb:** Duplication is cheaper than wrong abstraction.

### 5. Naming Conventions

**Standard:** Names should be self-documenting.

**Files:**
- `snake_case.rs` for Rust files
- Descriptive, not abbreviated: `provider.rs` not `prov.rs`
- Match module name exactly

**Functions:**
- Verbs for actions: `create_agent()`, `validate_config()`
- Clear intent: `complete()` not `do_thing()`
- No abbreviations: `configuration` not `cfg`

**Types:**
- `PascalCase` for types
- Descriptive: `ProviderConfig` not `Config`
- Trait suffix for traits: `LLMProvider`, `Validator`

**Variables:**
- `snake_case` for variables
- Descriptive: `api_key` not `key`
- Avoid single letters except loops: `i`, `j`, `k` OK

### 6. Function Length

**Standard:** Functions should do one thing well.

**Guidelines:**
- Target: 10-20 lines
- Maximum: 50 lines
- If > 50 lines, consider extracting helper functions

**Signs of too long:**
- Multiple levels of nesting (> 3)
- Needs comments to explain sections
- Can't understand at a glance
- Multiple return points scattered throughout

**Refactoring approach:**
```rust
// BEFORE: Long function
fn process_data(data: Vec<Data>) -> Result<Output> {
    // 100 lines of logic
}

// AFTER: Composed from smaller functions
fn process_data(data: Vec<Data>) -> Result<Output> {
    let validated = validate_data(data)?;
    let transformed = transform_data(validated)?;
    let result = aggregate_results(transformed)?;
    Ok(result)
}
```

### 7. Error Handling

**Standard:** Errors should be clear and actionable.

**Requirements:**
- Use `Result<T, E>` for fallible operations
- Custom error types when needed
- Context in error messages
- No silent failures

**Error message quality:**
```rust
// BAD: Vague error
Err("Failed")?

// GOOD: Specific, actionable error
Err("OPENAI_API_KEY is required but not set. Set the environment variable or provide in config.")?
```

### 8. Comments and Documentation

**Standard:** Code should be self-documenting; comments explain why, not what.

**When to comment:**
- Non-obvious design decisions
- Performance trade-offs
- Security considerations
- Workarounds for external bugs

**When NOT to comment:**
```rust
// BAD: Obvious comment
// Increment counter
counter += 1;

// GOOD: Self-documenting code
counter += 1;  // No comment needed
```

**Documentation requirements:**
- Public API must have doc comments
- Module-level documentation in mod.rs
- Examples for complex functionality
- Link to relevant context network docs

### 9. Async Code Standards

**Standard:** Async code should be explicit and testable.

**Guidelines:**
- Mark functions `async` only when needed
- Don't block async runtime (no `.unwrap()` in production)
- Test async code with `#[tokio::test]`
- Provide sync wrappers for CLI/blocking contexts

**Example:**
```rust
// Async implementation
#[async_trait::async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: Vec<Message>) -> Result<String>;
}

// Sync wrapper for CLI
pub fn run_cli(agent: Agent) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async_run_cli(agent))
}
```

## Rust-Specific Standards

### 1. Error Types

**Standard:** Use appropriate error types for context.

**Options:**
- `Box<dyn Error>` for simple cases
- `thiserror` for library errors
- `anyhow` for application errors

**Current approach:** `Box<dyn Error + Send + Sync>` for flexibility

### 2. Trait Bounds

**Standard:** Explicit is better than implicit.

**Requirements:**
- Specify `Send + Sync` for async traits
- Document thread safety requirements
- Use `?Sized` when appropriate

### 3. Ownership Patterns

**Standard:** Prefer borrowing, own when necessary.

**Guidelines:**
- Pass by reference when possible: `&str` not `String`
- Use `Into<T>` for flexible APIs
- Clone only when needed, document why

## Quality Checklist

Before committing code, verify:
- [ ] No file exceeds 300 lines
- [ ] Each file has single, clear purpose
- [ ] Functions are < 50 lines
- [ ] Names are self-documenting
- [ ] Errors are clear and actionable
- [ ] Tests are focused and fast
- [ ] Public API is documented
- [ ] No compiler warnings
- [ ] `cargo clippy` passes
- [ ] `cargo fmt` applied

## Enforcement

**Automatic:**
- `cargo fmt` - Code formatting
- `cargo clippy` - Linting
- CI checks - File size, test coverage

**Manual:**
- Code review - Architecture, design decisions
- Refactoring triggers - When files grow too large

## Evolution

These standards will evolve based on:
- Real pain points encountered
- Team feedback and discussion
- Rust ecosystem best practices
- Project-specific needs

**Last Updated:** 2025-10-13 (CDT)
**Last Updated By:** Provider refactoring retrospective
**Next Review:** When adding new major component

## Related Documents
- [testing-philosophy.md](./testing-philosophy.md) - Testing standards
- [CLAUDE.md](../../CLAUDE.md) - Project philosophy
- `.claude/commands/implement.md` - Implementation workflow

## Metadata
- **Status:** Active
- **Owner:** Team
- **Scope:** All Rust code in patinox
