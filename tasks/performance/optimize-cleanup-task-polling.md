# Optimize Cleanup Task Polling Mechanism

## Problem
The current cleanup task in ResourceRegistry uses a timeout-based polling approach that can lead to busy-waiting and resource contention under high load. The 10ms timeout could cause unnecessary CPU usage and reduce system efficiency.

## Current Implementation
```rust
// Try to receive new requests without blocking too long
match tokio::time::timeout(
    std::time::Duration::from_millis(CLEANUP_POLL_INTERVAL_MS),
    cleanup_rx.recv()
).await {
    Ok(Some(request)) => {
        // Add to priority queue
        pending.push(Reverse(request));
    }
    Ok(None) => {
        // Channel closed, shutdown
        break;
    }
    Err(_) => {
        // Timeout, continue to process pending
    }
}
```

## Proposed Solution
Use tokio's `select!` macro for more efficient event-driven processing:

```rust
use tokio::select;
use tokio::time::{interval, Duration};

fn start_cleanup_task(...) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut pending = BinaryHeap::new();
        let mut process_interval = interval(Duration::from_millis(100));
        
        loop {
            if shutdown.load(Ordering::Relaxed) {
                // Process remaining requests
                while let Some(Reverse(request)) = pending.pop() {
                    Self::process_cleanup_request(request, &active, &monitor, &cleanup_count).await;
                }
                break;
            }
            
            select! {
                // Receive new cleanup requests
                request = cleanup_rx.recv() => {
                    match request {
                        Some(req) => pending.push(Reverse(req)),
                        None => break, // Channel closed
                    }
                }
                // Process pending requests periodically
                _ = process_interval.tick() => {
                    if let Some(Reverse(request)) = pending.pop() {
                        Self::process_cleanup_request(request, &active, &monitor, &cleanup_count).await;
                    }
                }
            }
        }
    })
}
```

## Benefits
- Eliminates busy-waiting and reduces CPU usage
- More responsive to incoming cleanup requests
- Better resource utilization under varying loads
- Cleaner separation of concerns (receiving vs processing)

## Acceptance Criteria
- [ ] Replace timeout polling with select! macro
- [ ] Maintain existing functionality and behavior
- [ ] Add performance benchmarks to validate improvement
- [ ] Update all related tests to ensure correct behavior
- [ ] Verify graceful shutdown still works correctly
- [ ] Test under various load conditions
- [ ] Document the new approach

## Testing Requirements
- [ ] Unit tests for the new polling mechanism
- [ ] Load testing to verify performance improvement
- [ ] Stress testing with high cleanup request volume
- [ ] Shutdown behavior verification
- [ ] Integration tests with ResourceRegistry

## Implementation Notes
- Consider making the processing interval configurable
- Ensure proper error handling in select branches
- May need to adjust other timeout values for consistency
- Profile before and after to measure improvement

## Priority: Medium
**Risk**: Concurrent behavior change
**Impact**: Better system performance and resource utilization
**Effort**: 3-5 hours including benchmarking and testing