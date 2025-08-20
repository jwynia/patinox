# Improve Error Context Preservation in CleanupError Conversion

## Problem
The current conversion from CleanupError to PatinoxError loses error context by converting to string, making debugging and error handling more difficult.

## Current Implementation
```rust
impl From<CleanupError> for PatinoxError {
    fn from(err: CleanupError) -> Self {
        PatinoxError::Execution(ExecutionError::ResourceExhausted(err.to_string()))
    }
}
```

## Issues
- Loses original error chain and context
- Maps to generic `ResourceExhausted` variant
- Makes debugging harder in production
- Doesn't preserve error source information

## Proposed Solutions

### Option 1: Add ResourceCleanup variant to ExecutionError
```rust
// In error.rs
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    // ... existing variants
    #[error("Resource cleanup failed: {0}")]
    ResourceCleanup(#[from] CleanupError),
}

// Updated conversion
impl From<CleanupError> for PatinoxError {
    fn from(err: CleanupError) -> Self {
        PatinoxError::Execution(ExecutionError::ResourceCleanup(err))
    }
}
```

### Option 2: Enhanced error mapping with context preservation
```rust
impl From<CleanupError> for PatinoxError {
    fn from(err: CleanupError) -> Self {
        let execution_error = match err {
            CleanupError::Timeout => ExecutionError::Timeout("Resource cleanup timed out".into()),
            CleanupError::AlreadyCleanedUp => ExecutionError::InvalidState("Resource already cleaned up".into()),
            CleanupError::Failed(source) => ExecutionError::ResourceExhausted(format!("Cleanup failed: {}", source)),
            CleanupError::ShuttingDown => ExecutionError::InvalidState("Registry shutting down".into()),
        };
        PatinoxError::Execution(execution_error)
    }
}
```

### Option 3: Create dedicated ResourceError category
```rust
// New error category
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Cleanup failed: {0}")]
    CleanupFailed(#[from] CleanupError),
    #[error("Resource allocation failed: {message}")]
    AllocationFailed { message: String },
    #[error("Resource exhausted: {resource_type}")]
    Exhausted { resource_type: String },
}

// Add to PatinoxError
#[derive(Debug, thiserror::Error)]
pub enum PatinoxError {
    // ... existing variants
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),
}
```

## Acceptance Criteria
- [ ] Choose appropriate error mapping strategy
- [ ] Preserve original error chain information
- [ ] Update all error conversion sites
- [ ] Add comprehensive error handling tests
- [ ] Update documentation with error handling examples
- [ ] Verify error display and debugging experience
- [ ] Ensure consistency with framework error patterns

## Investigation Required
- [ ] Audit existing ExecutionError variants for best fit
- [ ] Review error handling patterns in other modules
- [ ] Consider impact on monitoring and logging systems
- [ ] Evaluate backward compatibility implications

## Implementation Notes
- This may require updating ExecutionError enum (breaking change)
- Need to coordinate with monitoring system integration
- Consider how errors are displayed to users vs logged
- May affect error recovery strategies

## Priority: Medium
**Risk**: Error handling pattern change
**Impact**: Better debugging and error handling experience
**Effort**: 2-3 hours including analysis and testing