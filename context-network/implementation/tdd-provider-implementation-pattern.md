# TDD Provider Implementation Pattern

## Classification
- **Domain**: Implementation Methodology
- **Stability**: Established (proven through Ollama implementation)  
- **Abstraction**: Implementation Guide
- **Confidence**: High

## Overview

Test-Driven Development approach for implementing new LLM providers in the Patinox framework. This pattern emerged from the successful Ollama provider implementation and establishes a proven methodology for future provider development.

## Core Methodology

### Phase 1: Test Structure Design (Before Any Implementation)

**Test Organization Pattern**:
```rust
mod provider_tests {
    // 1. Provider Creation Tests
    #[tokio::test]
    async fn test_provider_creation_with_default_endpoint() { }
    
    #[tokio::test]
    async fn test_provider_creation_with_custom_endpoint() { }
    
    // 2. Error Handling Tests (Write These First!)
    #[tokio::test]
    async fn test_service_unavailable_scenarios() { }
    
    #[tokio::test]
    async fn test_network_error_handling() { }
    
    // 3. Request Validation Tests
    #[tokio::test]
    async fn test_request_format_validation() { }
    
    // 4. Core Functionality Tests
    #[tokio::test] 
    async fn test_list_models_functionality() { }
    
    #[tokio::test]
    async fn test_completion_functionality() { }
    
    // 5. Integration Tests (Ignored by Default)
    #[tokio::test]
    #[ignore = "requires running [service] with [model]"]
    async fn test_integration_with_real_service() { }
}
```

### Phase 2: Error-First Implementation

**Priority Order**:
1. **Error Conditions**: Implement proper error mapping first
2. **Request Validation**: Handle malformed inputs appropriately  
3. **Happy Path**: Implement core functionality
4. **Edge Cases**: Handle boundary conditions

**Error Mapping Pattern**:
```rust
// Established pattern from Ollama implementation
.map_err(|e| match e {
    reqwest::Error if e.is_connect() => ProviderError::NetworkError(format!("Failed to connect: {}", e)),
    reqwest::Error if e.is_timeout() => ProviderError::NetworkError(format!("Request timeout: {}", e)), 
    _ => ProviderError::NetworkError(format!("Request failed: {}", e))
})
```

### Phase 3: API Integration Patterns

**HTTP Client Setup**:
```rust
// Standard pattern with appropriate timeout
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    .build()
    .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;
```

**Request/Response Transformation**:
- Create internal types for API requests/responses
- Transform between domain types and API types
- Handle missing/optional fields gracefully with `#[serde(default)]`

## Test Categories

### Unit Tests (No External Dependencies)
- Provider creation and configuration
- Request validation and error handling
- Service unavailable scenarios
- Concurrent request handling
- Request format validation

### Integration Tests (External Service Required)
- Real API interaction testing
- Model listing with actual service
- Completion requests with real models
- Service availability verification

**Key Pattern**: Use `#[ignore = "requires running [service]"]` with descriptive reasons

## Implementation Checklist

### Pre-Implementation
- [ ] Define test structure following established pattern
- [ ] Write failing tests for all error conditions
- [ ] Write failing tests for core functionality
- [ ] Verify test compilation (RED phase)

### Core Implementation  
- [ ] Implement error mapping and HTTP client setup
- [ ] Implement request validation
- [ ] Implement API integration with proper error handling
- [ ] Verify all unit tests pass (GREEN phase)

### Refinement
- [ ] Refactor for code quality (REFACTOR phase)
- [ ] Add comprehensive documentation
- [ ] Optimize performance if needed
- [ ] Add integration tests for real service validation

## Success Metrics

**Test Coverage Indicators**:
- All provider trait methods have corresponding tests
- Error conditions are comprehensively tested
- Both unit and integration tests are present
- All tests have descriptive, intention-revealing names

**Code Quality Indicators**:
- HTTP client patterns match established providers
- Error mapping is consistent with framework patterns
- Request/response types are well-structured
- Documentation includes usage examples

## Related Patterns
- **HTTP Error Mapping**: Standardized error transformation approach
- **Provider Abstraction Compliance**: Ensuring trait implementation correctness
- **Service Integration Testing**: Managing external service dependencies

## Discovered During
- Task: Ollama Provider Implementation (2025-08-23)
- Context: First comprehensive TDD implementation of provider
- Validation: 16 tests (11 unit, 5 integration), 100% pass rate, zero regressions

## Future Applications
- LMStudio provider implementation
- Any future local provider implementations  
- Remote provider implementations requiring similar patterns
- Provider refactoring initiatives

---

*This pattern emerged from the successful Ollama provider implementation and should be used as the standard approach for all future provider development.*