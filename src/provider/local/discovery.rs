//! Service discovery for local model providers

use super::config::DiscoveryConfig;
use super::error::{LocalProviderError, LocalProviderResult};
use super::types::ServiceMetrics;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache TTL for service discovery results (30 seconds)
const DISCOVERY_CACHE_TTL_SECONDS: u64 = 30;

/// Cache TTL for model discovery results (5 minutes)  
const MODEL_CACHE_TTL_SECONDS: u64 = 300;

/// Service discovery and health monitoring for local providers
pub struct ServiceDiscovery {
    /// Configuration for discovery behavior
    config: DiscoveryConfig,

    /// Known services and their status
    services: RwLock<HashMap<ServiceType, ServiceInfo>>,

    /// HTTP client for health checks
    health_client: reqwest::Client,

    /// Discovery cache with TTL
    discovery_cache: RwLock<Option<CacheEntry<Vec<ServiceType>>>>,

    /// Model cache with TTL per service
    model_cache: RwLock<HashMap<ServiceType, CacheEntry<Vec<String>>>>,
}

/// Cache entry with TTL support
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// The cached data
    data: T,

    /// When this entry was created
    created_at: Instant,

    /// How long this entry is valid
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Types of local AI services we can discover
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceType {
    Ollama,
    LMStudio,
}

