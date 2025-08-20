//! Integration tests for Memory Management Utilities
//!
//! These tests verify that the AsyncResourceGuard and ResourceRegistry components
//! work together correctly in realistic usage scenarios.

use patinox::memory::{AsyncResourceGuard, ResourceId, ResourceInfo, ResourceRegistry};
use patinox::traits::Monitor;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Test resource that tracks cleanup
#[derive(Debug, Clone)]
struct TestResource {
    id: u32,
    name: String,
    cleaned_up: Arc<AtomicBool>,
}

impl TestResource {
    fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            cleaned_up: Arc::new(AtomicBool::new(false)),
        }
    }

    fn is_cleaned_up(&self) -> bool {
        self.cleaned_up.load(Ordering::Relaxed)
    }

    async fn cleanup(self) -> Result<(), patinox::memory::CleanupError> {
        // Simulate cleanup work
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        self.cleaned_up.store(true, Ordering::Relaxed);
        Ok(())
    }
}

/// Simple mock monitor for testing
#[derive(Debug)]
struct TestMonitor {
    name: String,
}

impl TestMonitor {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait::async_trait]
impl Monitor for TestMonitor {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start_monitoring(
        &self,
        _execution_id: uuid::Uuid,
        _agent_id: uuid::Uuid,
    ) -> Result<(), patinox::error::PatinoxError> {
        Ok(())
    }

    async fn record_event(
        &self,
        _event: patinox::traits::MonitorEvent,
    ) -> Result<(), patinox::error::PatinoxError> {
        Ok(())
    }

    async fn complete_monitoring(
        &self,
        _execution_id: uuid::Uuid,
        _summary: patinox::traits::ExecutionSummary,
    ) -> Result<(), patinox::error::PatinoxError> {
        Ok(())
    }

    async fn query_events(
        &self,
        _query: patinox::traits::MonitorQuery,
    ) -> Result<Vec<patinox::traits::MonitorEvent>, patinox::error::PatinoxError> {
        Ok(Vec::new())
    }

    fn config(&self) -> &patinox::traits::MonitorConfig {
        use patinox::traits::{MonitorConfig, MonitorEventType};
        static CONFIG: std::sync::OnceLock<MonitorConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| MonitorConfig {
            name: "TestMonitor".to_string(),
            enabled: true,
            buffer_size: 1000,
            flush_interval_ms: 5000,
            sampling_rate: 1.0,
            event_types: vec![MonitorEventType::ExecutionStarted],
        })
    }
}

