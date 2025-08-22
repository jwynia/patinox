//! Service discovery for local model providers

use super::config::DiscoveryConfig;
use super::error::LocalProviderResult;
use super::types::ServiceMetrics;
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::RwLock;

/// Service discovery and health monitoring for local providers
pub struct ServiceDiscovery {
    /// Configuration for discovery behavior
    config: DiscoveryConfig,
    
    /// Known services and their status
    services: RwLock<HashMap<ServiceType, ServiceInfo>>,
    
    /// HTTP client for health checks
    health_client: reqwest::Client,
}

/// Types of local AI services we can discover
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceType {
    Ollama,
    LMStudio,
}

/// Information about a discovered service
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service type
    pub service_type: ServiceType,
    
    /// Service endpoint URL
    pub endpoint: String,
    
    /// Service version (if available)
    pub version: Option<String>,
    
    /// Available models
    pub models: Vec<String>,
    
    /// Current service status
    pub status: ServiceStatus,
    
    /// Last successful health check
    pub last_health_check: Instant,
    
    /// Consecutive health check failures
    pub consecutive_failures: u32,
    
    /// Performance metrics
    pub metrics: ServiceMetrics,
}

/// Service availability status
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    /// Service is available and healthy
    Available,
    
    /// Service is available but degraded
    Degraded,
    
    /// Service is unavailable
    Unavailable,
    
    /// Service discovery has not been attempted
    Unknown,
}

impl ServiceDiscovery {
    /// Create new service discovery with configuration
    pub fn new(config: DiscoveryConfig) -> Self {
        let health_client = reqwest::Client::builder()
            .timeout(config.health_check.timeout)
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            config,
            services: RwLock::new(HashMap::new()),
            health_client,
        }
    }
    
    /// Discover all enabled services
    pub async fn discover_services(&self) -> LocalProviderResult<Vec<ServiceType>> {
        // For now, return empty list - will implement in next phase
        Ok(Vec::new())
    }
    
    /// Get list of currently available services
    pub async fn available_services(&self) -> Vec<ServiceType> {
        // For now, return empty list - will implement in next phase
        Vec::new()
    }
    
    /// Get service info by type
    pub async fn get_service_info(&self, _service_type: &ServiceType) -> Option<ServiceInfo> {
        // For now, return None - will implement in next phase
        None
    }
}