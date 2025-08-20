//! Resource Registry System
//!
//! This module provides a centralized registry for tracking and managing resources.
//! The main component is [`ResourceRegistry`] which coordinates resource cleanup
//! and provides observability into resource usage.

use super::resource::{ResourceId, CleanupError, CleanupPriority};
use crate::traits::Monitor;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

/// Timeout configuration constants
const SHUTDOWN_GRACE_PERIOD_MS: u64 = 100;
const CLEANUP_POLL_INTERVAL_MS: u64 = 10;

/// Information about a registered resource
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    /// Type name of the resource
    pub type_name: &'static str,
    /// When the resource was created
    pub created_at: Instant,
    /// Size in bytes (if known)
    pub size_bytes: Option<usize>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// A cleanup request for the registry
pub struct CleanupRequest {
    /// ID of the resource to clean up
    pub resource_id: ResourceId,
    /// The cleanup future to execute
    pub cleanup: Pin<Box<dyn Future<Output = Result<(), CleanupError>> + Send>>,
    /// Priority of this cleanup operation
    pub priority: CleanupPriority,
}

impl std::fmt::Debug for CleanupRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CleanupRequest")
            .field("resource_id", &self.resource_id)
            .field("priority", &self.priority)
            .finish()
    }
}

impl PartialEq for CleanupRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.resource_id == other.resource_id
    }
}

impl Eq for CleanupRequest {}

impl PartialOrd for CleanupRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CleanupRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority should come first in the heap
        self.priority.cmp(&other.priority)
    }
}

/// Central registry for resource tracking and cleanup coordination
pub struct ResourceRegistry {
    /// Map of active resources
    active: Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
    /// Channel for cleanup requests
    cleanup_tx: mpsc::UnboundedSender<CleanupRequest>,
    /// Monitor for observability (TODO: integrate with monitoring system)
    #[allow(dead_code)] // Will be used once monitoring integration is implemented
    monitor: Arc<dyn Monitor>,
    /// Shutdown flag
    shutdown: Arc<AtomicBool>,
    /// Cleanup task handle
    _cleanup_task: JoinHandle<()>,
    /// Counter for successful cleanups (for testing)
    cleanup_count: Arc<AtomicU32>,
}

impl ResourceRegistry {
    /// Create a new resource registry
    pub fn new(monitor: Arc<dyn Monitor>) -> Self {
        let (cleanup_tx, cleanup_rx) = mpsc::unbounded_channel();
        let active = Arc::new(RwLock::new(HashMap::new()));
        let shutdown = Arc::new(AtomicBool::new(false));
        let cleanup_count = Arc::new(AtomicU32::new(0));
        
        // Start cleanup task
        let cleanup_task = Self::start_cleanup_task(
            cleanup_rx,
            Arc::clone(&active),
            Arc::clone(&monitor),
            Arc::clone(&shutdown),
            Arc::clone(&cleanup_count),
        );
        
        Self {
            active,
            cleanup_tx,
            monitor,
            shutdown,
            _cleanup_task: cleanup_task,
            cleanup_count,
        }
    }
    
    /// Register a resource with the registry
    pub async fn register(
        &self,
        resource_id: ResourceId,
        info: ResourceInfo,
    ) -> Result<(), CleanupError> {
        if self.shutdown.load(Ordering::Relaxed) {
            return Err(CleanupError::ShuttingDown);
        }
        
        {
            let mut active = self.active.write().await;
            active.insert(resource_id, info.clone());
        }
        
        // TODO: Add monitoring integration once MonitorEventType supports resource events
        // Blocked by: Need to extend MonitorEventType enum in traits module
        // For now, we'll just log resource creation
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("Resource registered: {} (type: {})", resource_id, info.type_name);
        }
        
