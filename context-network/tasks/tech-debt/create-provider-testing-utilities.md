# Create Provider Testing Utilities

## Task Overview
**Priority**: High  
**Effort**: Medium (2-3 hours)  
**Risk**: Low  
**Source**: Retrospective recommendation from provider implementation experience

## Background
Multiple provider implementations (OpenAI, Anthropic, OpenRouter, Ollama) have revealed common testing patterns and boilerplate code that could be abstracted into shared utilities. This would accelerate future provider development and ensure consistent testing approaches.

## Current State
**Testing Boilerplate Identified**:
- Provider creation and configuration testing patterns
- Service unavailable error handling test patterns
- Request validation test patterns
- Mock HTTP client setup for unit tests
- Integration test organization and annotation patterns

**Code Duplication Examples**:
```rust
// Repeated across all provider tests
let provider = Provider::with_endpoint("http://localhost:99999".to_string())
    .expect("Should create provider");
let result = provider.some_method().await;
assert!(result.is_err());
match result.unwrap_err() {
    ProviderError::NetworkError(_) => { /* Expected */ }
    other => panic!("Expected NetworkError, got {:?}", other),
}
```

## Acceptance Criteria

### Core Test Utilities
- [ ] Create `ProviderTestUtils` with common test patterns
- [ ] Abstract service-unavailable error testing pattern
- [ ] Create mock HTTP client builders for unit testing
- [ ] Standardize integration test annotation patterns
- [ ] Provide request validation test helpers

### Test Organization Utilities
- [ ] Create macros or helpers for provider trait implementation testing
- [ ] Standardize concurrent request testing patterns
- [ ] Abstract timeout and error condition testing
- [ ] Provide test data builders for common request types

### Integration Test Management
- [ ] Create utilities for managing external service dependencies
- [ ] Standardize service availability checking
- [ ] Provide clear patterns for ignoring integration tests in CI

## Implementation Approach

### Phase 1: Common Pattern Analysis
1. **Review Existing Tests**: Analyze all provider test suites for common patterns
2. **Extract Abstractions**: Identify reusable components and patterns
3. **Design API**: Create clean, ergonomic testing utility API

### Phase 2: Utility Implementation
```rust
// Example utility structure
pub struct ProviderTestUtils;

impl ProviderTestUtils {
    /// Test that provider handles service unavailable gracefully
    pub async fn test_service_unavailable<P, F>(
        provider_factory: F
    ) -> Result<(), Box<dyn std::error::Error>>
    where 
        P: ModelProvider,
        F: Fn(String) -> Result<P, ProviderError>,
    {
        let provider = provider_factory("http://localhost:99999".to_string())?;
        let result = provider.list_models().await;
        assert!(matches!(result, Err(ProviderError::NetworkError(_))));
        Ok(())
    }
    
    /// Create mock HTTP client for unit testing
    pub fn mock_http_client() -> reqwest::Client { /* ... */ }
    
    /// Standard integration test annotation
    pub fn integration_test_attrs(service: &str) -> String {
        format!("#[ignore = \"requires running {} service\"]", service)
    }
}
```

### Phase 3: Test Macro Creation
```rust
// Example test generation macro
macro_rules! provider_standard_tests {
    ($provider_type:ty, $service_name:expr) => {
        #[tokio::test]
        async fn test_provider_creation() {
            // Standard provider creation test
        }
        
        #[tokio::test] 
        async fn test_service_unavailable() {
            ProviderTestUtils::test_service_unavailable(
                |endpoint| <$provider_type>::with_endpoint(endpoint)
            ).await.unwrap();
        }
        
        // ... other standard tests
    };
}
```

### Phase 4: Integration and Validation
1. **Apply to Existing Providers**: Refactor existing tests to use utilities
2. **Validate Approach**: Ensure utilities actually reduce boilerplate
3. **Document Usage**: Create clear guide for using test utilities

## Files to Create
- `tests/utils/mod.rs` - Main test utility module
- `tests/utils/provider_test_utils.rs` - Provider-specific test utilities  
- `tests/utils/mock_builders.rs` - Mock HTTP client and response builders
- `tests/utils/macros.rs` - Test generation macros

## Expected Benefits

### Immediate Benefits
- **Reduced Boilerplate**: 30-50% reduction in test code duplication
- **Consistent Patterns**: All providers use same testing approaches
- **Faster Development**: New providers can leverage established test patterns

### Long-term Benefits  
- **Test Quality**: Comprehensive test patterns ensure better coverage
- **Maintainability**: Changes to test patterns apply across all providers
- **Developer Experience**: Clear, reusable testing patterns reduce cognitive load

## Success Metrics
- **Boilerplate Reduction**: Measurable decrease in duplicated test code
- **Pattern Adoption**: New providers use utilities by default
- **Test Coverage**: Consistent test coverage across all providers
- **Development Speed**: Faster provider test implementation

## Design Considerations

### API Design
- **Ergonomic**: Easy to use and understand
- **Flexible**: Works with different provider types and configurations
- **Type-safe**: Leverages Rust's type system for correctness

### Backwards Compatibility
- **Non-breaking**: Existing tests continue to work
- **Incremental**: Can be adopted gradually across providers
- **Optional**: Utilities enhance but don't replace manual testing

### Test Organization
- **Clear Structure**: Utilities have obvious purpose and usage
- **Documentation**: Comprehensive examples and usage guides
- **Integration**: Works with existing test infrastructure

## Related Patterns
- **TDD Provider Implementation**: Utilities support TDD methodology
- **Provider HTTP Error Mapping**: Utilities test error mapping consistency
- **Local Provider Integration**: Utilities handle local service testing patterns

## Dependencies
- **Existing Providers**: Need current test patterns for abstraction
- **Test Infrastructure**: Build on existing test framework
- **Provider Traits**: Utilities work with ModelProvider abstraction

## Metadata
- **Created**: 2025-08-23 19:30 CDT
- **Source**: Retrospective recommendation from testing pattern analysis
- **Category**: Technical Debt/Developer Experience
- **Estimated Duration**: 2-3 hours including design, implementation, and documentation