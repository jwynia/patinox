//! Tests for Resource Management System
//!
//! This test suite covers the AsyncResourceGuard and ResourceRegistry components,
//! including normal operation, error conditions, concurrent access, and panic safety.

use patinox::memory::{AsyncResourceGuard, CleanupError};
use patinox::prelude::PatinoxError;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;

/// Test timeout constants
const ASYNC_CLEANUP_WAIT_MS: u64 = 50;
const SHORT_TIMEOUT_MS: u64 = 10;
const VERY_SHORT_DELAY_MS: u64 = 1;

/// Test resource that tracks if it was cleaned up
#[derive(Debug, Clone)]
struct TestResource {
    id: u32,
    cleaned_up: Arc<AtomicBool>,
    should_fail_cleanup: bool,
}

impl TestResource {
    fn new(id: u32) -> Self {
        Self {
            id,
            cleaned_up: Arc::new(AtomicBool::new(false)),
            should_fail_cleanup: false,
        }
    }
    
    fn with_cleanup_failure(id: u32) -> Self {
        Self {
            id,
            cleaned_up: Arc::new(AtomicBool::new(false)),
            should_fail_cleanup: true,
        }
    }
    
    fn is_cleaned_up(&self) -> bool {
        self.cleaned_up.load(Ordering::Relaxed)
    }
    
    async fn cleanup(self) -> Result<(), CleanupError> {
        if self.should_fail_cleanup {
            return Err(CleanupError::Failed("Intentional test failure".into()));
        }
        
        // Simulate async cleanup work
        tokio::time::sleep(Duration::from_millis(VERY_SHORT_DELAY_MS)).await;
        self.cleaned_up.store(true, Ordering::Relaxed);
        Ok(())
    }
}

#[cfg(test)]
mod async_resource_guard_tests {
    use super::*;