        Ok(())
    }
    
    /// Unregister a resource from the registry
    pub async fn unregister(&self, resource_id: &ResourceId) -> Option<ResourceInfo> {
        let removed = {
            let mut active = self.active.write().await;
            active.remove(resource_id)
        };
        
        if removed.is_some() {
            // TODO: Add monitoring integration once MonitorEventType supports resource events
            // Blocked by: Need to extend MonitorEventType enum in traits module
            if log::log_enabled!(log::Level::Debug) {
                log::debug!("Resource unregistered: {}", resource_id);
            }
        }
        
        removed
    }
    
    /// Get information about a specific resource
    pub async fn get_resource_info(&self, resource_id: &ResourceId) -> Option<ResourceInfo> {
        let active = self.active.read().await;
        active.get(resource_id).cloned()
    }
    
    /// Get the count of active resources
    pub async fn active_count(&self) -> usize {
        let active = self.active.read().await;
        active.len()
    }
    
    /// Schedule a cleanup operation
    pub async fn schedule_cleanup(&self, request: CleanupRequest) {
        if self.cleanup_tx.send(request).is_err() {
            // Cleanup task is no longer running, likely due to shutdown
            log::warn!("Failed to schedule cleanup: registry is shutting down");
        }
    }
    
    /// Force cleanup of all registered resources
    pub async fn force_cleanup_all(&self) -> Result<usize, CleanupError> {
        if self.shutdown.load(Ordering::Relaxed) {
            return Err(CleanupError::ShuttingDown);
        }
        
        let active_resources = {
            let active = self.active.read().await;
            active.keys().cloned().collect::<Vec<_>>()
        };
        
        let count = active_resources.len();
        
        // Schedule high-priority cleanup for all resources
        for resource_id in active_resources {
            let cleanup_future = Box::pin(async move {
                // Generic cleanup - just remove from registry
                Ok(())
            });
            
            let request = CleanupRequest {
                resource_id,
                cleanup: cleanup_future,
                priority: CleanupPriority::High,
            };
            
            self.schedule_cleanup(request).await;
        }
        
        Ok(count)
    }
    
    /// Check if the registry is healthy (not shutting down)
    pub async fn is_healthy(&self) -> bool {
        !self.shutdown.load(Ordering::Relaxed)
    }
    
    /// Shutdown the registry and cleanup all resources
    pub async fn shutdown(&self) {
        // Set shutdown flag
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Force cleanup all resources
        let _ = self.force_cleanup_all().await;
        
        // Give cleanup task time to process remaining requests
        tokio::time::sleep(std::time::Duration::from_millis(SHUTDOWN_GRACE_PERIOD_MS)).await;
        
        // Clear any remaining resources from the registry
        {
            let mut active = self.active.write().await;
            active.clear();
        }
    }
    
    /// Get cleanup count (for testing)
    pub fn cleanup_count(&self) -> u32 {
        self.cleanup_count.load(Ordering::Relaxed)
    }
    
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
                // Check if we should shutdown
                if shutdown.load(Ordering::Relaxed) {
                    // Process remaining requests with higher priority first
                    while let Some(Reverse(request)) = pending.pop() {
                        Self::process_cleanup_request(
                            request,
                            &active,
                            &monitor,
                            &cleanup_count,
                        ).await;
                    }
                    break;
                }
                
                // Try to receive new requests without blocking too long
                match tokio::time::timeout(
                    std::time::Duration::from_millis(CLEANUP_POLL_INTERVAL_MS),
                    cleanup_rx.recv()
                ).await {
                    Ok(Some(request)) => {
                        // Add to priority queue (using Reverse for max-heap behavior)
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
                
                // Process one pending request if available
                if let Some(Reverse(request)) = pending.pop() {
                    Self::process_cleanup_request(
                        request,
                        &active,
                        &monitor,
                        &cleanup_count,
                    ).await;
                }
            }
        })
    }
    
    async fn process_cleanup_request(
        request: CleanupRequest,
        active: &Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
        _monitor: &Arc<dyn Monitor>,
        cleanup_count: &Arc<AtomicU32>,
    ) {
        let start = Instant::now();
        let resource_id = request.resource_id;
        
        // Execute the cleanup
        let result = request.cleanup.await;
        let success = result.is_ok();
        
        if success {
            cleanup_count.fetch_add(1, Ordering::Relaxed);
            
            // Remove from active registry
            let mut active_lock = active.write().await;
            active_lock.remove(&resource_id);
        }
        
        // TODO: Add monitoring integration once MonitorEventType supports resource events
        // Blocked by: Need to extend MonitorEventType enum in traits module
        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "Resource cleanup completed: {} (success: {}, duration: {}ms)",
                resource_id,
                success,
                start.elapsed().as_millis()
            );
        }
        
        if let Err(e) = result {
            log::error!("Resource cleanup failed for {}: {}", resource_id, e);
        }
    }
}

