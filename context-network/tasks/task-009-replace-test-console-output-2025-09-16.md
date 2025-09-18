# Task 009: Replace Test Console Output with Proper Controls

## Overview
**Type**: Test Quality
**Priority**: Medium
**Effort**: Small
**Created**: 2025-09-16
**Source**: Test Review Recommendation

## Problem Statement
The streaming tests contain excessive console output (`println!` statements) that clutter test results and make CI logs noisy. This makes it difficult to spot real issues in test output and reduces the professionalism of the test suite.

**Examples**:
```rust
println!("Skipping Ollama test - provider creation failed");
println!("Expected network error - no Ollama server");
println!("Unexpected success - real Ollama instance detected");
```

**Issues**:
- Cluttered test output in CI/CD pipelines
- No standardized approach to conditional test execution
- Debugging information mixed with test results
- Inconsistent handling of optional external dependencies

## Acceptance Criteria

1. **Replace Console Output**:
   - Remove all `println!` statements from test files
   - Implement proper test skipping mechanisms
   - Maintain current test behavior (skip when dependencies unavailable)

2. **Implement Proper Test Controls**:
   - Use conditional compilation for integration tests
   - Add feature flags for external service tests
   - Use `#[ignore]` attribute where appropriate

3. **Standardize Test Categories**:
   - Unit tests: Always run, no external dependencies
   - Integration tests: Conditional on feature flags
   - External service tests: Conditional on environment

4. **Documentation**:
   - Document test categorization in README
   - Explain how to run different test suites
   - Update CI configuration if needed

## Implementation Plan

### Phase 1: Categorize Tests
```rust
// Unit tests (always run)
#[tokio::test]
async fn test_stream_completion_validates_empty_model() { ... }

// Integration tests (conditional)
#[tokio::test]
#[cfg(feature = "integration-tests")]
async fn test_stream_completion_with_real_server() { ... }

// External service tests (environment-based)
#[tokio::test]
#[cfg_attr(not(env = "CI"), ignore)]
async fn test_with_external_dependency() { ... }
```

### Phase 2: Feature Flags
Add to `Cargo.toml`:
```toml
[features]
default = []
integration-tests = []
external-services = ["integration-tests"]
```

### Phase 3: Environment Detection
```rust
fn should_skip_external_test() -> bool {
    std::env::var("SKIP_EXTERNAL_TESTS").is_ok() ||
    std::env::var("CI").is_ok()
}

#[tokio::test]
async fn test_with_external_service() {
    if should_skip_external_test() {
        return; // Silent skip
    }
    // Test implementation
}
```

## Files to Modify
- `tests/local_provider_streaming_test.rs` - Remove println! statements
- `Cargo.toml` - Add feature flags for test categories
- `README.md` - Document test execution patterns
- `.github/workflows/` or CI config - Update test commands

## Technical Considerations

### Feature Flag Strategy
- `integration-tests`: Tests that require mock servers
- `external-services`: Tests that need real external services
- Default: Only unit tests with full mocking

### CI/CD Integration
```bash
# Unit tests only (default)
cargo test

# With integration tests
cargo test --features integration-tests

# All tests including external services
cargo test --features external-services
```

### Backward Compatibility
- Existing test behavior preserved
- No breaking changes to test execution
- CI continues to work with default settings

## Success Metrics
- Zero `println!` statements in test output
- Clean test logs in CI/CD
- Proper categorization of all tests
- Documentation of test execution patterns
- Maintained test coverage and behavior

## Dependencies
- None (independent test improvement)

## Risk Assessment
- **Risk Level**: Low (test infrastructure improvement)
- **Breaking Changes**: None (behavior preserved)
- **Test Impact**: Improved (cleaner output, better organization)

## Alternative Approaches Considered

1. **Test-specific logging**: Keep println but use proper logging framework
   - Rejected: Still clutters output, doesn't solve root issue

2. **Environment variables only**: Skip based on env vars
   - Rejected: Less explicit than feature flags

3. **Separate test binaries**: Split tests into different executables
   - Rejected: Over-engineering for current needs