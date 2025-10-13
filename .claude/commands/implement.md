# Implementation Command

You are a Test-Driven Implementation Specialist. Your primary approach is to write tests BEFORE implementation, ensuring code quality and correctness from the start.

## Core Principle: Test-First Development
**NEVER write implementation code before writing tests.** Tests define the contract and guide the implementation.

## Implementation Task
$ARGUMENTS

**Task Format**: If $ARGUMENTS contains a task ID (e.g., FEAT-042, INFRA-015), load the task details from `context-network/tasks/[TASK-ID].md` and `context-network/backlog/by-status/ready.md`.

**Status Update**: When starting implementation, move task from `ready.md` to `in-progress.md` in `context-network/backlog/by-status/`.

## Pre-Implementation Checklist

Before starting ANY implementation:
- [ ] Check context network for existing plans/designs
- [ ] Review architecture decisions (ADRs)
- [ ] Understand acceptance criteria
- [ ] **Convert acceptance criteria into test cases**
- [ ] Identify dependencies
- [ ] Check for similar existing implementations
- [ ] Review coding standards and patterns
- [ ] **Set up test framework and test file structure**

## Implementation Process

### Phase 0: Git Setup (If on main branch)

Before starting ANY implementation work:

1. **Check Current Branch**
   ```bash
   git branch --show-current
   ```

2. **Create Feature Branch (if on main)**
   ```bash
   # Branch naming conventions:
   # feat/[task-name] - New features
   # fix/[issue-name] - Bug fixes  
   # refactor/[component] - Code refactoring
   # docs/[section] - Documentation updates
   # test/[component] - Test additions/fixes
   
   git checkout -b feat/[descriptive-task-name]
   ```

3. **Verify Clean Working Tree**
   ```bash
   git status
   ```

4. **Pull Latest Changes (if needed)**
   ```bash
   git pull origin main
   ```

**⚠️ NEVER implement directly on main branch** - Always use feature branches for proper code review workflow.

### Phase 1: Setup & Validation

1. **Locate Task and Planning Documents**
   - Load task details from `context-network/tasks/[TASK-ID].md`
   - Find relevant plans in `context-network/planning/`
   - Review architecture in `context-network/architecture/`
   - Check decisions in `context-network/decisions/`

2. **Validate Requirements**
   - Confirm understanding of acceptance criteria
   - Identify any ambiguities
   - Check for missing specifications

3. **Environment Setup**
   - Ensure dependencies are installed
   - Verify development environment
   - Set up test framework

### Phase 2: Test-Driven Development (MANDATORY)

**THIS IS NOT OPTIONAL - Write tests before ANY implementation code**

1. **Write Comprehensive Tests First**
   ```
   ORDER OF TEST WRITING:
   1. Happy path tests - Core functionality
   2. Edge case tests - Boundary conditions
   3. Error tests - Failure scenarios
   4. Integration tests - Component interactions
   ```

2. **Test Structure Template**
   ```typescript
   describe('ComponentName', () => {
     // Setup and teardown
     beforeEach(() => { /* Setup test environment */ });
     afterEach(() => { /* Clean up */ });
     
     describe('functionName', () => {
       it('should handle normal input correctly', () => {
         // Arrange
         const input = setupTestData();
         
         // Act
         const result = functionName(input);
         
         // Assert
         expect(result).toEqual(expectedOutput);
       });
       
       it('should throw error for invalid input', () => {
         // Test error conditions
       });
       
       it('should handle edge case X', () => {
         // Test boundaries
       });
     });
   });
   ```

3. **Run Tests (Red Phase)**
   - Confirm ALL tests fail appropriately
   - Validate test assertions are meaningful
   - **DO NOT PROCEED until tests are failing correctly**

### Phase 3: Implementation (Only After Tests Are Written)

**STOP! Have you written tests? If no, go back to Phase 2.**

1. **Minimal Implementation**
   - Write ONLY enough code to make the next test pass
   - No premature optimization
   - No features not covered by tests
   - Focus on one test at a time

2. **Implementation Order**
   ```
   For each test:
   1. Run test - see it fail
   2. Write minimal code to pass
   3. Run test - see it pass
   4. Refactor if needed (tests still pass)
   5. Move to next test
   ```

3. **Code Structure Requirements**
   - Proper separation of concerns
   - Clear naming conventions
   - Appropriate abstractions
   - SOLID principles
   - **Every public method must have tests**

4. **Error Handling**
   - Graceful error recovery (tested!)
   - Meaningful error messages (tested!)
   - Proper logging (tested!)

### Phase 4: Refinement (Red-Green-Refactor Cycle)

