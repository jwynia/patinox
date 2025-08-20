# Refactor Large Cleanup Task Function

## Problem
The `start_cleanup_task` function in ResourceRegistry is 47 lines long and handles multiple concerns, making it harder to read, test, and maintain.

## Current Implementation
The function currently handles:
1. Task loop setup
2. Shutdown detection
3. Request receiving with timeout
4. Request processing
5. Shutdown cleanup

## Proposed Refactoring

### Split into focused functions:

```rust
impl ResourceRegistry {
    fn start_cleanup_task(
        mut cleanup_rx: mpsc::UnboundedReceiver<CleanupRequest>,
        active: Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
        monitor: Arc<dyn Monitor>,
        shutdown: Arc<AtomicBool>,
        cleanup_count: Arc<AtomicU32>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut pending = BinaryHeap::new();
            
            loop {
                if shutdown.load(Ordering::Relaxed) {
                    Self::process_shutdown_cleanup(&mut pending, &active, &monitor, &cleanup_count).await;
                    break;
                }
                
                match Self::receive_cleanup_request(&mut cleanup_rx).await {
                    RequestResult::NewRequest(request) => {
                        pending.push(Reverse(request));
                    }
                    RequestResult::ChannelClosed => break,
                    RequestResult::Timeout => {
                        // Continue to process pending
                    }
                }
                
                Self::process_next_pending_request(&mut pending, &active, &monitor, &cleanup_count).await;
            }
        })
    }
    
    async fn receive_cleanup_request(
        cleanup_rx: &mut mpsc::UnboundedReceiver<CleanupRequest>
    ) -> RequestResult {
        match tokio::time::timeout(
            std::time::Duration::from_millis(CLEANUP_POLL_INTERVAL_MS),
            cleanup_rx.recv()
        ).await {
            Ok(Some(request)) => RequestResult::NewRequest(request),
            Ok(None) => RequestResult::ChannelClosed,
            Err(_) => RequestResult::Timeout,
        }
    }
    
    async fn process_next_pending_request(
        pending: &mut BinaryHeap<Reverse<CleanupRequest>>,
        active: &Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
        monitor: &Arc<dyn Monitor>,
        cleanup_count: &Arc<AtomicU32>,
    ) {
        if let Some(Reverse(request)) = pending.pop() {
            Self::process_cleanup_request(request, active, monitor, cleanup_count).await;
        }
    }
    
    async fn process_shutdown_cleanup(
        pending: &mut BinaryHeap<Reverse<CleanupRequest>>,
        active: &Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
        monitor: &Arc<dyn Monitor>,
        cleanup_count: &Arc<AtomicU32>,
    ) {
        // Process remaining requests with higher priority first
        while let Some(Reverse(request)) = pending.pop() {
            Self::process_cleanup_request(request, active, monitor, cleanup_count).await;
        }
    }
}

enum RequestResult {
    NewRequest(CleanupRequest),
    ChannelClosed,
    Timeout,
}
```

## Benefits
- Each function has a single responsibility
- Easier to test individual components
- More readable and maintainable code
- Better error handling isolation
- Clearer control flow

## Acceptance Criteria
- [ ] Split `start_cleanup_task` into focused functions
- [ ] Maintain existing functionality and behavior
- [ ] Add unit tests for individual functions where possible
- [ ] Update integration tests to ensure no regressions
- [ ] Improve code documentation for each function
- [ ] Consider making helper functions private

## Testing Strategy
- [ ] Test each extracted function independently
- [ ] Verify overall behavior remains unchanged
- [ ] Add specific tests for edge cases in each function
- [ ] Test shutdown behavior thoroughly

## Implementation Notes
- Keep the public API unchanged
- Consider using private helper functions vs associated functions
- May want to group related functionality into sub-modules
- Ensure proper error propagation between functions

## Priority: Low-Medium
**Risk**: Low (refactoring only)
**Impact**: Better code maintainability and testability
**Effort**: 1-2 hours including testing