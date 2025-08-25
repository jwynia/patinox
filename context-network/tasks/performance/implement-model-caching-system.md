# Implement Model Caching System

## Task Overview
**Priority**: High  
**Effort**: Medium (45-60 minutes)  
**Risk**: Medium  
**Source**: Code Review Recommendation

## Background
The Ollama provider has an unused `model_cache` field and makes redundant API calls in `supports_model()` and `model_capabilities()` methods. Each call to these methods triggers a new HTTP request to `/api/tags`, which is inefficient for repeated model queries.

## Current State
**Performance Issue** ❌:
- `model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>` exists but never used
- `supports_model()` calls `list_models().await` (HTTP request)  
- `model_capabilities()` calls `list_models().await` (another HTTP request)
- No cache invalidation or refresh mechanism

**Impact**:
- Unnecessary network overhead for repeated model queries
- Slower response times for model validation
- Unused memory allocation for cache structure

## Acceptance Criteria

### Core Functionality
- [ ] Implement cache population in `list_models()` method
- [ ] Create `get_cached_models()` helper method
- [ ] Update `supports_model()` to use cache
- [ ] Update `model_capabilities()` to use cache

### Cache Management
- [ ] Implement cache invalidation strategy (TTL or manual refresh)
- [ ] Handle cache misses gracefully
- [ ] Provide cache refresh mechanism
- [ ] Ensure thread-safe cache access

### Quality Standards
- [ ] All existing tests continue to pass
- [ ] Cache behavior is predictable and documented
- [ ] No race conditions in cache access
- [ ] Error handling maintains existing patterns

## Implementation Approach

### Phase 1: Cache Infrastructure
```rust
impl OllamaProvider {
    /// Get models from cache or fetch from API if cache is empty/stale
    async fn get_cached_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        // Try to read from cache first
        {
            let cache = self.model_cache.read().await;
            if !cache.is_empty() {
                return Ok(cache.values().cloned().collect());
            }
        }
        
        // Cache miss - fetch from API and populate cache
        self.refresh_model_cache().await
    }
    
    /// Refresh the model cache from the Ollama API
    async fn refresh_model_cache(&self) -> ProviderResult<Vec<ModelInfo>> {
        let response: OllamaTagsResponse = self.make_request("/api/tags").await?;
        
        let models: Vec<ModelInfo> = response.models
            .into_iter()
            .map(|ollama_model| {
                // ... existing model creation logic
            })
            .collect();
        
        // Update cache
        {
            let mut cache = self.model_cache.write().await;
            cache.clear();
            for model in &models {
                cache.insert(model.id.name().to_string(), model.clone());
            }
        }
        
        Ok(models)
    }
}
```

### Phase 2: Update Public Methods
```rust
async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
    // Use cached models or fetch fresh
    self.get_cached_models().await
}

async fn supports_model(&self, model: &ModelId) -> bool {
    // Use cache instead of making API call
    match self.get_cached_models().await {
        Ok(models) => models.iter().any(|m| m.id.name() == model.name()),
        Err(_) => false,
    }
}

async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
    // Use cache instead of making API call
    match self.get_cached_models().await {
        Ok(models) => models
            .into_iter()
            .find(|m| m.id.name() == model.name())
            .map(|m| m.capabilities),
        Err(_) => None,
    }
}
```

### Phase 3: Cache Management (Optional Enhancement)
Consider adding cache invalidation:
- Time-based TTL (Time To Live)
- Manual refresh method
- Cache size limits
- Cache statistics

## Files to Modify
- `src/provider/local/ollama.rs` - Implement caching logic in methods around lines 239-338

## Testing Requirements
- [ ] Test cache population on first `list_models()` call
- [ ] Verify `supports_model()` uses cache (no HTTP call on second invocation)
- [ ] Verify `model_capabilities()` uses cache (no HTTP call on second invocation)
- [ ] Test cache refresh after explicit invalidation
- [ ] Test concurrent access to cache doesn't cause race conditions
- [ ] Test error handling when cache is populated but API fails on refresh

## Performance Benefits
- **Reduced Network Calls**: Multiple model queries use single API call
- **Faster Response Times**: Cache hits avoid network round-trip
- **Better Resource Utilization**: Existing cache structure actually used
- **Improved User Experience**: Faster model validation and capability queries

## Success Metrics
- Cache hit rate > 80% for repeated model queries
- Response time improvement for `supports_model()` and `model_capabilities()`
- No increase in memory usage beyond existing allocation
- All existing tests continue to pass

## Considerations
- **Cache Staleness**: Models may be added/removed from Ollama without cache knowing
- **Memory Usage**: Cache grows with number of models but should be bounded
- **Thread Safety**: Multiple concurrent requests must handle cache safely
- **Error Recovery**: Cache errors shouldn't break core functionality

## Dependencies
- **Blocked by**: None
- **Blocks**: None  
- **Related**: Could inform caching patterns for other providers

## Metadata
- **Created**: 2025-08-23 12:21 CDT
- **Source**: Code review recommendation for performance improvement
- **Category**: Performance/Technical Debt
- **Estimated Duration**: 1-1.5 hours including testing and documentation