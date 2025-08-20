# Add Monitoring for Drop Implementation Failures

## Problem
The current Drop implementation for AsyncResourceGuard uses tokio::spawn which could fail silently, potentially leading to resource leaks without visibility.

## Current Implementation
```rust
impl<T: Send + 'static> Drop for AsyncResourceGuard<T> {
    fn drop(&mut self) {
        if let (Some(resource), Some(cleanup)) = (self.resource.take(), self.cleanup.take()) {
            tokio::spawn(async move {
                if let Err(e) = cleanup(resource).await {
                    log::error!("Resource cleanup failed: {}", e);
                }
            });
        }
    }
}
```

## Issues
- tokio::spawn could fail if runtime is shutting down
- No visibility into failed spawn attempts
- No fallback mechanism for cleanup
- Potential resource leaks go undetected

## Proposed Solutions

### Option 1: Add spawn failure monitoring
```rust
impl<T: Send + 'static> Drop for AsyncResourceGuard<T> {
    fn drop(&mut self) {
        if let (Some(resource), Some(cleanup)) = (self.resource.take(), self.cleanup.take()) {
            let resource_id = self.resource_id;
            
            match tokio::spawn(async move {
                if let Err(e) = cleanup(resource).await {
                    log::error!("Resource cleanup failed for {}: {}", resource_id, e);
                }
            }) {
                Ok(_) => {
                    // Spawn successful, cleanup will happen asynchronously
                }
                Err(e) => {
                    // Spawn failed - runtime likely shutting down
                    log::error!("Failed to spawn cleanup task for resource {}: {}", resource_id, e);
                    // TODO: Add metrics/monitoring for this failure
                    // Consider synchronous cleanup as fallback
                }
            }
        }
    }
}
```

### Option 2: Add fallback synchronous cleanup
```rust
impl<T: Send + 'static> Drop for AsyncResourceGuard<T> {
    fn drop(&mut self) {
        if let (Some(resource), Some(cleanup)) = (self.resource.take(), self.cleanup.take()) {
            let resource_id = self.resource_id;
            
            // Try async cleanup first
            if tokio::spawn(async move {
                if let Err(e) = cleanup(resource).await {
                    log::error!("Resource cleanup failed for {}: {}", resource_id, e);
                }
            }).is_err() {
                // Async spawn failed, try sync cleanup if possible
                log::warn!("Async cleanup spawn failed for {}, attempting synchronous cleanup", resource_id);
                
                // This would require a synchronous cleanup API
                // TODO: Add sync cleanup support
            }
        }
    }
}
```

### Option 3: Integration with ResourceRegistry
```rust
// Add to ResourceRegistry
pub fn register_drop_cleanup(&self, resource_id: ResourceId, cleanup: CleanupFn) {
    // Handle cleanup through registry system
    // This provides better monitoring and coordination
}

// Update AsyncResourceGuard to use registry when available
impl<T: Send + 'static> Drop for AsyncResourceGuard<T> {
    fn drop(&mut self) {
        if let (Some(resource), Some(cleanup)) = (self.resource.take(), self.cleanup.take()) {
            if let Some(registry) = &self.registry {
                // Use registry for coordinated cleanup
                registry.register_drop_cleanup(self.resource_id, cleanup);
            } else {
                // Fallback to direct spawn with monitoring
                self.spawn_with_monitoring(resource, cleanup);
            }
        }
    }
}
```

## Acceptance Criteria
- [ ] Add monitoring for tokio::spawn failures
- [ ] Consider fallback mechanisms for cleanup
- [ ] Add metrics for resource leak detection
- [ ] Update tests to verify failure handling
- [ ] Document cleanup behavior guarantees
- [ ] Consider integration with ResourceRegistry
- [ ] Add logging for troubleshooting

## Dependencies
- Monitoring system integration
- Metrics collection infrastructure
- Consider ResourceRegistry integration design

## Implementation Notes
- Need to balance between reliability and complexity
- Consider runtime shutdown scenarios
- May need to add synchronous cleanup support
- Should coordinate with monitoring system design

## Priority: Medium
**Risk**: Monitoring system integration
**Impact**: Better visibility into resource management failures
**Effort**: 2-4 hours including monitoring integration