//! Configuration for local model providers

use crate::provider::local::error::{LocalProviderError, LocalProviderResult};
use std::time::Duration;
// Note: Serde derives may be added later when needed for configuration serialization

/// Validate that a URL is properly formatted and uses HTTP/HTTPS
pub fn validate_endpoint_url(url: &str) -> LocalProviderResult<()> {
    if url.is_empty() {
        return Err(LocalProviderError::InvalidConfiguration(
            "Endpoint URL cannot be empty".to_string(),
        ));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(LocalProviderError::InvalidConfiguration(
            "Endpoint URL must start with http:// or https://".to_string(),
        ));
    }

    // Basic URL parsing validation using reqwest
    url.parse::<reqwest::Url>().map_err(|e| {
        LocalProviderError::InvalidConfiguration(format!("Invalid endpoint URL: {}", e))
    })?;

    Ok(())
}

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

impl LocalProviderConfig {
    /// Validate the configuration
    pub fn validate(&self) -> LocalProviderResult<()> {
        // Validate timeout values
        if self.request_timeout.as_secs() == 0 {
            return Err(LocalProviderError::InvalidConfiguration(
                "Request timeout must be greater than 0".to_string(),
            ));
        }

        if self.request_timeout > Duration::from_secs(300) {
            return Err(LocalProviderError::InvalidConfiguration(
                "Request timeout should not exceed 300 seconds".to_string(),
            ));
        }

        // Validate connection pool size
        if self.connection_pool_size == 0 {
            return Err(LocalProviderError::InvalidConfiguration(
                "Connection pool size must be greater than 0".to_string(),
            ));
        }

        if self.connection_pool_size > 100 {
            return Err(LocalProviderError::InvalidConfiguration(
                "Connection pool size should not exceed 100 connections".to_string(),
            ));
        }

        // Validate discovery configuration
        self.discovery.validate()?;

        Ok(())
    }
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

impl DiscoveryConfig {
    /// Validate the discovery configuration
    pub fn validate(&self) -> LocalProviderResult<()> {
        // Validate discovery timeout
        if self.discovery_timeout.as_secs() == 0 {
            return Err(LocalProviderError::InvalidConfiguration(
                "Discovery timeout must be greater than 0".to_string(),
            ));
        }

        if self.discovery_timeout > Duration::from_secs(60) {
            return Err(LocalProviderError::InvalidConfiguration(
                "Discovery timeout should not exceed 60 seconds".to_string(),
            ));
        }

        // Validate health check configuration
        self.health_check.validate()?;

        Ok(())
    }
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

impl HealthCheckConfig {
    /// Validate the health check configuration
    pub fn validate(&self) -> LocalProviderResult<()> {
        // Validate health check interval
        if self.interval.as_secs() == 0 {
            return Err(LocalProviderError::InvalidConfiguration(
                "Health check interval must be greater than 0".to_string(),
            ));
        }

        if self.interval > Duration::from_secs(3600) {
            return Err(LocalProviderError::InvalidConfiguration(
                "Health check interval should not exceed 1 hour".to_string(),
            ));
        }

        // Validate health check timeout
        if self.timeout.as_secs() == 0 {
            return Err(LocalProviderError::InvalidConfiguration(
                "Health check timeout must be greater than 0".to_string(),
            ));
        }

        if self.timeout > Duration::from_secs(30) {
            return Err(LocalProviderError::InvalidConfiguration(
                "Health check timeout should not exceed 30 seconds".to_string(),
            ));
        }

        // Validate that timeout is less than interval
        if self.timeout >= self.interval {
            return Err(LocalProviderError::InvalidConfiguration(
                "Health check timeout must be less than interval".to_string(),
            ));
        }

        // Validate max failures
        if self.max_failures == 0 {
            return Err(LocalProviderError::InvalidConfiguration(
                "Max failures must be greater than 0".to_string(),
            ));
        }

        if self.max_failures > 10 {
            return Err(LocalProviderError::InvalidConfiguration(
                "Max failures should not exceed 10".to_string(),
            ));
        }

        Ok(())
    }
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
