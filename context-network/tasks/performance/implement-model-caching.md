# Task: Implement Model Caching for LMStudio Provider

## Priority: High
**Category**: Performance Optimization  
**Effort Estimate**: Medium (30-60 minutes)  
**Created**: 2025-08-25 20:45 CDT  
**Source**: Code Review - apply-recommendations  

## Problem Statement

The LMStudio provider's `supports_model()` and `model_capabilities()` methods currently call `list_models()` on every invocation, resulting in redundant HTTP API calls to the LMStudio service. This creates unnecessary network overhead and potential performance bottlenecks.

### Current Implementation Issues

**File**: `src/provider/local/lmstudio.rs:332-348`

```rust
async fn supports_model(&self, model: &ModelId) -> bool {
    match self.list_models().await {  // API call every time
        Ok(models) => models.iter().any(|m| m.id.name() == model.name()),
        Err(_) => false,
    }
}

async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
    match self.list_models().await {  // Another API call
        Ok(models) => models.into_iter().find(|m| m.id.name() == model.name()).map(|m| m.capabilities),
        Err(_) => None,
    }
}
```

### Impact Analysis

- **Performance**: Multiple API calls for simple lookups
- **Network Load**: Unnecessary HTTP requests to LMStudio
- **User Experience**: Slower response times for model queries
- **Resource Usage**: Increased memory allocation for repeated model list parsing

## Acceptance Criteria

### Must Have
- [ ] Model cache is populated during `list_models()` calls
- [ ] `supports_model()` checks cache before making API calls
- [ ] `model_capabilities()` uses cached data when available
- [ ] Cache is thread-safe (proper use of `Arc<RwLock>`)
- [ ] Fallback to API call if cache is empty or stale
- [ ] All existing tests continue to pass

### Should Have  
- [ ] Cache expiration strategy (time-based or manual invalidation)
- [ ] Cache hit/miss metrics for observability
- [ ] Efficient cache key strategy (model name vs full ModelId)
- [ ] Cache warming on provider initialization

### Could Have
- [ ] Configurable cache TTL
- [ ] Cache size limits to prevent memory growth
- [ ] Background cache refresh
- [ ] Cache persistence across provider instances

## Technical Design

### Cache Architecture
```rust
struct LMStudioProvider {
    // ... existing fields
    model_cache: Arc<RwLock<HashMap<String, CachedModelInfo>>>,
    cache_expiry: Arc<RwLock<Option<Instant>>>,
}

struct CachedModelInfo {
    model_info: ModelInfo,
    cached_at: Instant,
}
```

### Implementation Strategy

1. **Cache Population**:
   - Modify `list_models()` to populate cache after successful API call
   - Use model name as cache key for efficient lookups
   - Store both ModelInfo and timestamp

2. **Cache Utilization**:
   - Update `supports_model()` to check cache first
   - Update `model_capabilities()` to use cached data
   - Implement cache miss fallback to API

3. **Cache Invalidation**:
   - Simple time-based expiration (5-10 minutes)
   - Manual cache clearing method for testing
   - Consider cache invalidation on API errors

### Example Implementation

```rust
impl LMStudioProvider {
    async fn supports_model(&self, model: &ModelId) -> bool {
        // Check cache first
        {
            let cache = self.model_cache.read().await;
            if let Some(cached_info) = cache.get(model.name()) {
                if !self.is_cache_expired().await {
                    return true;
                }
            }
        }
        
        // Cache miss - refresh cache and check
        match self.refresh_model_cache().await {
            Ok(_) => {
                let cache = self.model_cache.read().await;
                cache.contains_key(model.name())
            },
            Err(_) => false,
        }
    }
    
    async fn refresh_model_cache(&self) -> ProviderResult<()> {
        let models = self.fetch_models_from_api().await?;
        
        let mut cache = self.model_cache.write().await;
        cache.clear();
        
        for model in models {
            cache.insert(model.name.clone(), CachedModelInfo {
                model_info: model,
                cached_at: Instant::now(),
            });
        }
        
        Ok(())
    }
}
```

## Testing Requirements

### Unit Tests to Add
- [ ] Cache population on successful `list_models()`
- [ ] Cache hit scenarios for `supports_model()` and `model_capabilities()`
- [ ] Cache miss fallback behavior
- [ ] Cache expiration handling
- [ ] Thread safety under concurrent access

### Performance Tests
- [ ] Benchmark cache hit vs API call performance
- [ ] Memory usage validation
- [ ] Concurrent access performance

## Dependencies

- **Requires**: Existing LMStudio provider implementation
- **Blocks**: None
- **Related**: None

## Risks & Mitigations

### Risk 1: Cache Consistency
**Issue**: Cache may become stale if models change on LMStudio server
**Mitigation**: Implement reasonable cache expiration (5-10 minutes)

### Risk 2: Memory Usage
**Issue**: Cache could grow unbounded with many models
**Mitigation**: Add cache size limits or periodic cleanup

### Risk 3: Thread Safety
**Issue**: Concurrent cache access could cause data races
**Mitigation**: Proper use of existing `Arc<RwLock>` pattern

## Implementation Notes

- Leverage existing `model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>` field
- Follow existing patterns from other providers in codebase
- Ensure changes are backwards compatible
- Add appropriate error handling for cache operations

## Success Metrics

- **Performance**: > 90% reduction in API calls for repeated model queries
- **Response Time**: < 1ms for cached model lookups vs ~100ms+ for API calls
- **Test Coverage**: All cache paths covered by unit tests
- **Reliability**: No regressions in existing functionality

---

**Created by**: Code Review Recommendation  
**Related to**: LMStudio Provider Performance Optimization  
**Status**: Open  