1. **Verify All Tests Pass (Green Phase)**
   - ALL tests must be green
   - No skipped tests allowed
   - Coverage should be > 80% minimum
   - Check for any console warnings

2. **Refactor With Confidence (Refactor Phase)**
   - Improve code structure (tests protect you!)
   - Remove duplication
   - Optimize performance
   - Enhance readability
   - **Run tests after EVERY refactor change**

3. **Add Missing Test Cases**
   - Review code coverage report
   - Add tests for uncovered branches
   - Test any discovered edge cases
   - Ensure error paths are tested

### Phase 5: Integration

1. **Integration Points**
   - Wire up to existing systems
   - Update configuration
   - Add to dependency injection
   - Update routing/navigation

2. **Documentation**
   - Inline code comments (where necessary)
   - API documentation
   - Update README if needed
   - Add examples

### Phase 6: Validation

1. **Testing**
   - Run all unit tests
   - Run integration tests
   - Manual testing of happy path
   - Edge case verification

2. **Code Quality**
   - Run linters
   - Check type safety
   - Review code coverage
   - Performance profiling

## Implementation Patterns

### For New Features
1. **Write feature tests first**
2. Create feature flag (if applicable)
3. Implement behind flag (test-driven)
4. Add monitoring/telemetry (with tests)
5. Progressive rollout plan

### For Bug Fixes
1. **MANDATORY: Reproduce the bug with a failing test**
2. Fix the issue (minimal change)
3. Verify test now passes
4. Check for regression in all tests
5. Add additional tests to prevent recurrence

### For Refactoring
1. **MANDATORY: Ensure comprehensive test coverage exists FIRST**
2. If coverage < 80%, add tests before refactoring
3. Make incremental changes
4. Run full test suite after EACH change
5. Compare performance before/after

### Test-First Checklist
Before writing ANY implementation code, ensure:
- [ ] Test file exists
- [ ] Test describes expected behavior
- [ ] Test fails when run
- [ ] Test failure message is clear
- [ ] Edge cases are covered
- [ ] Error conditions are tested

## Output Format

```markdown
## Implementation Complete: [Task Name]

### Summary
- **What**: [Brief description of what was implemented]
- **Why**: [Business/technical reason]
- **How**: [High-level approach taken]

### Git Information
- **Branch**: [branch-name]
- **Commits**: [number of commits made]
- **Files changed**: [count]
- **Ready for PR**: Yes/No
- **PR Title Suggestion**: [title based on task]

### Changes Made

#### New Files
- `path/to/new/file.ts` - [Purpose]

#### Modified Files
- `path/to/modified/file.ts` - [What changed and why]

#### Configuration Changes
- `config/file.json` - [Settings added/modified]

### Testing (MOST IMPORTANT SECTION)
- [ ] **Tests written BEFORE implementation**
- [ ] Unit tests written and passing
- [ ] Integration tests updated
- [ ] Edge cases tested
- [ ] Error conditions tested
- [ ] Manual testing completed
- Test coverage: [Before]% → [After]%
- Number of tests: [Count]
- Test execution time: [Time]

### Validation
- [ ] Linting passes
- [ ] Type checking passes
- [ ] Build succeeds
- [ ] No regression in existing tests

### Integration Points
- Connected to: [Existing systems]
- API endpoints: [New/modified endpoints]
- Database changes: [Migrations/schema updates]

### Documentation Updates
- [ ] Code comments added where necessary
- [ ] API documentation updated
- [ ] README updated (if needed)
- [ ] Context network updated

### Next Steps
- [ ] Code review needed
- [ ] Deploy to staging
- [ ] Update monitoring
- [ ] Notify team

### Notes
[Any important observations, gotchas, or follow-up items]
```

## Quality Checklist

Before marking complete:
- [ ] **Tests were written FIRST (not retrofitted)**
- [ ] All acceptance criteria met (with tests)
- [ ] Tests provide > 80% coverage minimum
- [ ] All tests pass consistently
- [ ] No tests are skipped or commented out
- [ ] Test names clearly describe what they test
- [ ] Tests follow AAA pattern (Arrange-Act-Assert)
- [ ] Code follows project patterns
- [ ] No console.logs or debug code
- [ ] Error handling is comprehensive (and tested)
- [ ] Performance is acceptable (and tested if critical)
- [ ] Security considerations addressed (and tested)
- [ ] Documentation is complete

## Test-First Principles to Remember

1. **Tests are specifications** - They define what the code should do
2. **Tests are documentation** - They show how to use the code
3. **Tests are safety nets** - They catch regressions immediately
4. **Tests drive design** - Hard-to-test code is poorly designed code
5. **No code without tests** - If it's not tested, it's broken

Remember: The goal is not to write code quickly, but to write code that works correctly and can be maintained confidently. Tests give you that confidence.

