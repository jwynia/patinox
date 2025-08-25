# Discovery Record: Local Provider Integration Patterns

**Record ID**: 2025-08-23-001  
**Date**: August 23, 2025  
**Discovered During**: Ollama Provider Implementation  
**Discoverer**: Implementation Team  

## What We Were Looking For
Patterns for integrating local LLM providers (services running on localhost) with the Patinox provider abstraction framework.

## What We Found

### Key Differences from Cloud Providers

**1. Service Availability Assumptions**
- **Cloud providers**: Assume service is available unless explicitly failing
- **Local providers**: Must handle service not running scenarios gracefully
- **Pattern**: Use service-unavailable endpoints (e.g., localhost:99999) in tests

**2. Error Handling Characteristics**
- **Connection errors are common**: Local services may not be running
- **No authentication typically**: Skip auth error handling patterns
- **No rate limiting**: Remove rate limit error mapping
- **Model loading delays**: Handle model loading/unloading scenarios

**3. Performance Characteristics**
- **No API rate limits**: Can make requests more aggressively
- **Low latency**: Network calls are essentially local function calls
- **Resource constraints**: Local hardware limitations vs cloud scaling

### Implementation Patterns Discovered

**1. Endpoint Configuration Pattern**
```rust
// Default to standard service ports
const DEFAULT_ENDPOINT: &str = "http://localhost:11434"; // Ollama
const DEFAULT_ENDPOINT: &str = "http://localhost:1234";  // LMStudio

// Allow easy endpoint customization for different setups
pub fn with_endpoint(endpoint: String) -> ProviderResult<Self>
```

**2. Service Discovery Integration**
```rust
// Local providers integrate with service discovery foundation
// Foundation handles port scanning and service detection
// Provider focuses on API integration once service is found
```

**3. Error Mapping for Local Services**
```rust
// Emphasize connection failures over auth/quota errors
match error {
    reqwest::Error if error.is_connect() => {
        ProviderError::NetworkError(format!(
            "{} service not available at {}. Ensure {} is running.",
            service_name, endpoint, service_name
        ))
    }
    // Focus on network and parsing errors, skip auth/quota
}
```

**4. Testing Strategy for Local Services**
```rust
// Unit tests: Point to non-existent ports (localhost:99999)
// Integration tests: Require running service with #[ignore] annotations
// Clear separation between testable logic and service dependencies

#[tokio::test]
#[ignore = "requires running Ollama service with llama3.2 model"]
async fn test_integration_with_real_service() { }
```

## Significance

**For Architecture**: 
- Local providers require different error handling patterns than cloud providers
- Service discovery integration is critical for local provider usability
- Testing strategies must accommodate optional external dependencies

**For Implementation**:
- Error messages should include service startup guidance
- Default endpoints should match standard service configurations
- Integration tests need clear dependency documentation

**For User Experience**:
- Users need clear feedback when local services aren't running
- Provider should fail fast and clearly when service unavailable
- Documentation should include service setup guidance

## Cross-References
- **Related**: [[TDD Provider Implementation Pattern]]
- **Related**: [[Provider HTTP Error Mapping Guide]]
- **Related**: [[Local Provider Foundation]] (service discovery)
- **Enables**: Future local provider implementations (LMStudio, others)
- **Informs**: Provider testing strategies and user documentation

## Implementation Evidence
- **Ollama Provider**: Successfully implemented following these patterns
- **Test Coverage**: 11 unit tests (no external deps) + 5 integration tests (service required)
- **Error Handling**: Network errors provide actionable guidance about service availability
- **Service Integration**: Clean integration with existing service discovery foundation

## Future Applications
- **LMStudio Provider**: Can follow identical patterns with different default port
- **Other Local Services**: Hugging Face Transformers local, vLLM, text-generation-webui
- **Service Management**: Could inform automatic service startup/management features
- **Documentation**: Should influence user guides for local provider setup

---

**Validation Status**: Confirmed through successful Ollama implementation  
**Next Steps**: Apply patterns to LMStudio provider implementation  
**Documentation Impact**: Should update provider implementation guides with local-specific considerations