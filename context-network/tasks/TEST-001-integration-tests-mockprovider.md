# TEST-001: Add Integration Tests Using MockProvider

## Status
- **Created**: 2025-10-13
- **Priority**: Medium
- **Effort**: Medium (30-60 minutes)
- **Type**: Testing / Quality Improvement

## Context

**Source**: PR #21 Code Review
**Current State**: Some tests skip when OPENAI_API_KEY is not set
**Suggestion**: Use MockProvider for deterministic integration testing

## Problem Statement

Current test approach has limitations:

```rust
#[tokio::test]
async fn test_openai_empty_messages() {
    let provider = OpenAIProvider::new(config);
    if provider.is_err() {
        return; // Skips test without API key
    }
    // ... test logic
}
```

**Issues**:
- Tests are skipped in CI/environments without API keys
- Non-deterministic (depends on external API)
- Slower (network calls)
- Can't test edge cases (API-specific error responses)

## Acceptance Criteria

- [ ] Add integration tests for both agents using MockProvider
- [ ] Tests run deterministically without API keys
- [ ] Cover full agent workflows (tool calls, LLM responses, error handling)
- [ ] Tests verify agent behavior, not LLM correctness
- [ ] All tests pass in CI without environment variables
- [ ] Maintain existing API integration tests (for manual verification)
- [ ] Document testing patterns for future agents

## Proposed Test Structure

### Test File Organization

```
tests/
├── agent_integration_test.rs (new)
├── file_processor_test.rs (new)
└── doc_generator_test.rs (new)
```

### Example Integration Test

```rust
#[tokio::test]
async fn test_file_processor_with_mock() {
    // Create agent with mock provider
    let mock_response = "File analysis: ...";
    let mock = MockProvider::new(mock_response);

    let agent = create_agent("test")
        .tool_fn("read_file", "Read file", |path| {
            Ok(format!("Contents of {}", path))
        })
        .with_provider(Box::new(mock));

    // Run agent
    let result = agent.run("analyze test.txt").await;

    // Verify behavior
    assert!(result.is_ok());
    assert!(result.unwrap().contains("File analysis"));
}

#[tokio::test]
async fn test_file_processor_tool_calls() {
    let mock = MockProvider::new("Used tools successfully");

    let agent = create_file_processor_agent()
        .with_provider(Box::new(mock));

    // Test that tools are registered correctly
    assert!(agent.tools.contains_key("read_file"));
    assert!(agent.tools.contains_key("count_lines"));
    // ...
}

#[tokio::test]
async fn test_agent_error_handling() {
    let mock = MockProvider::with_error("API error");

    let agent = create_agent("test")
        .with_provider(Box::new(mock));

    let result = agent.run("test input").await;

    // Verify error propagation
    assert!(result.is_err());
}
```

## Test Scenarios to Cover

### For File Processor Agent
1. **Happy path**: Read file → analyze → success
2. **Tool execution**: Verify all tools callable
3. **Error handling**: File not found, permission denied
4. **Edge cases**: Empty file, large file (mocked)

### For Documentation Generator
1. **Happy path**: Read source → generate docs → success
2. **Tool execution**: Verify code analysis tools work
3. **Error handling**: Invalid Rust file, parse errors
4. **Output**: Verify markdown generation (with mock)

### General Agent Behavior
1. **Provider integration**: Mock responses work correctly
2. **Tool chaining**: Multiple tool calls in sequence
3. **Error propagation**: Errors bubble up correctly
4. **Configuration**: Builder pattern works as expected

## Files to Create/Modify

**New Test Files:**
- `tests/file_processor_integration_test.rs` (~100 lines)
- `tests/doc_generator_integration_test.rs` (~100 lines)

**Existing Files to Review:**
- Keep `tests/local_provider_test.rs` (API integration tests)
- Keep `src/provider/mock.rs` (may need enhancements)

## MockProvider Enhancements Needed?

Current MockProvider might need:
- [ ] Response sequences (multiple calls return different responses)
- [ ] Error injection (return errors on demand)
- [ ] Call tracking (verify what the LLM was called with)
- [ ] Conditional responses (based on input)

**Example Enhancement:**
```rust
pub struct MockProvider {
    responses: VecDeque<String>,
    should_error: bool,
    call_count: Arc<Mutex<usize>>,
}

impl MockProvider {
    pub fn with_responses(responses: Vec<&str>) -> Self { ... }
    pub fn with_error(message: &str) -> Self { ... }
    pub fn call_count(&self) -> usize { ... }
}
```

## Testing Plan

1. **Write tests for file processor agent**
2. **Write tests for doc generator agent**
3. **Enhance MockProvider if needed**
4. **Run full test suite**: `cargo test`
5. **Verify tests pass without API key**
6. **Check test coverage**: Should improve

## Why Deferred (Not Applied Immediately)

1. **Medium Effort**: 30-60 minutes to write comprehensive tests
2. **Not Blocking**: Current testing is adequate for V2 phase
3. **Additive**: Doesn't fix bugs, adds quality
4. **Timing**: Better after Week 3 plugins stabilize
5. **Strategy**: Should be part of broader test strategy discussion

## Benefits of Implementation

- ✅ Deterministic tests (no flaky network calls)
- ✅ Faster CI (no API calls)
- ✅ Test edge cases (mock various responses)
- ✅ No API key requirement
- ✅ Better test coverage
- ✅ Clear patterns for future agents

## Dependencies

- MockProvider exists and works (✅ confirmed in PR)
- May need MockProvider enhancements (minor)

## Related Work

- Consider: Property-based testing with proptest?
- Review: Testing patterns in V1 archive
- Document: Testing philosophy for V2 agents
- Future: Test utilities for common agent patterns?

## Success Metrics

- Test coverage increases by ~10-15%
- All integration tests pass without API keys
- CI runs faster (no network calls in tests)
- Tests are deterministic and reliable
- Clear testing patterns established for future agents

## Notes

**Good Practice**: This is a solid improvement that aligns with TDD principles and V2's testing philosophy.

**Priority Rationale**: Medium priority because:
- Current test coverage is adequate for V2 minimal phase
- Not blocking any features
- Improves quality but doesn't fix bugs
- Better done after plugin architecture is stable

**Recommendation**: Implement during Week 3 or Week 4 as part of test strategy refinement.

## Testing Philosophy Alignment

From `context-network/meta/testing-philosophy.md`:
- ✅ Test our code, not external APIs
- ✅ Use mocks for external dependencies
- ✅ Integration tests should be deterministic

This task aligns perfectly with documented testing philosophy.
