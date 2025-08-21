# Task: Organize Large Test Suites into Focused Submodules

## Overview
**Priority**: Low-Medium
**Effort**: Small to Medium (15-45 minutes)
**Risk**: Low
**Created**: 2025-08-20
**Source**: Code review recommendations

## Problem Statement

Some test files have grown large and could benefit from better organization:
- `tests/openrouter_provider_test.rs`: 581 lines with 20+ test cases
- Future test files may grow similarly as more features are added

Better test organization improves maintainability, makes it easier to run focused test suites, and helps with test discoverability.

## Acceptance Criteria

### OpenRouter Test Organization
- [ ] Split `tests/openrouter_provider_test.rs` into logical submodules:
  - [ ] `creation_tests` - Provider creation and configuration
  - [ ] `request_conversion_tests` - Format conversion logic
  - [ ] `error_handling_tests` - Error scenarios and edge cases
  - [ ] `integration_tests` - Full request/response cycles
  - [ ] `model_routing_tests` - Provider routing and preferences
- [ ] All 20+ tests continue passing
- [ ] Tests remain discoverable with `cargo test openrouter`
- [ ] Test names remain clear and descriptive

### Establish Test Organization Pattern
- [ ] Create consistent pattern for future test file organization
- [ ] Document test organization guidelines in context network
- [ ] Consider applying similar organization to other large test files

## Technical Approach

### Test Module Structure
```rust
// tests/openrouter_provider_test.rs
mod creation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_openrouter_provider_creation_with_valid_api_key() {
        // Test implementation
    }
    
    #[tokio::test]
    async fn test_openrouter_provider_creation_with_empty_api_key() {
        // Test implementation
    }
    // ... other creation tests
}

mod request_conversion_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_request_serialization() {
        // Test implementation
    }
    // ... other conversion tests
}

// etc.
```

### Test Categories for OpenRouter
1. **Creation Tests** (5 tests)
   - Provider creation with valid/invalid keys
   - Custom configuration (base URL, headers)
   - Builder pattern functionality

2. **Request Conversion Tests** (4 tests)
   - Message format conversion
   - Provider preference handling
   - Request serialization

3. **Error Handling Tests** (4 tests)
   - Rate limiting scenarios
   - Authentication failures
   - Network timeouts
   - Invalid model handling

4. **Integration Tests** (4 tests)
   - Complete request/response cycles
   - Model capabilities
   - Embedding functionality

5. **Model Routing Tests** (3 tests)
   - Provider routing strategies
   - Cost optimization
   - Model support checking

## Success Metrics
- Tests organized into clear functional groups
- Zero test failures after reorganization
- Improved test discoverability and navigation
- Consistent pattern for future test organization
- Maintained test execution speed

## Implementation Guidelines

### Module Organization Principles
- Group tests by functionality, not by test type
- Each module should test a cohesive set of behaviors
- Avoid circular dependencies between test modules
- Use descriptive module names that indicate what's being tested

### Test Naming Conventions
- Keep existing descriptive test names
- Use module name to provide context (avoid redundant prefixes)
- Ensure tests can still be run individually or by pattern

### Shared Setup
- Extract common test setup to shared functions
- Use module-level setup where appropriate
- Maintain test isolation (no shared mutable state)

## Optional Extensions

### If Time Permits
- [ ] Apply similar organization to `tests/provider_integration_test.rs` (470 lines)
- [ ] Create test organization documentation in context network
- [ ] Consider test utility functions for common patterns

### Future Considerations
- Organize new test files from the start
- Consider property-based test organization
- Think about performance test separation

## Dependencies
- Should coordinate with any ongoing test development
- Consider impact on CI test categorization
- May want to standardize across all provider tests

## Risks and Mitigations
- **Risk**: Breaking test discovery or CI pipelines
- **Mitigation**: Test with `cargo test` patterns, maintain same test names

- **Risk**: Reduced test isolation
- **Mitigation**: Ensure each module remains self-contained

## Validation Checklist
- [ ] All 20+ OpenRouter tests pass
- [ ] Tests discoverable with `cargo test openrouter`
- [ ] Individual test execution works: `cargo test test_specific_test_name`
- [ ] Test module pattern matching works
- [ ] No increase in test execution time
- [ ] Test coverage unchanged