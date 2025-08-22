//! Configuration for local model providers

use std::time::Duration;
// Note: Serde derives may be added later when needed for configuration serialization

/// Configuration for local model providers
#[derive(Debug, Clone)]
pub struct LocalProviderConfig {
    /// Request timeout for HTTP calls
    pub request_timeout: Duration,
    
    /// Connection pool size per host
    pub connection_pool_size: usize,
    
    /// Service discovery configuration
    pub discovery: DiscoveryConfig,
}

impl Default for LocalProviderConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            connection_pool_size: 10,
            discovery: DiscoveryConfig::default(),
        }
    }
}

/// Configuration for service discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Timeout for discovery operations
    pub discovery_timeout: Duration,
    
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_timeout: Duration::from_secs(5),
            health_check: HealthCheckConfig::default(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub interval: Duration,
    
    /// Timeout for each health check
    pub timeout: Duration,
    
    /// Maximum consecutive failures before marking unavailable
    pub max_failures: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            max_failures: 3,
        }
    }
}