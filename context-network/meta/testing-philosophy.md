# Testing Philosophy - Patinox

## Purpose
Define our approach to testing that prioritizes testability, maintainability, and confidence. These principles emerged from real experience and the recognition that integration tests often mask design problems.

## Classification
- **Domain:** Meta/Process
- **Stability:** Stable (core philosophy)
- **Abstraction:** Principles
- **Confidence:** High (battle-tested)

## Core Philosophy

### Tests Validate OUR Code, Not Dependencies

**Principle:** Unit tests should test the code we write, not external libraries or APIs.

**What this means:**
- ✅ Test our logic, error handling, and business rules
- ✅ Test our abstractions and interfaces
- ✅ Test how we use dependencies
- ❌ Don't test if OpenAI's API works
- ❌ Don't test if async-openai crate works
- ❌ Don't test if HTTP works

**Example from provider refactoring:**

**Bad (Testing external API):**
```rust
#[tokio::test]
#[ignore] // Requires real API key
async fn test_openai_returns_response() {
    let provider = OpenAIProvider::new(config)?;
    let response = provider.complete(messages).await?;
    assert!(!response.is_empty()); // Testing OpenAI, not our code
}
```

**Good (Testing our code):**
```rust
#[tokio::test]
async fn test_provider_requires_api_key() {
    let config = ProviderConfig::new(Provider::OpenAI);
    config.api_key = None;

    let result = OpenAIProvider::new(config);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API key"));
}
```

## Integration Tests Are a Design Smell

**Principle:** If you need integration tests with real external services, your code isn't testable enough.

**Why integration tests are problematic:**
1. **Slow** - Network calls, real APIs, timeouts
2. **Flaky** - Network issues, rate limits, API changes
3. **Expensive** - API costs, CI time
4. **Brittle** - Break when external service changes
5. **Mask design issues** - Hard to test = poor design

**What to do instead:**

### Design for Testability

**Strategy 1: Dependency Injection**
```rust
// Bad: Hard-coded dependency
pub struct Agent {
    provider: OpenAIProvider, // Tightly coupled
}

// Good: Trait-based injection
pub struct Agent {
    provider: Box<dyn LLMProvider>, // Mockable
}
```

**Strategy 2: Thin Wrappers**
```rust
// Keep external dependencies thin and isolated
pub struct OpenAIProvider {
    client: async_openai::Client, // External dependency
    config: ProviderConfig,        // Our code
}

// Test our configuration logic, not the client
#[test]
fn test_config_validation() {
    // Test OUR validation logic
}
```

**Strategy 3: Seams for Testing**
```rust
// Provide mock implementations
pub struct MockProvider {
    response: String,
}

impl LLMProvider for MockProvider {
    async fn complete(&self, _: Vec<Message>) -> Result<String> {
        Ok(self.response.clone()) // Controlled, predictable
    }
}
```

## Test-Driven Development (TDD)

**Principle:** Write tests BEFORE implementation. This isn't optional.

**Why TDD:**
1. **Tests as specs** - Define what code should do
2. **Tests as docs** - Show how to use code
3. **Tests as safety net** - Catch regressions
4. **Tests drive design** - Hard to test = bad design

### The RED-GREEN-REFACTOR Cycle

**Phase 1: RED - Write Failing Test**
```rust
#[test]
fn test_provider_validates_empty_messages() {
    let provider = OpenAIProvider::new(valid_config())?;
    let result = provider.complete(vec![]).await;

    assert!(result.is_err()); // Currently fails - not implemented
}
```

**Phase 2: GREEN - Minimal Implementation**
```rust
pub async fn complete(&self, messages: Vec<Message>) -> Result<String> {
    if messages.is_empty() {
        return Err("Cannot complete with empty messages".into());
    }
    // ... rest of implementation
}
```

**Phase 3: REFACTOR - Improve Code**
```rust
// Refactor with confidence - tests protect you!
pub async fn complete(&self, messages: Vec<Message>) -> Result<String> {
    validate_messages(&messages)?; // Extracted, still tested
    // ... rest of implementation
}
```

