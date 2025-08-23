# Extract Common HTTP Request Logic

## Task Overview
**Priority**: High  
**Effort**: Medium (30-45 minutes)  
**Risk**: Medium  
**Source**: Code Review Recommendation

## Background
The Ollama provider has duplicate error handling and URL construction logic between `make_request()` and `make_post_request()` methods. This creates maintenance overhead and violates the DRY principle.

## Current State
**Duplication Identified** ❌:
- URL formatting: `format!("{}{}", self.base_url, path)` repeated
- Error handling patterns duplicated across both methods
- Response processing logic nearly identical
- Status code checking repeated

**Impact**:
- ~40 lines of duplicate code
- Maintenance burden when updating error handling
- Risk of inconsistent behavior between GET and POST requests

## Acceptance Criteria

### Core Functionality
- [ ] Extract common request handling logic into `execute_request` helper method
- [ ] Eliminate duplicate URL formatting code
- [ ] Consolidate error handling patterns
- [ ] Maintain existing API surface unchanged

### Quality Standards
- [ ] All existing tests continue to pass
- [ ] No change in behavior for calling code
- [ ] Clean separation between request building and execution
- [ ] Error messages remain informative and context-specific

### Implementation Approach
- [ ] Create `execute_request<T>(&self, request: reqwest::RequestBuilder) -> ProviderResult<T>`
- [ ] Update `make_request` to use common helper
- [ ] Update `make_post_request` to use common helper
- [ ] Ensure generic type constraints remain appropriate

## Suggested Implementation

### Phase 1: Create Common Helper
```rust
async fn execute_request<T>(&self, request: reqwest::RequestBuilder) -> ProviderResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let response = request
        .send()
        .await
        .map_err(|e| ProviderError::NetworkError(format!("Failed to connect to Ollama: {}", e)))?;

    if !response.status().is_success() {
        return Err(ProviderError::NetworkError(format!(
            "Ollama API returned status: {}",
            response.status()
        )));
    }

    response
        .json()
        .await
        .map_err(|e| ProviderError::ApiError(format!("Failed to parse Ollama response: {}", e)))
}
```

### Phase 2: Refactor Existing Methods
```rust
async fn make_request<T>(&self, path: &str) -> ProviderResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let url = format!("{}{}", self.base_url, path);
    self.execute_request(self.client.get(&url)).await
}

async fn make_post_request<T, B>(&self, path: &str, body: &B) -> ProviderResult<T>
where
    T: serde::de::DeserializeOwned,
    B: serde::Serialize,
{
    let url = format!("{}{}", self.base_url, path);
    self.execute_request(self.client.post(&url).json(body)).await
}
```

## Files to Modify
- `src/provider/local/ollama.rs` - Extract common logic from lines 104-161

## Testing Requirements
- [ ] Run all existing Ollama provider tests
- [ ] Verify no behavior changes in error handling
- [ ] Confirm both GET and POST requests work correctly
- [ ] Test error conditions still return appropriate error types

## Success Metrics
- Code duplication reduced by ~40 lines
- Single source of truth for request error handling
- Consistent behavior between GET and POST operations
- All existing tests continue to pass

## Dependencies
- **Blocked by**: None
- **Blocks**: None
- **Related**: Other provider implementations might benefit from similar patterns

## Metadata
- **Created**: 2025-08-23 12:21 CDT
- **Source**: Code review recommendation from comprehensive review
- **Category**: Refactoring/Technical Debt
- **Estimated Duration**: 45-60 minutes including testing