impl ServiceType {
    /// Get the default endpoint URL for this service type
    pub fn default_endpoint(&self) -> &'static str {
        match self {
            ServiceType::Ollama => "http://localhost:11434",
            ServiceType::LMStudio => "http://localhost:1234",
        }
    }
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
    pub fn new(config: DiscoveryConfig) -> LocalProviderResult<Self> {
        let health_client = reqwest::Client::builder()
            .timeout(config.health_check.timeout)
            .build()
            .map_err(|e| LocalProviderError::InvalidConfiguration(
                format!("Failed to create HTTP client: {}", e)
            ))?;

        Ok(Self {
            config,
            services: RwLock::new(HashMap::new()),
            health_client,
            discovery_cache: RwLock::new(None),
            model_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Discover all enabled services
    pub async fn discover_services(&self) -> LocalProviderResult<Vec<ServiceType>> {
        // Check cache first
        {
            let cache = self.discovery_cache.read().await;
            if let Some(entry) = cache.as_ref() {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        // Cache miss or expired, perform actual discovery
        let mut discovered_services = Vec::new();

        // Probe Ollama (port 11434)
        if let Ok(Some(_)) = self
            .probe_service(ServiceType::Ollama, ServiceType::Ollama.default_endpoint())
            .await
        {
            discovered_services.push(ServiceType::Ollama);
        }

        // Probe LMStudio (port 1234)
        if let Ok(Some(_)) = self
            .probe_service(
                ServiceType::LMStudio,
                ServiceType::LMStudio.default_endpoint(),
            )
            .await
        {
            discovered_services.push(ServiceType::LMStudio);
        }

        // Update cache with discovery TTL
        {
            let mut cache = self.discovery_cache.write().await;
            *cache = Some(CacheEntry::new(
                discovered_services.clone(),
                Duration::from_secs(DISCOVERY_CACHE_TTL_SECONDS),
            ));
        }

        Ok(discovered_services)
    }

    /// Get list of currently available services
    pub async fn available_services(&self) -> Vec<ServiceType> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|info| info.status == ServiceStatus::Available)
            .map(|info| info.service_type.clone())
            .collect()
    }

    /// Get service info by type
    pub async fn get_service_info(&self, service_type: &ServiceType) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.get(service_type).cloned()
    }

    /// Probe a specific service at the given endpoint
    pub async fn probe_service(
        &self,
        service_type: ServiceType,
        endpoint: &str,
    ) -> LocalProviderResult<Option<ServiceInfo>> {
        // Try to perform a health check first
        match self.health_check(&service_type, endpoint).await {
            Ok(()) => {
                // Service is healthy, create service info
                let service_info = ServiceInfo {
                    service_type: service_type.clone(),
                    endpoint: endpoint.to_string(),
                    version: None,      // Will be populated by health check later
                    models: Vec::new(), // Will be populated by model discovery later
                    status: ServiceStatus::Available,
                    last_health_check: Instant::now(),
                    consecutive_failures: 0,
                    metrics: ServiceMetrics::default(),
                };

                // Cache the service info
                {
                    let mut services = self.services.write().await;
                    services.insert(service_type, service_info.clone());
                }

                Ok(Some(service_info))
            }
            Err(_e) => {
                // Service is not available - log for debugging if needed
                // tracing::debug!("Service probe failed for {:?} at {}: {}", service_type, endpoint, e);
                Ok(None)
            }
        }
    }

    /// Perform health check on a service
    pub async fn health_check(
        &self,
        service_type: &ServiceType,
        endpoint: &str,
    ) -> LocalProviderResult<()> {
        let health_endpoint = match service_type {
            ServiceType::Ollama => format!("{}/api/tags", endpoint),
            ServiceType::LMStudio => format!("{}/v1/models", endpoint),
        };

        let response = tokio::time::timeout(
            self.config.health_check.timeout,
            self.health_client.get(&health_endpoint).send(),
        )
        .await;

        match response {
            Ok(Ok(resp)) => {
                if resp.status().is_success() {
                    Ok(())
                } else {
                    Err(super::error::LocalProviderError::ServiceError(format!(
                        "Health check failed with status: {}",
                        resp.status()
                    )))
                }
            }
            Ok(Err(e)) => Err(super::error::LocalProviderError::NetworkError(e)),
            Err(_) => Err(super::error::LocalProviderError::ServiceTimeout(
                self.config.health_check.timeout,
            )),
        }
    }

    /// Discover available models for a service type
    pub async fn discover_models(
        &self,
        service_type: &ServiceType,
    ) -> LocalProviderResult<Vec<String>> {
        // Check model cache first
        {
            let cache = self.model_cache.read().await;
            if let Some(entry) = cache.get(service_type) {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        // Cache miss or expired, fetch models
        let service_info = {
            let services = self.services.read().await;
            services.get(service_type).cloned()
        };

        let endpoint = if let Some(info) = service_info {
            info.endpoint
        } else {
            // Use default endpoint for service type
            service_type.default_endpoint().to_string()
        };

        let models = self.fetch_models(service_type, &endpoint).await?;

        // Update cache with model TTL
        {
            let mut cache = self.model_cache.write().await;
            cache.insert(
                service_type.clone(),
                CacheEntry::new(models.clone(), Duration::from_secs(MODEL_CACHE_TTL_SECONDS)),
            );
        }

        Ok(models)
    }

    /// Fetch models from a specific service endpoint
    async fn fetch_models(
        &self,
        service_type: &ServiceType,
        endpoint: &str,
    ) -> LocalProviderResult<Vec<String>> {
        let models_endpoint = match service_type {
            ServiceType::Ollama => format!("{}/api/tags", endpoint),
            ServiceType::LMStudio => format!("{}/v1/models", endpoint),
        };

        let response = tokio::time::timeout(
            self.config.discovery_timeout,
            self.health_client.get(&models_endpoint).send(),
        )
        .await;

        match response {
            Ok(Ok(resp)) => {
                if resp.status().is_success() {
                    let text = resp
                        .text()
                        .await
                        .map_err(super::error::LocalProviderError::NetworkError)?;

                    // Parse the response based on service type
                    let models = match service_type {
                        ServiceType::Ollama => self.parse_ollama_models(&text)?,
                        ServiceType::LMStudio => self.parse_lmstudio_models(&text)?,
                    };

                    Ok(models)
                } else {
                    Err(super::error::LocalProviderError::ServiceError(format!(
                        "Models fetch failed with status: {}",
                        resp.status()
                    )))
                }
            }
            Ok(Err(e)) => Err(super::error::LocalProviderError::NetworkError(e)),
            Err(_) => Err(super::error::LocalProviderError::ServiceTimeout(
                self.config.discovery_timeout,
            )),
        }
    }

    /// Parse Ollama models response
    fn parse_ollama_models(&self, response: &str) -> LocalProviderResult<Vec<String>> {
        // Ollama returns JSON with "models" array
        // Each model has a "name" field
        use serde_json::Value;

        let json: Value = serde_json::from_str(response).map_err(|e| {
            super::error::LocalProviderError::ParseError(format!(
                "Failed to parse Ollama response: {}",
                e
            ))
        })?;

        if let Some(models) = json["models"].as_array() {
            let model_names: Vec<String> = models
                .iter()
                .filter_map(|model| model["name"].as_str().map(String::from))
                .collect();
            Ok(model_names)
        } else {
            Ok(Vec::new())
        }
    }

    /// Parse LMStudio models response  
    fn parse_lmstudio_models(&self, response: &str) -> LocalProviderResult<Vec<String>> {
        // LMStudio returns OpenAI-compatible models list
        // Each model has an "id" field
        use serde_json::Value;

        let json: Value = serde_json::from_str(response).map_err(|e| {
            super::error::LocalProviderError::ParseError(format!(
                "Failed to parse LMStudio response: {}",
                e
            ))
        })?;

        if let Some(models) = json["data"].as_array() {
            let model_names: Vec<String> = models
                .iter()
                .filter_map(|model| model["id"].as_str().map(String::from))
                .collect();
            Ok(model_names)
        } else {
            Ok(Vec::new())
        }
    }
}