**Discipline:**
- ✅ Write test first
- ✅ Watch it fail
- ✅ Write minimal code to pass
- ✅ Refactor only when green
- ❌ Don't skip the RED phase
- ❌ Don't write code before tests

## Test Quality Standards

### 1. Test Names Are Documentation

**Good test names:**
```rust
#[test]
fn test_provider_requires_api_key() { }

#[test]
fn test_empty_messages_return_error() { }

#[test]
fn test_unknown_role_returns_error() { }
```

**Bad test names:**
```rust
#[test]
fn test_1() { }

#[test]
fn test_provider() { }

#[test]
fn it_works() { }
```

### 2. Tests Are Focused

**One concept per test:**
```rust
// Good: Tests one thing
#[test]
fn test_config_builder_sets_model() {
    let config = ProviderConfig::new(Provider::OpenAI)
        .model("gpt-4o");

    assert_eq!(config.model, "gpt-4o");
}

// Bad: Tests multiple things
#[test]
fn test_config() {
    let config = ProviderConfig::new(Provider::OpenAI)
        .model("gpt-4o")
        .temperature(0.5);

    assert_eq!(config.model, "gpt-4o");
    assert_eq!(config.temperature, Some(0.5));
    assert!(config.api_key.is_some()); // Too many assertions
}
```

### 3. Tests Are Independent

**Each test stands alone:**
```rust
// Good: Self-contained
#[test]
fn test_validation() {
    let input = create_test_input(); // Fresh for each test
    assert!(validate(input).is_ok());
}

// Bad: Depends on other tests or state
static mut SHARED_STATE: i32 = 0;

#[test]
fn test_increments() {
    unsafe { SHARED_STATE += 1; } // Don't do this
}
```

### 4. Tests Follow AAA Pattern

**Arrange-Act-Assert:**
```rust
#[test]
fn test_message_creation() {
    // Arrange - Set up test data
    let content = "Hello, world!";

    // Act - Execute the code
    let msg = Message::user(content);

    // Assert - Verify results
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, content);
}
```

## What to Test

### ✅ DO Test

**1. Business Logic**
- Validation rules
- Calculations
- Data transformations
- State transitions

**2. Error Conditions**
- Invalid inputs
- Missing required data
- Edge cases
- Boundary conditions

**3. Public API**
- All public functions
- All trait implementations
- Configuration builders
- Return value contracts

**4. Our Error Handling**
- Error messages are clear
- Errors contain useful context
- Appropriate error types

**5. Our Integration Logic**
- How we call external APIs
- How we transform external data
- How we handle external errors

### ❌ DON'T Test

**1. External Dependencies**
- If OpenAI API works
- If HTTP client works
- If async runtime works

**2. Standard Library**
- If Vec works
- If Option works
- If Result works

**3. Framework Behavior**
- If tokio spawns tasks
- If serde serializes correctly
- If derive macros work

**4. Implementation Details**
- Private functions (test through public API)
- Internal state (test behavior, not implementation)

## Test Coverage Guidelines

**Target:** 80%+ coverage of our code

**Priority:**
1. **Critical paths** - Core functionality (100% coverage)
2. **Error handling** - All error branches (100% coverage)
3. **Edge cases** - Boundary conditions (90%+ coverage)
4. **Happy paths** - Normal operation (100% coverage)

**Not counted toward coverage:**
- External dependency calls (thin wrappers)
- Trivial getters/setters
- Generated code

## When Tests Fail

**Tests should fail for exactly one reason:**
```rust
// Good: Specific assertion
assert_eq!(result.model, "gpt-4o", "Model should be set to gpt-4o");

// Bad: Vague assertion
assert!(result.is_ok()); // Why did it fail?
```

