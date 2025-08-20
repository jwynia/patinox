//! Integration tests for ResourceRegistry
//!
//! Tests the ResourceRegistry component including resource tracking,
//! cleanup coordination, and shutdown behavior.

use patinox::memory::{ResourceRegistry, ResourceId, ResourceInfo};
use patinox::traits::Monitor;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Simple mock monitor for testing registry integration
#[derive(Debug)]
struct TestMonitor {
    name: String,
}

impl TestMonitor {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
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
    
    async fn record_event(&self, _event: patinox::traits::MonitorEvent) -> Result<(), patinox::error::PatinoxError> {
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
async fn test_registry_creation_and_health() {
    let monitor = Arc::new(TestMonitor::new("test-monitor")) as Arc<dyn Monitor>;
    let registry = ResourceRegistry::new(monitor);
    
    assert!(registry.is_healthy().await);
    assert_eq!(registry.active_count().await, 0);
}

#[tokio::test]
async fn test_resource_registration_lifecycle() {
    let monitor = Arc::new(TestMonitor::new("lifecycle-test")) as Arc<dyn Monitor>;
    let registry = ResourceRegistry::new(monitor);
    
    // Test initial state
    assert_eq!(registry.active_count().await, 0);
    
    // Register a resource
    let resource_id = ResourceId::generate();
    let resource_info = ResourceInfo {
        type_name: "TestResource",
        created_at: Instant::now(),
        size_bytes: Some(1024),
        metadata: {
            let mut map = HashMap::new();
            map.insert("test_key".to_string(), "test_value".to_string());
            map
        },
    };
    
    let result = registry.register(resource_id, resource_info.clone()).await;
    assert!(result.is_ok());
    assert_eq!(registry.active_count().await, 1);
    
    // Retrieve resource info
    let retrieved_info = registry.get_resource_info(&resource_id).await;
    assert!(retrieved_info.is_some());
    let retrieved_info = retrieved_info.unwrap();
    assert_eq!(retrieved_info.type_name, "TestResource");
    assert_eq!(retrieved_info.size_bytes, Some(1024));
    assert_eq!(retrieved_info.metadata.get("test_key"), Some(&"test_value".to_string()));
    
    // Unregister resource
    let removed = registry.unregister(&resource_id).await;
    assert!(removed.is_some());
    assert_eq!(registry.active_count().await, 0);
    
    // Verify resource is no longer retrievable
    let missing_info = registry.get_resource_info(&resource_id).await;
    assert!(missing_info.is_none());
}

#[tokio::test]
async fn test_multiple_resource_registration() {
    let monitor = Arc::new(TestMonitor::new("multiple-test")) as Arc<dyn Monitor>;
    let registry = ResourceRegistry::new(monitor);
    
    let resource_count = 10;
    let mut resource_ids = Vec::new();
    
    // Register multiple resources
    for i in 0..resource_count {
        let resource_id = ResourceId::generate();
        let resource_info = ResourceInfo {
            type_name: "TestResource",
            created_at: Instant::now(),
            size_bytes: Some(i * 100),
            metadata: HashMap::new(),
        };
        
        let result = registry.register(resource_id, resource_info).await;
        assert!(result.is_ok());
        resource_ids.push(resource_id);
    }
    
    assert_eq!(registry.active_count().await, resource_count);
    
    // Verify all resources are retrievable
    for resource_id in &resource_ids {
        let info = registry.get_resource_info(resource_id).await;
        assert!(info.is_some());
    }
    
    // Clean up all resources
    for resource_id in resource_ids {
        let removed = registry.unregister(&resource_id).await;
        assert!(removed.is_some());
    }
    
    assert_eq!(registry.active_count().await, 0);
}

#[tokio::test]
async fn test_registry_shutdown_behavior() {
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

#[tokio::test]
async fn test_force_cleanup_all() {
    let monitor = Arc::new(TestMonitor::new("cleanup-test")) as Arc<dyn Monitor>;
    let registry = ResourceRegistry::new(monitor);
    
    // Register resources
    let mut resource_ids = Vec::new();
    for i in 0..5 {
        let resource_id = ResourceId::generate();
        let resource_info = ResourceInfo {
            type_name: "TestResource",
            created_at: Instant::now(),
            size_bytes: Some(i * 50),
            metadata: HashMap::new(),
        };
        
        let _ = registry.register(resource_id, resource_info).await;
        resource_ids.push(resource_id);
    }
    
    assert_eq!(registry.active_count().await, 5);
    
    // Force cleanup all
    let result = registry.force_cleanup_all().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5); // Should return count of resources scheduled for cleanup
    
    // Give cleanup time to process
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_resource_id_uniqueness() {
    let mut ids = std::collections::HashSet::new();
    
    for _ in 0..1000 {
        let id = ResourceId::generate();
        assert!(ids.insert(id), "Duplicate ResourceId generated: {}", id);
    }
}

#[tokio::test]
async fn test_registry_after_shutdown_rejects_operations() {
    let monitor = Arc::new(TestMonitor::new("post-shutdown-test")) as Arc<dyn Monitor>;
    let registry = ResourceRegistry::new(monitor);
    
    // Shutdown the registry
    registry.shutdown().await;
    
    // Attempts to register should fail
    let resource_id = ResourceId::generate();
    let resource_info = ResourceInfo {
        type_name: "TestResource",
        created_at: Instant::now(),
        size_bytes: Some(100),
        metadata: HashMap::new(),
    };
    
    let result = registry.register(resource_id, resource_info).await;
    assert!(result.is_err());
    
    // Force cleanup should also fail
    let cleanup_result = registry.force_cleanup_all().await;
    assert!(cleanup_result.is_err());
}