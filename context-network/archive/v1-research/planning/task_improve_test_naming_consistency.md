# Task: Improve Test Naming Consistency

## Status
- **Priority**: Low
- **Complexity**: Small
- **Effort**: Trivial-Small
- **Dependencies**: None

## Context
During test quality review, inconsistent test naming conventions were identified across the codebase. Some tests use `test_` prefix while others don't, creating inconsistency.

## Problem Statement
Test naming is inconsistent across the codebase:
- Some unit tests use `test_` prefix
- Some integration tests omit prefixes
- Function names don't always clearly describe the scenario being tested

## Requirements
1. **Consistent Naming Convention**: Adopt uniform naming across all test files
2. **Descriptive Names**: Test names should clearly describe the scenario
3. **Hierarchical Structure**: Use consistent patterns for grouping related tests
4. **Documentation**: Update testing guidelines with naming conventions

## Proposed Convention
```rust
// Unit tests: test_[component]_[scenario]
#[test]
fn test_model_id_with_provider_hint() { ... }

// Integration tests: test_[workflow]_[scenario] 
#[tokio::test]
async fn test_provider_error_recovery_flow() { ... }

// Edge cases: test_[component]_edge_case_[scenario]
#[test] 
fn test_validation_edge_case_empty_input() { ... }
```

## Implementation Steps
1. Audit all test files for naming patterns
2. Create naming convention documentation
3. Rename tests to follow consistent pattern
4. Update test organization and grouping
5. Add naming checks to CI/linting if desired

## Acceptance Criteria
- [ ] All test names follow consistent convention
- [ ] Test names clearly describe scenarios being tested
- [ ] Related tests are logically grouped
- [ ] Naming convention is documented
- [ ] All tests continue to pass after renaming

## Files to Review
- All `#[test]` and `#[tokio::test]` functions
- Integration test files in `/tests/`
- Embedded unit tests in `/src/` modules

## Benefits
- Improved test discoverability
- Clearer test failure messages
- Better test organization
- Easier maintenance and debugging

Created: 2025-01-20 (deferred from test quality review)