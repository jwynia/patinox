# Refactor fetch_models Function Length

## Context
The `fetch_models()` function in `ServiceDiscovery` is over 40 lines and handles multiple responsibilities:
1. Building service-specific endpoints
2. Making HTTP requests with timeout
3. Processing response status
4. Extracting response text
5. Parsing service-specific JSON

## Current Function Location
`/workspaces/patinox/src/provider/local/discovery.rs:285-325`

## Objective
Break the function into smaller, focused methods while maintaining error handling quality.

## Proposed Breakdown
```rust
async fn fetch_models(&self, service_type: &ServiceType, endpoint: &str) -> LocalProviderResult<Vec<String>> {
    let models_endpoint = self.build_models_endpoint(service_type, endpoint);
    let response_text = self.fetch_models_response(&models_endpoint).await?;
    self.parse_models_response(service_type, &response_text)
}

fn build_models_endpoint(&self, service_type: &ServiceType, endpoint: &str) -> String { /* ... */ }

async fn fetch_models_response(&self, endpoint: &str) -> LocalProviderResult<String> { /* ... */ }

fn parse_models_response(&self, service_type: &ServiceType, response: &str) -> LocalProviderResult<Vec<String>> { /* ... */ }
```

## Acceptance Criteria  
- [ ] Original `fetch_models` function is under 20 lines
- [ ] Each extracted method has single responsibility
- [ ] All error handling is preserved
- [ ] Error messages maintain current context and quality
- [ ] All existing tests continue to pass
- [ ] No performance regression

## Implementation Notes
- Keep the async nature only where needed (HTTP request)
- Maintain the same error types and messages  
- Consider if the helper methods should be private
- Ensure timeout handling stays with the HTTP request

## Estimated Effort
30 minutes

## Priority
Medium - Code maintainability improvement