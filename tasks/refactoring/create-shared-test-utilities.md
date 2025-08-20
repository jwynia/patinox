# Create Shared Test Utilities Module

## Problem
Multiple test files implement similar mock structures and test utilities, leading to code duplication and maintenance overhead.

## Current Duplication

### TestMonitor/MockMonitor implementations found in:
- `tests/resource_management_test.rs` - MockMonitor struct (lines 384-398)
- `tests/registry_integration_test.rs` - TestMonitor struct (lines 14-71) 
- `tests/memory_integration_test.rs` - TestMonitor struct (lines 44-101)
- `src/memory/registry.rs` - MockMonitor in tests module (lines 347-412)

### Duplicated patterns:
- Mock Monitor trait implementations with same boilerplate
- TestResource structures with similar cleanup tracking
- Common test timeout constants
- Shared assertion patterns

## Proposed Solution

### Create `tests/common/mod.rs` module:

```rust
//! Shared test utilities for memory management tests

use patinox::traits::{Monitor, MonitorConfig, MonitorEvent, MonitorEventType, MonitorQuery, ExecutionSummary};
use patinox::error::PatinoxError;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test timeout constants used across test suites
pub mod timeouts {
    pub const ASYNC_CLEANUP_WAIT_MS: u64 = 50;
    pub const SHORT_TIMEOUT_MS: u64 = 10;
    pub const VERY_SHORT_DELAY_MS: u64 = 1;
    pub const STRESS_TEST_COUNT: u32 = 100;
}

/// Configurable mock monitor for testing
#[derive(Debug)]
pub struct TestMonitor {
    name: String,
    events: Arc<Mutex<Vec<MonitorEvent>>>,
    should_fail: bool,
}

impl TestMonitor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            events: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
        }
    }
    
    pub fn with_failure(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            events: Arc::new(Mutex::new(Vec::new())),
            should_fail: true,
        }
    }
    
    pub async fn get_events(&self) -> Vec<MonitorEvent> {
        self.events.lock().await.clone()
    }
    
    pub async fn event_count(&self) -> usize {
        self.events.lock().await.len()
    }
}

#[async_trait::async_trait]
impl Monitor for TestMonitor {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn start_monitoring(&self, _: uuid::Uuid, _: uuid::Uuid) -> Result<(), PatinoxError> {
        if self.should_fail {
            Err(PatinoxError::Configuration("Test monitor configured to fail".into()))
        } else {
            Ok(())
        }
    }
    
    async fn record_event(&self, event: MonitorEvent) -> Result<(), PatinoxError> {
        if self.should_fail {
            Err(PatinoxError::Configuration("Test monitor configured to fail".into()))
        } else {
            self.events.lock().await.push(event);
            Ok(())
        }
    }
    
    // ... implement other required methods
}

/// Test resource with controllable cleanup behavior
#[derive(Debug, Clone)]
pub struct TestResource {
    pub id: u32,
    pub name: String,
    pub cleaned_up: Arc<std::sync::atomic::AtomicBool>,
    pub should_fail_cleanup: bool,
}

impl TestResource {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            name: format!("test-resource-{}", id),
            cleaned_up: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            should_fail_cleanup: false,
        }
    }
    
    pub fn with_name(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            cleaned_up: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            should_fail_cleanup: false,
        }
    }
    
    pub fn with_cleanup_failure(id: u32) -> Self {
        Self {
            id,
            name: format!("failing-resource-{}", id),
            cleaned_up: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            should_fail_cleanup: true,
        }
    }
    
    pub fn is_cleaned_up(&self) -> bool {
        self.cleaned_up.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    pub async fn cleanup(self) -> Result<(), patinox::memory::CleanupError> {
        use patinox::memory::CleanupError;
        use crate::common::timeouts::VERY_SHORT_DELAY_MS;
        
        if self.should_fail_cleanup {
            return Err(CleanupError::Failed("Intentional test failure".into()));
        }
        
        // Simulate async cleanup work
        tokio::time::sleep(std::time::Duration::from_millis(VERY_SHORT_DELAY_MS)).await;
        self.cleaned_up.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

/// Helper macros for common test patterns
#[macro_export]
macro_rules! assert_cleanup_success {
    ($guard:expr, $tracker:expr) => {
        let result = $guard.cleanup().await;
        assert!(result.is_ok(), "Cleanup should succeed");
        assert!($tracker.is_cleaned_up(), "Resource should be marked as cleaned up");
    };
}

#[macro_export]
macro_rules! wait_for_async_cleanup {
    ($tracker:expr) => {
        tokio::time::sleep(std::time::Duration::from_millis(
            $crate::common::timeouts::ASYNC_CLEANUP_WAIT_MS
        )).await;
        assert!($tracker.is_cleaned_up(), "Async cleanup should have completed");
    };
}
```

### Update existing test files to use shared utilities:

```rust
// In tests/resource_management_test.rs
use crate::common::{TestMonitor, TestResource, timeouts::*};

// Remove duplicated implementations and use shared ones
```

## Benefits
- Eliminates code duplication across test files
- Provides consistent test utilities and behavior
- Easier maintenance when changing test infrastructure
- Better test organization and reusability
- Consistent timeout values across all tests

## Acceptance Criteria
- [ ] Create `tests/common/mod.rs` with shared utilities
- [ ] Migrate all test files to use shared TestMonitor
- [ ] Migrate all test files to use shared TestResource
- [ ] Consolidate timeout constants in shared module
- [ ] Add helpful test macros for common patterns
- [ ] Ensure all tests continue to pass after migration
- [ ] Remove all duplicate implementations
- [ ] Update test documentation to reference shared utilities

## Migration Strategy
1. Create shared utilities module with comprehensive implementations
2. Update one test file at a time to use shared utilities
3. Run tests after each migration to ensure compatibility
4. Remove duplicate code after successful migration
5. Add documentation and examples for shared utilities

## Dependencies
- All existing tests must continue to pass
- Consider impact on test compilation times
- Ensure shared utilities don't introduce coupling between test files

## Implementation Notes
- Use `#[cfg(test)]` appropriately for test-only code
- Consider whether utilities should be in `tests/common/` or `src/test_utils/`
- May need to adjust visibility and module organization
- Ensure mock implementations are flexible enough for all use cases

## Priority: Medium
**Risk**: Medium (affects multiple test files)
**Impact**: Improved test maintainability and consistency
**Effort**: 2-4 hours including migration and validation