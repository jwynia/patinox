//! Memory Management Utilities
//!
//! This module provides foundational utilities for memory management, resource cleanup,
//! connection pooling, and data sharing across the Patinox framework.
//!
//! ## Components
//!
//! - **Resource Management**: [`AsyncResourceGuard`] and [`ResourceRegistry`] for safe
//!   resource cleanup and tracking
//! - **Connection Pooling**: Generic connection pools with health checking and fair scheduling
//! - **Data Sharing**: Efficient data sharing with Arc-based patterns and copy-on-write
//! - **Caching**: Multi-policy caching with TTL and metrics
//!
//! ## Design Principles
//!
//! - **Async-First**: All components designed for tokio compatibility
//! - **Type Safety**: Leverage Rust's type system for resource safety
//! - **Zero-Cost**: Abstractions that compile to optimal code
//! - **Integration**: Seamless integration with existing error and monitoring systems
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use patinox::memory::{AsyncResourceGuard, ResourceRegistry};
//! use patinox::traits::Monitor;
//! use std::sync::Arc;
//!
//! # // Note: In real usage, add async-trait to Cargo.toml dependencies
//!
//! // Mock monitor for example
//! #[derive(Debug)]
//! struct ExampleMonitor;
//!
//! #[async_trait::async_trait]
//! impl Monitor for ExampleMonitor {
//!     fn name(&self) -> &str { "example" }
//!     async fn start_monitoring(&self, _: uuid::Uuid, _: uuid::Uuid) -> Result<(), patinox::error::PatinoxError> { Ok(()) }
//!     async fn record_event(&self, _: patinox::traits::MonitorEvent) -> Result<(), patinox::error::PatinoxError> { Ok(()) }
//!     async fn complete_monitoring(&self, _: uuid::Uuid, _: patinox::traits::ExecutionSummary) -> Result<(), patinox::error::PatinoxError> { Ok(()) }
//!     async fn query_events(&self, _: patinox::traits::MonitorQuery) -> Result<Vec<patinox::traits::MonitorEvent>, patinox::error::PatinoxError> { Ok(Vec::new()) }
//!     fn config(&self) -> &patinox::traits::MonitorConfig {
//!         use patinox::traits::{MonitorConfig, MonitorEventType};
//!         static CONFIG: std::sync::OnceLock<MonitorConfig> = std::sync::OnceLock::new();
//!         CONFIG.get_or_init(|| MonitorConfig {
//!             name: "example".to_string(), enabled: true, buffer_size: 1000,
//!             flush_interval_ms: 5000, sampling_rate: 1.0,
//!             event_types: vec![MonitorEventType::ExecutionStarted],
//!         })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a registry for tracking resources (requires Monitor)
//!     let monitor = Arc::new(ExampleMonitor) as Arc<dyn Monitor>;
//!     let _registry = ResourceRegistry::new(monitor);
//!     
//!     // Create a resource guard that will clean up automatically
//!     let guard = AsyncResourceGuard::new(
//!         "example resource",
//!         |resource| async move {
//!             println!("Cleaning up: {}", resource);
//!             Ok(())
//!         }
//!     );
//!     
//!     // Use the resource...
//!     println!("Using resource: {}", guard.get());
//!     
//!     // Resource will be cleaned up when guard is dropped
//!     Ok(())
//! }
//! ```

pub mod registry;
pub mod resource;

// Re-export main types for convenient access
pub use registry::{CleanupRequest, ResourceInfo, ResourceRegistry};
pub use resource::{AsyncResourceGuard, CleanupError, CleanupFn, CleanupPriority, ResourceId};