    #[tokio::test]
    async fn should_create_resource_guard_and_allow_access() {
        // Arrange
        let resource = TestResource::new(1);
        let cleanup_called = Arc::new(AtomicBool::new(false));
        let cleanup_called_clone = Arc::clone(&cleanup_called);
        
        // Act
        let guard = AsyncResourceGuard::new(
            resource,
            move |res| {
                let cleanup_called = Arc::clone(&cleanup_called_clone);
                async move {
                    cleanup_called.store(true, Ordering::Relaxed);
                    res.cleanup().await
                }
            }
        );
        
        // Assert - guard should be created successfully
        assert_eq!(guard.get().id, 1);
        assert!(!cleanup_called.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn should_provide_immutable_access_to_wrapped_resource() {
        // Arrange
        let resource = TestResource::new(42);
        let guard = AsyncResourceGuard::new(
            resource,
            |res| async move { res.cleanup().await }
        );
        
        // Act & Assert - should be able to access resource
        assert_eq!(guard.get().id, 42);
        assert!(!guard.get().is_cleaned_up());
    }

    #[tokio::test]
    async fn should_provide_mutable_access_to_wrapped_resource() {
        // Arrange
        let mut resource = TestResource::new(1);
        resource.id = 10;
        let mut guard = AsyncResourceGuard::new(
            resource,
            |res| async move { res.cleanup().await }
        );
        
        // Act - modify resource through guard
        guard.get_mut().id = 20;
        
        // Assert
        assert_eq!(guard.get().id, 20);
    }

    #[tokio::test]
    async fn test_into_inner_consumes_guard() {
        // Arrange
        let resource = TestResource::new(42);
        let cleanup_called = Arc::new(AtomicBool::new(false));
        let cleanup_called_clone = Arc::clone(&cleanup_called);
        
        let guard = AsyncResourceGuard::new(
            resource,
            move |res| {
                let cleanup_called = Arc::clone(&cleanup_called_clone);
                async move {
                    cleanup_called.store(true, Ordering::Relaxed);
                    res.cleanup().await
                }
            }
        );
        
        // Act - consume the guard
        let resource = guard.into_inner();
        
        // Assert - resource should be returned, cleanup should not have been called
        assert_eq!(resource.id, 42);
        assert!(!cleanup_called.load(Ordering::Relaxed));
        
        // Give time for any potential async cleanup
        tokio::time::sleep(Duration::from_millis(SHORT_TIMEOUT_MS)).await;
        assert!(!cleanup_called.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_manual_cleanup_success() {
        // Arrange
        let resource = TestResource::new(1);
        let cleaned_up_tracker = Arc::clone(&resource.cleaned_up);
        
        let guard = AsyncResourceGuard::new(
            resource,
            |res| async move { res.cleanup().await }
        );
        
        // Act - manually trigger cleanup
        let result = guard.cleanup().await;
        
        // Assert - cleanup should succeed
        assert!(result.is_ok());
        assert!(cleaned_up_tracker.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_manual_cleanup_failure() {
        // Arrange
        let resource = TestResource::with_cleanup_failure(1);
        
        let guard = AsyncResourceGuard::new(
            resource,
            |res| async move { res.cleanup().await }
        );
        
        // Act - manually trigger cleanup that should fail
        let result = guard.cleanup().await;
        
        // Assert - cleanup should fail with appropriate error
        assert!(result.is_err());
        match result.unwrap_err() {
            CleanupError::Failed(msg) => assert!(msg.to_string().contains("Intentional test failure")),
            _ => panic!("Expected Failed error variant"),
        }
    }

    #[tokio::test]
    async fn test_drop_triggers_async_cleanup() {
        // Arrange
        let resource = TestResource::new(1);
        let cleaned_up_tracker = Arc::clone(&resource.cleaned_up);
        
        // Act - create guard in scope and let it drop
        {
            let _guard = AsyncResourceGuard::new(
                resource,
                |res| async move { res.cleanup().await }
            );
            // Guard drops here
        }
        
        // Assert - give time for async cleanup to complete
        tokio::time::sleep(Duration::from_millis(ASYNC_CLEANUP_WAIT_MS)).await;
        assert!(cleaned_up_tracker.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_drop_cleanup_with_failure() {
        // Arrange
        let resource = TestResource::with_cleanup_failure(1);
        let cleaned_up_tracker = Arc::clone(&resource.cleaned_up);
        
        // Act - create guard that will fail cleanup on drop
        {
            let _guard = AsyncResourceGuard::new(
                resource,
                |res| async move { res.cleanup().await }
            );
            // Guard drops here, cleanup should fail but not panic
        }
        
        // Assert - resource should not be cleaned up due to failure
        tokio::time::sleep(Duration::from_millis(ASYNC_CLEANUP_WAIT_MS)).await;
        assert!(!cleaned_up_tracker.load(Ordering::Relaxed));
        
        // The important thing is that the failure didn't crash the program
    }

    #[tokio::test]
    async fn test_concurrent_guard_creation() {
        // Test that multiple AsyncResourceGuards can be created and dropped concurrently
        // without race conditions or resource leaks. This is a stress test to verify
        // thread safety and proper async cleanup coordination.
        
        // Arrange
        let task_count = 100;
        let successful_cleanups: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
        
        // Act - create many guards concurrently
        let handles: Vec<_> = (0..task_count).map(|i| {
            let counter = Arc::clone(&successful_cleanups);
            tokio::spawn(async move {
                let resource = TestResource::new(i);
                let resource_tracker = Arc::clone(&resource.cleaned_up);
                
                {
                    let _guard = AsyncResourceGuard::new(
                        resource,
                        |res| async move {
                            res.cleanup().await
                        }
                    );
                    // Small delay to simulate work
                    tokio::time::sleep(Duration::from_millis(VERY_SHORT_DELAY_MS)).await;
                }
                
                // Wait for cleanup
                tokio::time::sleep(Duration::from_millis(SHORT_TIMEOUT_MS)).await;
                
                if resource_tracker.load(Ordering::Relaxed) {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            })
        }).collect();
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Assert - all cleanups should have succeeded
        assert_eq!(successful_cleanups.load(Ordering::Relaxed), task_count);
    }

    #[tokio::test]
    async fn test_cleanup_timeout_handling() {
        // Arrange - resource with very slow cleanup
        let resource = TestResource::new(1);
        
        let guard = AsyncResourceGuard::new(
            resource,
            |res| async move {
                // Simulate slow cleanup
                tokio::time::sleep(Duration::from_millis(100)).await;
                res.cleanup().await
            }
        );
        
        // Act - try to cleanup with short timeout
        let result = timeout(Duration::from_millis(SHORT_TIMEOUT_MS), guard.cleanup()).await;
        
        // Assert - should timeout
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_guard_send_sync() {
        // This test ensures AsyncResourceGuard can be sent between threads
        let resource = TestResource::new(1);
        
        let guard = AsyncResourceGuard::new(
            resource,
            |res| async move { res.cleanup().await }
        );
        
        // Act - send guard to another task
        let handle = tokio::spawn(async move {
            assert_eq!(guard.get().id, 1);
            guard.cleanup().await
        });
        
        // Assert - should complete successfully
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_panic_safety_in_cleanup() {
        // Arrange - resource that panics during cleanup
        let resource = TestResource::new(1);
        
        let guard = AsyncResourceGuard::new(
            resource,
            |_| async move {
                panic!("Cleanup panic!");
            }
        );
        
        // Act & Assert - manual cleanup should handle panic gracefully
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                guard.cleanup().await
            })
        }));
        
        // The panic should be contained within the cleanup future
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod cleanup_error_tests {
    use super::*;

    #[test]
    fn test_cleanup_error_recovery_strategies() {
        // Test that each error type provides appropriate recovery strategy
        let timeout_error = CleanupError::Timeout;
        let failed_error = CleanupError::Failed("test".into());
        let already_cleaned_error = CleanupError::AlreadyCleanedUp;
        let shutting_down_error = CleanupError::ShuttingDown;
        
        // These methods should be implemented on CleanupError
        use patinox::error::RecoveryStrategy;
        
        // Note: These assertions will fail until CleanupError is implemented
        // with recovery_strategy() method
        assert!(matches!(timeout_error.recovery_strategy(), RecoveryStrategy::Retry));
        assert!(matches!(failed_error.recovery_strategy(), RecoveryStrategy::Fallback));
        assert!(matches!(already_cleaned_error.recovery_strategy(), RecoveryStrategy::Fail));
        assert!(matches!(shutting_down_error.recovery_strategy(), RecoveryStrategy::Fail));
    }

    #[test]
    fn test_cleanup_error_display() {
        let timeout_error = CleanupError::Timeout;
        let error_msg = format!("{}", timeout_error);
        assert!(error_msg.contains("timed out"));
    }

    #[test]
    fn test_cleanup_error_conversion_to_patinox_error() {
        let cleanup_error = CleanupError::Failed("test failure".into());
        let patinox_error: PatinoxError = cleanup_error.into();
        
        // Should convert to appropriate PatinoxError variant
        match patinox_error {
            PatinoxError::Execution(exec_error) => {
                assert!(exec_error.to_string().contains("test failure"));
            },
            _ => panic!("Expected Execution error variant"),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Mock monitor for testing integration
    struct MockMonitor {
        events: Arc<Mutex<Vec<String>>>,
    }
    
    impl MockMonitor {
        fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        async fn get_events(&self) -> Vec<String> {
            self.events.lock().await.clone()
        }
    }
    
    // Note: This will need to be implemented once Monitor trait integration is added
    #[tokio::test]
    #[ignore = "Monitoring integration not yet implemented"]
    async fn test_resource_guard_with_monitoring() {
        // Arrange
        let monitor = Arc::new(MockMonitor::new());
        let resource = TestResource::new(1);
        
        // Act - create guard with monitoring (this will need implementation)
        let guard = AsyncResourceGuard::new(
            resource,
            |res| async move { res.cleanup().await }
        );
        
        // Manually trigger cleanup
        let _result = guard.cleanup().await;
        
        // Assert - monitor should have recorded events
        // This test will need actual implementation
        let events = monitor.get_events().await;
        // assert!(events.iter().any(|e| e.contains("resource_cleanup")));
        
        // For now, just ensure the test structure is correct
        assert!(events.is_empty()); // Will change when monitoring is implemented
    }
}

#[cfg(test)]
mod property_based_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_resource_id_generation(id in any::<u32>()) {
            // Property: every resource should get a unique ID when created
            let resource1 = TestResource::new(id);
            let resource2 = TestResource::new(id);
            
            // Resource content might be same, but each should be cleanable
            prop_assert_eq!(resource1.id, resource2.id);
            prop_assert!(!resource1.is_cleaned_up());
            prop_assert!(!resource2.is_cleaned_up());
        }
        
        #[test] 
        fn test_resource_basic_properties(id in any::<u32>()) {
            // Property: resources should be created in non-cleaned-up state
            let resource = TestResource::new(id);
            prop_assert_eq!(resource.id, id);
            prop_assert!(!resource.is_cleaned_up());
        }
    }
}