# Provider HTTP Error Mapping Guide

## Classification
- **Domain**: Implementation Patterns
- **Stability**: Established
- **Abstraction**: Technical Guide
- **Confidence**: High

## Overview

Standardized approach for mapping external API errors to `ProviderError` variants across all provider implementations. This ensures consistent error handling behavior and user experience across different LLM providers.

## Error Mapping Hierarchy

### Network-Level Errors → `ProviderError::NetworkError`

**Connection Failures**:
```rust
.map_err(|e| match e {
    reqwest::Error if e.is_connect() => 
        ProviderError::NetworkError(format!("Failed to connect to [Service]: {}", e)),
    reqwest::Error if e.is_timeout() => 
        ProviderError::NetworkError(format!("Request timeout: {}", e)),
    _ => ProviderError::NetworkError(format!("Network error: {}", e))
})
```

**HTTP Status Errors**:
```rust
if !response.status().is_success() {
    return Err(ProviderError::NetworkError(format!(
        "[Service] API returned status: {}",
        response.status()
    )));
}
```

### API Response Errors → `ProviderError::ApiError`

**JSON Parsing Failures**:
```rust
let body: ResponseType = response
    .json()
    .await
    .map_err(|e| ProviderError::ApiError(format!("Failed to parse [Service] response: {}", e)))?;
```

**Malformed API Responses**:
```rust
// When required fields are missing or invalid
if response.model.is_empty() {
    return Err(ProviderError::ApiError(
        "Service returned empty model name".to_string()
    ));
}
```

### Request Validation Errors → `ProviderError::InvalidRequest`

**Pre-API Validation**:
```rust
// Validate before making HTTP request
if request.model.name().is_empty() {
    return Err(ProviderError::InvalidRequest("Model name cannot be empty".to_string()));
}

if request.messages.is_empty() {
    return Err(ProviderError::InvalidRequest("Messages cannot be empty".to_string()));
}
```

## Service-Specific Error Mapping

### Authentication Errors → `ProviderError::AuthenticationError`
```rust
match response.status() {
    reqwest::StatusCode::UNAUTHORIZED => 
        Err(ProviderError::AuthenticationError("Invalid API key".to_string())),
    reqwest::StatusCode::FORBIDDEN =>
        Err(ProviderError::AuthenticationError("Access forbidden".to_string())),
    // ... other status codes
}
```

### Rate Limiting → `ProviderError::RateLimited`
```rust
match response.status() {
    reqwest::StatusCode::TOO_MANY_REQUESTS => {
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs);
        
        Err(ProviderError::RateLimited { retry_after })
    }
}
```

### Model Availability → `ProviderError::ModelNotAvailable`
```rust
// When service reports model doesn't exist or isn't loaded
if error_contains("model not found") {
    return Err(ProviderError::ModelNotAvailable {
        model: request.model.name().to_string()
    });
}
```

## Error Context Guidelines

### Include Service Context
Always include the service name in error messages for clarity:
```rust
// GOOD: Clear service context
ProviderError::NetworkError(format!("Failed to connect to Ollama: {}", e))

// BAD: Generic error without context
ProviderError::NetworkError(format!("Connection failed: {}", e))
```

### Preserve Original Error Information
Include underlying error details when helpful for debugging:
```rust
// GOOD: Includes underlying error
ProviderError::ApiError(format!("Failed to parse Ollama response: {}", e))

// AVOID: Too much technical detail exposed to end users
ProviderError::ApiError(format!("JSON deserialization failed at byte 1247: {}", e))
```

### Provide Actionable Context
When possible, suggest what the user can do:
```rust
// GOOD: Actionable guidance
ProviderError::NetworkError("Ollama service not available at http://localhost:11434. Ensure Ollama is running.".to_string())

// BAD: Just reports the problem
ProviderError::NetworkError("Connection refused".to_string())
```

## Testing Error Mapping

### Unit Test Pattern
```rust
#[tokio::test]
async fn test_error_mapping_for_service_unavailable() {
    // Arrange - provider pointing to non-existent service
    let provider = Provider::with_endpoint("http://localhost:99999".to_string())
        .expect("Should create provider");

    // Act
    let result = provider.some_operation().await;

    // Assert - verify specific error type
    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::NetworkError(_) => {
            // Expected error type for network failures
        }
        other => panic!("Expected NetworkError, got {:?}", other),
    }
}
```

### Integration Test Considerations
```rust
#[tokio::test]
#[ignore = "requires authentication failure testing"]
async fn test_authentication_error_mapping() {
    // Test with invalid API key to verify auth error mapping
    let provider = Provider::new("invalid-key")?;
    let result = provider.list_models().await;
    
    match result.unwrap_err() {
        ProviderError::AuthenticationError(_) => {
            // Verify auth errors are mapped correctly
        }
        other => panic!("Expected AuthenticationError, got {:?}", other),
    }
}
```

## Common Patterns by Provider Type

### Cloud Providers (OpenAI, Anthropic, etc.)
- Emphasize authentication and rate limiting errors
- Handle API versioning issues
- Map quota/billing errors appropriately

### Local Providers (Ollama, LMStudio, etc.)
- Focus on service availability (connection refused)
- Handle model loading/availability issues
- Less emphasis on rate limiting and authentication

### Router Providers (OpenRouter, etc.)
- Handle upstream provider failures
- Map provider selection issues
- Aggregate multiple provider error types

## Error Message Guidelines

### Structure
```
[Context]: [Problem Description]
```

### Examples
```rust
// Good error messages:
"Failed to connect to Ollama: connection refused"
"Ollama API returned status: 404"
"Model 'llama3.2' not available on this Ollama instance"
"Invalid request: model name cannot be empty"

// Avoid generic messages:
"Request failed"
"Error occurred"  
"Something went wrong"
```

## Related Patterns
- **TDD Provider Implementation**: Error mapping is tested first
- **Provider Abstraction Compliance**: All errors map to defined variants
- **Service Integration Testing**: Verify error mapping with real services

## Validation
- **Ollama Provider**: Comprehensive error mapping implemented and tested
- **Anthropic Provider**: Authentication and API errors properly mapped
- **OpenRouter Provider**: Multi-provider error aggregation patterns

---

*This guide establishes consistent error handling across all provider implementations and should be followed for all new provider development.*