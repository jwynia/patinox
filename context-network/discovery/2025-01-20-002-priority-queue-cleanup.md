# Discovery: BinaryHeap Priority Queue for Resource Cleanup

**Date**: 2025-01-20  
**Context**: ResourceRegistry Implementation  
**Significance**: Efficient priority-based resource cleanup coordination

## The Problem

Resource cleanup operations have different priorities (Critical > High > Normal > Low), but standard queues process in FIFO order. Need priority-based processing while maintaining simplicity and performance.

## Solution Discovered

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

// BinaryHeap is a max-heap, but we want higher priority values first
let mut pending: BinaryHeap<Reverse<CleanupRequest>> = BinaryHeap::new();

// Add requests (wrapped in Reverse for correct ordering)
pending.push(Reverse(high_priority_request));
pending.push(Reverse(low_priority_request));

// Process highest priority first
if let Some(Reverse(request)) = pending.pop() {
    // Process the highest priority request
}
```

## Key Insights

### Reverse Wrapper Pattern
- BinaryHeap is naturally a max-heap (largest values first)
- Reverse<T> inverts comparison logic for min-heap behavior when needed
- For priority queue, we want higher enum values (Critical=3) processed first
- BinaryHeap naturally provides this without Reverse since Critical > Normal

### Performance Characteristics
- Insert: O(log n) - efficient for background processing
- Pop highest priority: O(log n) - optimal for priority processing
- Peek: O(1) - can check priority without removing

### CleanupPriority Design
```rust
#[derive(PartialOrd, Ord)]
pub enum CleanupPriority {
    Low = 0,     // Least urgent
    Normal = 1,  // Default priority  
    High = 2,    // Important resources
    Critical = 3, // Must cleanup immediately
}
```

## Implementation Details

### Ordering Implementation
```rust
impl Ord for CleanupRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority values should come first in the heap
        self.priority.cmp(&other.priority)
    }
}
```

### Background Processing Loop
```rust
while let Some(Reverse(request)) = pending.pop() {
    // Process highest priority request first
    process_cleanup_request(request).await;
}
```

## Alternative Approaches Considered

### Vec with Sort
- Pro: Simple implementation
- Con: O(n log n) sort on each priority change
- **Verdict**: Too expensive for frequent operations

### Multiple Queues by Priority
- Pro: O(1) priority selection
- Con: Complex queue management and starvation issues
- **Verdict**: BinaryHeap simpler and sufficient

### Priority Queue Crate
- Pro: Specialized features
- Con: Additional dependency for standard use case
- **Verdict**: std::collections::BinaryHeap sufficient

## Usage Patterns

### Cleanup Request Creation
```rust
let request = CleanupRequest {
    resource_id,
    cleanup: Box::pin(cleanup_future),
    priority: CleanupPriority::High,
};
pending.push(Reverse(request));
```

### Batch Processing
```rust
// Process up to N requests per iteration
for _ in 0..max_batch_size {
    if let Some(Reverse(request)) = pending.pop() {
        process_cleanup_request(request).await;
    } else {
        break; // No more pending requests
    }
}
```

## Related Patterns

**See Also**:
- [[AsyncResourceGuard Drop Pattern]]
- [[Background Task Coordination]]
- [[Resource Cleanup Strategies]]

## Testing Considerations

- Test priority ordering with mixed priority requests
- Verify FIFO within same priority level
- Load test with high volumes of cleanup requests
- Validate memory usage with large numbers of pending requests

## Future Enhancements

- [ ] Priority aging (prevent starvation of low priority items)
- [ ] Dynamic priority adjustment based on resource age
- [ ] Batching optimization for same-priority requests
- [ ] Memory pressure-based priority boosting