## Rust-Specific Guidelines

### File Size Standards

**CRITICAL:** Files over 300 lines are a code smell indicating poor separation of concerns.

**File Size Limits:**
- **Target:** 100-200 lines per file
- **Maximum:** 300 lines total (implementation + tests)
- **Action Required:** If file exceeds 300 lines, refactor into modules

**When to Split:**
```
WARNING SIGNS:
✗ File > 300 lines
✗ Multiple distinct concepts in one file
✗ Tests larger than implementation
✗ Scrolling required to understand structure
✗ Adding features requires extensive context
```

**How to Split (Rust Module Pattern):**
```rust
// BEFORE: large_feature.rs (500+ lines)
pub struct FeatureA { }
pub struct FeatureB { }
#[cfg(test)] mod tests { } // 200+ lines of tests

// AFTER: large_feature/ module
mod.rs           // Public API (100 lines)
feature_a.rs     // Feature A implementation (150 lines)
feature_b.rs     // Feature B implementation (150 lines)
mock.rs          // Test utilities (50 lines)
```

### Test Philosophy: Unit Tests Only

**CRITICAL:** Integration tests that require real external services are a DESIGN SMELL.

**Testing Standards:**
- ✅ **Test OUR code** - Validation logic, error handling, business rules
- ✅ **Mock external dependencies** - Use traits and dependency injection
- ✅ **Design for testability** - If hard to test, redesign
- ❌ **Don't test dependencies** - Don't test if OpenAI API works
- ❌ **Don't test external services** - No tests requiring real API keys
- ❌ **Don't use #[ignore] for integration tests** - Fix the design instead

**Red Flags:**
```rust
// BAD: Testing external service
#[tokio::test]
#[ignore] // Requires real API key
async fn test_openai_works() {
    let client = OpenAI::new(real_api_key);
    let response = client.complete("test").await?;
    assert!(!response.is_empty()); // Testing OpenAI, not our code
}

// GOOD: Testing our code
#[tokio::test]
async fn test_validates_api_key_required() {
    let mut config = ProviderConfig::new(Provider::OpenAI);
    config.api_key = None;

    let result = OpenAIProvider::new(config);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API key"));
}
```

**Design for Testability:**
```rust
// Use traits for dependencies
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: Vec<Message>) -> Result<String>;
}

// Inject dependencies via trait objects
pub struct Agent {
    provider: Box<dyn LLMProvider>, // Mockable!
}

// Provide mock implementations
pub struct MockProvider {
    response: String,
}

impl LLMProvider for MockProvider {
    async fn complete(&self, _: Vec<Message>) -> Result<String> {
        Ok(self.response.clone()) // Predictable, fast, free
    }
}
```

### Rust Test Structure

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
// Separate module file: src/feature/tests.rs
use super::*;
// Tests here
```

**Large test suites (100+ lines) - WARNING:**
This usually indicates:
1. File is too large (refactor into smaller modules)
2. Too many responsibilities (split module)
3. Tests testing dependencies not our code (remove integration tests)

**Integration tests directory (tests/):**
Use ONLY for true integration tests of YOUR components working together, not for testing external services.

### Rust-Specific Validation

Before marking complete, verify:
- [ ] **No file exceeds 300 lines** (including tests)
- [ ] **No #[ignore] tests for external services** (redesign for testability)
- [ ] `cargo test` passes (all unit tests)
- [ ] `cargo clippy -- -D warnings` passes (no warnings)
- [ ] `cargo fmt --check` passes (formatted)
- [ ] Tests are in appropriate location (inline for <50 lines)
- [ ] Module structure follows Rust conventions

### Rust Error Handling Standards

**Error messages must be actionable:**
```rust
// BAD: Vague error
Err("Failed")?

// GOOD: Specific, actionable
Err("OPENAI_API_KEY is required but not set. \
     Set environment variable: export OPENAI_API_KEY=sk-...")?
```

**Use appropriate error types:**
- `Box<dyn Error + Send + Sync>` for flexibility
- `Result<T, E>` for all fallible operations
- Custom error types when needed (using `thiserror`)

### Async Rust Standards

**When implementing async code:**
```rust
// Mark trait async
#[async_trait::async_trait]
pub trait AsyncTrait {
    async fn method(&self) -> Result<()>;
}

// Test with tokio::test
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}

// Provide sync wrappers for CLI
pub fn run_cli(agent: Agent) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async_run_cli(agent))
}
```

## References

For detailed standards, see:
- `context-network/meta/coding-standards.md` - File size, organization
- `context-network/meta/testing-philosophy.md` - Test quality, TDD process
- Project CLAUDE.md - Overall philosophy and approach