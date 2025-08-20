//! Resource Management System
//!
//! This module provides utilities for safe resource cleanup with async support.
//! The main components are:
//!
//! - [`AsyncResourceGuard`]: RAII guard that handles async cleanup on drop
//! - [`CleanupError`]: Error type for cleanup operations
//! - [`CleanupFn`]: Type alias for cleanup functions
//! - [`CleanupPriority`]: Priority levels for cleanup operations

use crate::error::{PatinoxError, ExecutionError, RecoveryStrategy};
use std::future::Future;
use std::pin::Pin;
// Arc and Weak will be used later for registry integration
use uuid::Uuid;

/// Unique identifier for resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ResourceId(Uuid);

impl ResourceId {
    /// Generate a new unique resource ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Priority levels for cleanup operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub enum CleanupPriority {
    /// Low priority cleanup (background maintenance)
    Low = 0,
    /// Normal priority cleanup (default)
    #[default]
    Normal = 1,
    /// High priority cleanup (important resources)
    High = 2,
    /// Critical priority cleanup (must be cleaned up immediately)
    Critical = 3,
}

/// Error types for resource cleanup operations
#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    /// Cleanup operation timed out
    #[error("Cleanup operation timed out")]
    Timeout,
    
    /// Resource was already cleaned up
    #[error("Resource was already cleaned up")]
    AlreadyCleanedUp,
    
    /// Cleanup operation failed with an error
    #[error("Cleanup failed: {0}")]
    Failed(#[from] Box<dyn std::error::Error + Send + Sync>),
    
    /// Registry is shutting down and cannot process cleanup
    #[error("Registry is shutting down")]
    ShuttingDown,
}

impl CleanupError {
    /// Get the recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            CleanupError::Timeout => RecoveryStrategy::Retry,
            CleanupError::AlreadyCleanedUp => RecoveryStrategy::Fail,
            CleanupError::Failed(_) => RecoveryStrategy::Fallback,
            CleanupError::ShuttingDown => RecoveryStrategy::Fail,
        }
    }
}

impl From<CleanupError> for PatinoxError {
    fn from(err: CleanupError) -> Self {
        PatinoxError::Execution(ExecutionError::ResourceExhausted(err.to_string()))
    }
}

/// Type alias for cleanup functions
pub type CleanupFn<T> = Box<dyn FnOnce(T) -> Pin<Box<dyn Future<Output = Result<(), CleanupError>> + Send>> + Send>;

/// RAII guard for async resource cleanup
///
/// This guard ensures that resources are properly cleaned up when they are no longer needed,
/// even across async boundaries. The cleanup function is called either when the guard is
/// manually dropped via [`cleanup()`] or when the guard is dropped normally.
///
/// # Examples
///
/// ```rust,no_run
/// use patinox::memory::AsyncResourceGuard;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a guard with a cleanup function
/// let guard = AsyncResourceGuard::new(
///     "my resource",
///     |resource| async move {
///         println!("Cleaning up: {}", resource);
///         Ok(())
///     }
/// );
///
/// // Use the resource
/// println!("Using: {}", guard.get());
///
/// // Resource will be cleaned up when guard drops
/// # Ok(())
/// # }
/// ```
pub struct AsyncResourceGuard<T: Send + 'static> {
    resource: Option<T>,
    cleanup: Option<CleanupFn<T>>,
    resource_id: ResourceId,
    // We'll add registry integration later
}

impl<T: Send + 'static> AsyncResourceGuard<T> {
    /// Create a new resource guard with a cleanup function
    ///
    /// The cleanup function will be called when the guard is dropped or when
    /// [`cleanup()`] is called manually.
    pub fn new<F, Fut>(resource: T, cleanup_fn: F) -> Self 
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), CleanupError>> + Send + 'static,
    {
        let cleanup: CleanupFn<T> = Box::new(move |res| {
            Box::pin(cleanup_fn(res))
        });
        
        Self {
            resource: Some(resource),
            cleanup: Some(cleanup),
            resource_id: ResourceId::generate(),
        }
    }
    
    /// Get an immutable reference to the resource
    pub fn get(&self) -> &T {
        self.resource.as_ref().expect("Resource was already consumed")
    }
    
    /// Get a mutable reference to the resource
    pub fn get_mut(&mut self) -> &mut T {
        self.resource.as_mut().expect("Resource was already consumed")
    }
    
    /// Consume the guard and return the resource without cleanup
    pub fn into_inner(mut self) -> T {
        self.resource.take().expect("Resource was already consumed")
    }
    
    /// Manually trigger cleanup and consume the guard
    ///
    /// This allows for error handling of the cleanup operation. If not called,
    /// cleanup will happen automatically on drop (but errors will be logged rather
    /// than returned).
    pub async fn cleanup(mut self) -> Result<(), CleanupError> {
        if let (Some(resource), Some(cleanup)) = (self.resource.take(), self.cleanup.take()) {
            cleanup(resource).await
        } else {
            Err(CleanupError::AlreadyCleanedUp)
        }
    }
    
    /// Get the resource ID
    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }
}