impl std::fmt::Debug for ResourceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceRegistry")
            .field("shutdown", &self.shutdown.load(Ordering::Relaxed))
            .finish()
    }
}

// TODO: Extend MonitorEventType to support resource events
// Estimate: Small task, requires updating traits::MonitorEventType enum
// Dependencies: None - can be done independently
// This will be added in a future iteration when we extend the monitoring system

#[cfg(test)]
mod tests {
    use super::*;
    // Removed unused Mutex import
    
    /// Mock monitor for testing
    #[derive(Debug)]
    struct MockMonitor;
    
    impl MockMonitor {
        fn new() -> Self {
            Self
        }
    }
    
    #[async_trait::async_trait] 
    impl Monitor for MockMonitor {
        fn name(&self) -> &str {
            "MockMonitor"
        }
        
        async fn start_monitoring(
            &self,
            _execution_id: uuid::Uuid,
            _agent_id: uuid::Uuid,
        ) -> Result<(), crate::error::PatinoxError> {
            Ok(())
        }
        
        async fn record_event(&self, _event: crate::traits::MonitorEvent) -> Result<(), crate::error::PatinoxError> {
            // For simplified testing, we'll just ignore the events for now
            Ok(())
        }
        
        async fn complete_monitoring(
            &self,
            _execution_id: uuid::Uuid,
            _summary: crate::traits::ExecutionSummary,
        ) -> Result<(), crate::error::PatinoxError> {
            Ok(())
        }
        
        async fn query_events(
            &self,
            _query: crate::traits::MonitorQuery,
        ) -> Result<Vec<crate::traits::MonitorEvent>, crate::error::PatinoxError> {
            Ok(Vec::new())
        }
        
        fn config(&self) -> &crate::traits::MonitorConfig {
            use crate::traits::{MonitorConfig, MonitorEventType};
            // Return a default config
            static CONFIG: std::sync::OnceLock<MonitorConfig> = std::sync::OnceLock::new();
            CONFIG.get_or_init(|| MonitorConfig {
                name: "MockMonitor".to_string(),
                enabled: true,
                buffer_size: 1000,
                flush_interval_ms: 5000,
                sampling_rate: 1.0,
                event_types: vec![MonitorEventType::ExecutionStarted],
            })
        }
    }
    
    #[tokio::test]
    async fn test_registry_basic_functionality() {
        let monitor = Arc::new(MockMonitor::new()) as Arc<dyn Monitor>;
        let registry = ResourceRegistry::new(monitor.clone());
        
        // Test initial state
        assert_eq!(registry.active_count().await, 0);
        assert!(registry.is_healthy().await);
        
        // Register a resource
        let resource_id = ResourceId::generate();
        let info = ResourceInfo {
            type_name: "TestResource",
            created_at: Instant::now(),
            size_bytes: Some(1024),
            metadata: HashMap::new(),
        };
        
        let result = registry.register(resource_id, info.clone()).await;
        assert!(result.is_ok());
        assert_eq!(registry.active_count().await, 1);
        
        // Get resource info
        let retrieved_info = registry.get_resource_info(&resource_id).await;
        assert!(retrieved_info.is_some());
        let retrieved_info = retrieved_info.unwrap();
        assert_eq!(retrieved_info.type_name, "TestResource");
        
        // Unregister resource
        let removed = registry.unregister(&resource_id).await;
        assert!(removed.is_some());
        assert_eq!(registry.active_count().await, 0);
    }
}