**Test failures should be:**
- ✅ **Clear** - Error message explains what went wrong
- ✅ **Specific** - Points to exact problem
- ✅ **Actionable** - Developer knows what to fix

## Test Organization

### Small Projects (< 5 modules)
```
src/
  feature.rs
    #[cfg(test)]
    mod tests { }  // Inline tests
```

### Medium Projects (5-20 modules)
```
src/
  feature/
    mod.rs
    core.rs
    tests.rs     // Separate test module
```

### Large Projects (20+ modules)
```
src/
  feature/
    mod.rs
    core.rs

tests/           // Integration tests directory
  feature_tests.rs
```

**Note:** Even in large projects, minimize integration tests. Most tests should be unit tests in `src/`.

## Common Anti-Patterns

### 1. Testing Implementation, Not Behavior

```rust
// Bad: Tests internal state
#[test]
fn test_internal_counter() {
    let agent = Agent::new();
    assert_eq!(agent.internal_counter, 0); // Testing implementation
}

// Good: Tests behavior
#[test]
fn test_agent_starts_fresh() {
    let agent = Agent::new();
    assert_eq!(agent.call_count(), 0); // Testing public API
}
```

### 2. Mocking Everything

```rust
// Bad: Over-mocked
#[test]
fn test_with_mocks() {
    let mock_provider = MockProvider::new();
    let mock_config = MockConfig::new();
    let mock_validator = MockValidator::new();
    // If you need this many mocks, design is wrong
}

// Good: Real objects where possible
#[test]
fn test_with_real_objects() {
    let config = ProviderConfig::new(Provider::OpenAI); // Real
    let provider = MockProvider::new("response");       // Only mock external I/O
}
```

### 3. Testing External Services

```rust
// Bad: Requires real API
#[tokio::test]
#[ignore]
async fn test_openai_api() {
    let response = openai_client.complete(prompt).await?;
    assert!(!response.is_empty()); // Testing OpenAI, not us
}

// Good: Test our code
#[tokio::test]
async fn test_our_error_handling() {
    let invalid_config = ProviderConfig::new(Provider::OpenAI);
    invalid_config.api_key = None;

    let result = OpenAIProvider::new(invalid_config);
    assert!(result.is_err()); // Testing our validation
}
```

## Decision Framework

**When considering an integration test, ask:**

1. **What am I really testing?**
   - If it's external behavior → Don't test it
   - If it's our logic → Unit test it

2. **Can I mock the external dependency?**
   - Yes → Do that instead
   - No → Redesign for testability

3. **Am I testing because it's hard to unit test?**
   - Yes → That's a design smell, refactor
   - No → Good, proceed

4. **Will this test be fast, reliable, and free?**
   - No → It's not a good test
   - Yes → Might be okay (but question #1 first)

## Real Example: Provider Refactoring

**Before (7 integration tests):**
- Required real API keys
- Made real HTTP calls
- Tested if OpenAI works
- Slow, flaky, expensive

**After (4 unit tests):**
- No API keys needed
- No network calls
- Test our validation logic
- Fast, reliable, free

**What we learned:**
- Integration tests were testing async-openai, not our code
- Our code was actually very simple and fully testable via unit tests
- Removing integration tests improved test suite quality

## Summary: Testing Principles

1. **Test OUR code, not dependencies**
2. **Integration tests = design smell**
3. **TDD is mandatory, not optional**
4. **RED-GREEN-REFACTOR discipline**
5. **Tests should be fast, focused, and independent**
6. **80%+ coverage of critical paths**
7. **Design for testability from the start**

## Related Documents
- [coding-standards.md](./coding-standards.md) - Code quality standards
- [CLAUDE.md](../../CLAUDE.md) - Project philosophy
- `.claude/commands/implement.md` - TDD implementation workflow

## Metadata
- **Status:** Active
- **Created:** 2025-10-13 (CDT)
- **Last Updated:** 2025-10-13 (CDT)
- **Last Updated By:** Provider refactoring lessons
- **Next Review:** After next major component implementation