impl<T: Send + 'static> Drop for AsyncResourceGuard<T> {
    fn drop(&mut self) {
        if let (Some(resource), Some(cleanup)) = (self.resource.take(), self.cleanup.take()) {
            // For now, we'll spawn the cleanup directly
            // TODO: Later we'll integrate with the ResourceRegistry for coordinated cleanup
            // Blocked by: Need to add registry field and update constructor API
            tokio::spawn(async move {
                if let Err(e) = cleanup(resource).await {
                    // Log the error but don't panic
                    log::error!("Resource cleanup failed: {}", e);
                }
            });
        }
    }
}

// Safety: AsyncResourceGuard can be sent between threads if T can be sent
unsafe impl<T: Send + 'static> Send for AsyncResourceGuard<T> {}
// Safety: AsyncResourceGuard can be shared between threads if T can be shared
unsafe impl<T: Send + Sync + 'static> Sync for AsyncResourceGuard<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_basic_functionality() {
        let cleanup_called = Arc::new(AtomicBool::new(false));
        let cleanup_called_clone = Arc::clone(&cleanup_called);
        
        let guard = AsyncResourceGuard::new(
            "test resource",
            move |_resource| {
                let cleanup_called = Arc::clone(&cleanup_called_clone);
                async move {
                    cleanup_called.store(true, Ordering::Relaxed);
                    Ok(())
                }
            }
        );
        
        assert_eq!(*guard.get(), "test resource");
        
        let result = guard.cleanup().await;
        assert!(result.is_ok());
        assert!(cleanup_called.load(Ordering::Relaxed));
    }
    
    #[test]
    fn test_resource_id_generation() {
        let id1 = ResourceId::generate();
        let id2 = ResourceId::generate();
        
        assert_ne!(id1, id2);
        
        // Test display
        let display_str = format!("{}", id1);
        assert!(!display_str.is_empty());
        assert!(display_str.contains('-'));
    }
    
    #[test]
    fn test_cleanup_priority_ordering() {
        let low = CleanupPriority::Low;
        let normal = CleanupPriority::Normal;
        let high = CleanupPriority::High;
        let critical = CleanupPriority::Critical;
        
        assert!(low < normal);
        assert!(normal < high);
        assert!(high < critical);
        
        assert_eq!(CleanupPriority::default(), CleanupPriority::Normal);
    }
    
    #[test]
    fn test_cleanup_error_recovery_strategies() {
        let timeout_error = CleanupError::Timeout;
        let failed_error = CleanupError::Failed("test".into());
        let already_cleaned_error = CleanupError::AlreadyCleanedUp;
        let shutting_down_error = CleanupError::ShuttingDown;
        
        assert!(matches!(timeout_error.recovery_strategy(), RecoveryStrategy::Retry));
        assert!(matches!(failed_error.recovery_strategy(), RecoveryStrategy::Fallback));
        assert!(matches!(already_cleaned_error.recovery_strategy(), RecoveryStrategy::Fail));
        assert!(matches!(shutting_down_error.recovery_strategy(), RecoveryStrategy::Fail));
    }
    
    #[test]
    fn test_cleanup_error_conversion() {
        let cleanup_error = CleanupError::Failed("test failure".into());
        let patinox_error: PatinoxError = cleanup_error.into();
        
        match patinox_error {
            PatinoxError::Execution(exec_error) => {
                assert!(exec_error.to_string().contains("test failure"));
            },
            _ => panic!("Expected Execution error variant"),
        }
    }
}