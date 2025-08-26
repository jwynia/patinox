# Task: Add Cache Expiration Strategy for Model Information

## Priority: Medium
**Category**: Technical Debt  
**Effort Estimate**: Small (15-30 minutes)  
**Created**: 2025-08-25 20:45 CDT  
**Source**: Code Review - apply-recommendations  

## Problem Statement

The LMStudio provider includes a model cache (`model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>`) that is allocated but never effectively utilized. This represents unused infrastructure that should either be implemented properly or removed to avoid confusion.

### Current Implementation Issues

**File**: `src/provider/local/lmstudio.rs:154`

```rust
/// Cached model information  
model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
```

The cache field exists but:
- Never populated with data
- Never consulted for lookups
- No expiration or invalidation strategy
- Adds memory overhead without benefit

## Acceptance Criteria

### Must Have
- [ ] Define cache expiration policy (time-based recommended)
- [ ] Implement cache population in `list_models()`
- [ ] Add cache invalidation mechanism
- [ ] Document cache behavior and limitations

### Should Have
- [ ] Configurable cache TTL (time-to-live)
- [ ] Cache statistics/metrics
- [ ] Thread-safe cache operations
- [ ] Memory-efficient cache management

### Could Have
- [ ] Cache warming on initialization
- [ ] Adaptive cache TTL based on API performance
- [ ] Cache persistence across restarts

## Technical Design

### Cache Expiration Strategy

**Recommended Approach**: Time-based expiration with manual invalidation

```rust
struct CacheEntry {
    model_info: ModelInfo,
    cached_at: Instant,
    ttl: Duration,
}

impl LMStudioProvider {
    const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
    
    async fn is_cache_valid(&self) -> bool {
        let cache_timestamp = *self.cache_timestamp.read().await;
        match cache_timestamp {
            Some(timestamp) => timestamp.elapsed() < Self::DEFAULT_CACHE_TTL,
            None => false,
        }
    }
}
```

### Implementation Strategy

1. **Add Cache Timestamp Tracking**:
   ```rust
   pub struct LMStudioProvider {
       // ... existing fields
       model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
       cache_timestamp: Arc<RwLock<Option<Instant>>>,
   }
   ```

2. **Cache Population**:
   ```rust
   async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
       // Check cache validity first
       if self.is_cache_valid().await && !self.model_cache.read().await.is_empty() {
           return Ok(self.model_cache.read().await.values().cloned().collect());
       }
       
       // Fetch from API and populate cache
       let response: LMStudioModelsResponse = self.make_request("/v1/models").await?;
       let models = /* ... existing transformation logic ... */;
       
       // Update cache
       let mut cache = self.model_cache.write().await;
       let mut timestamp = self.cache_timestamp.write().await;
       
       cache.clear();
       for model in &models {
           cache.insert(model.name.clone(), model.clone());
       }
       *timestamp = Some(Instant::now());
       
       Ok(models)
   }
   ```

3. **Cache Utilization**:
   - This task enables the high-priority caching task
   - Provides foundation for `supports_model()` and `model_capabilities()` optimization

## Testing Requirements

### Unit Tests to Add
- [ ] Cache TTL expiration behavior
- [ ] Cache population on `list_models()`
- [ ] Cache hit when valid
- [ ] Cache miss when expired
- [ ] Thread safety under concurrent access

### Performance Tests
- [ ] Cache hit vs API call latency
- [ ] Memory usage with cached models
- [ ] Cache invalidation performance

## Dependencies

- **Requires**: Existing model cache infrastructure
- **Blocks**: High-priority model caching optimization task  
- **Related**: Performance optimization initiatives

## Risks & Mitigations

### Risk 1: Cache Staleness
**Issue**: Cached data may not reflect actual LMStudio model state
**Mitigation**: Conservative TTL (5 minutes), manual invalidation methods

### Risk 2: Memory Usage
**Issue**: Cache could consume excessive memory with many models
**Mitigation**: Monitor cache size, implement size limits if needed

### Risk 3: Thread Safety
**Issue**: Concurrent cache updates could cause issues
**Mitigation**: Leverage existing `Arc<RwLock>` pattern correctly

## Implementation Notes

- Build on existing cache infrastructure rather than replacing
- Use `std::time::Instant` for cache timestamps (not system time)  
- Consider making TTL configurable via environment variables
- Add appropriate error handling for cache operations
- Document cache behavior in module documentation

## Success Metrics

- **Functionality**: Cache is populated and expires correctly
- **Performance**: Foundation for eliminating redundant API calls
- **Reliability**: No regressions in model listing functionality
- **Code Quality**: Clean, well-tested cache management code

## Future Enhancements

After this implementation:
- Integration with high-priority model lookup optimization
- Advanced cache strategies (LRU, adaptive TTL)
- Cache metrics and observability
- Configuration-driven cache behavior

---

**Created by**: Code Review Recommendation  
**Related to**: LMStudio Provider Technical Debt  
**Status**: Open