#[tokio::test]
async fn test_resource_guard_basic_functionality() {
    // Test that AsyncResourceGuard works correctly for simple cases
    let resource = TestResource::new(1, "test-resource");
    let cleanup_tracker = Arc::clone(&resource.cleaned_up);

    let guard = AsyncResourceGuard::new(resource, |res| async move { res.cleanup().await });

    // Verify resource access
    assert_eq!(guard.get().id, 1);
    assert_eq!(guard.get().name, "test-resource");
    assert!(!cleanup_tracker.load(Ordering::Relaxed));

    // Manually cleanup
    let result = guard.cleanup().await;
    assert!(result.is_ok());
    assert!(cleanup_tracker.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_resource_guard_drop_cleanup() {
    // Test that resources are cleaned up when guard is dropped
    let cleanup_tracker = {
        let resource = TestResource::new(2, "drop-test");
        let tracker = Arc::clone(&resource.cleaned_up);

        {
            let _guard =
                AsyncResourceGuard::new(resource, |res| async move { res.cleanup().await });
            // Guard drops here
        }

        tracker
    };

    // Wait for async cleanup to complete
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    assert!(cleanup_tracker.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_resource_registry_lifecycle() {
    // Test ResourceRegistry basic operations
    let monitor = Arc::new(TestMonitor::new("registry-test")) as Arc<dyn Monitor>;
    let registry = ResourceRegistry::new(monitor);

    // Verify initial state
    assert_eq!(registry.active_count().await, 0);
    assert!(registry.is_healthy().await);

    // Register a resource
    let resource_id = ResourceId::generate();
    let resource_info = ResourceInfo {
        type_name: "TestResource",
        created_at: Instant::now(),
        size_bytes: Some(100),
        metadata: {
            let mut map = HashMap::new();
            map.insert("test_key".to_string(), "test_value".to_string());
            map
        },
    };

    let register_result = registry.register(resource_id, resource_info.clone()).await;
    assert!(register_result.is_ok());
    assert_eq!(registry.active_count().await, 1);

    // Retrieve resource info
    let retrieved = registry.get_resource_info(&resource_id).await;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.type_name, "TestResource");
    assert_eq!(retrieved.size_bytes, Some(100));

    // Unregister resource
    let removed = registry.unregister(&resource_id).await;
    assert!(removed.is_some());
    assert_eq!(registry.active_count().await, 0);
}

#[tokio::test]
async fn test_multiple_resources_cleanup() {
    // Test concurrent resource management
    let monitor = Arc::new(TestMonitor::new("concurrent-test")) as Arc<dyn Monitor>;
    let registry = Arc::new(ResourceRegistry::new(monitor));
    let successful_cleanups = Arc::new(AtomicU32::new(0));

    let resource_count = 10;
    let mut handles = Vec::new();

    for i in 0..resource_count {
        let registry_clone = Arc::clone(&registry);
        let counter = Arc::clone(&successful_cleanups);

        let handle = tokio::spawn(async move {
            // Create resource
            let resource = TestResource::new(i, format!("resource-{}", i));
            let cleanup_tracker = Arc::clone(&resource.cleaned_up);

            // Register with registry
            let resource_id = ResourceId::generate();
            let resource_info = ResourceInfo {
                type_name: "TestResource",
                created_at: Instant::now(),
                size_bytes: Some(i as usize * 10),
                metadata: HashMap::new(),
            };

            let _ = registry_clone.register(resource_id, resource_info).await;

            // Create guard and let it drop
            {
                let _guard =
                    AsyncResourceGuard::new(resource, |res| async move { res.cleanup().await });

                // Simulate some work
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }

            // Wait for cleanup
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;

            if cleanup_tracker.load(Ordering::Relaxed) {
                counter.fetch_add(1, Ordering::Relaxed);
            }

            // Clean up registry entry
            let _ = registry_clone.unregister(&resource_id).await;
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all resources were cleaned up
    assert_eq!(successful_cleanups.load(Ordering::Relaxed), resource_count);
    assert_eq!(registry.active_count().await, 0);
}

#[tokio::test]
async fn test_resource_guard_error_handling() {
    // Test that guard handles cleanup errors gracefully
    let resource = TestResource::new(3, "error-test");

    let guard = AsyncResourceGuard::new(resource, |_| async move {
        Err(patinox::memory::CleanupError::Failed(
            "Intentional test error".into(),
        ))
    });

    // Manual cleanup should return error
    let result = guard.cleanup().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resource_id_uniqueness() {
    // Test that ResourceId generates unique values
    let mut ids = std::collections::HashSet::new();

    for _ in 0..1000 {
        let id = ResourceId::generate();
        assert!(ids.insert(id), "Duplicate ResourceId generated: {}", id);
    }
}

#[tokio::test]
async fn test_registry_shutdown() {
    // Test registry shutdown behavior
    let monitor = Arc::new(TestMonitor::new("shutdown-test")) as Arc<dyn Monitor>;
    let registry = ResourceRegistry::new(monitor);

    // Register some resources
    for i in 0..3 {
        let resource_id = ResourceId::generate();
        let resource_info = ResourceInfo {
            type_name: "TestResource",
            created_at: Instant::now(),
            size_bytes: Some(i * 100),
            metadata: HashMap::new(),
        };

        let _ = registry.register(resource_id, resource_info).await;
    }

    assert_eq!(registry.active_count().await, 3);
    assert!(registry.is_healthy().await);

    // Shutdown registry
    registry.shutdown().await;

    assert!(!registry.is_healthy().await);
    assert_eq!(registry.active_count().await, 0);
}
