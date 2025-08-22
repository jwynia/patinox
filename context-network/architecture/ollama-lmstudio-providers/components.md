# Component Specifications: Ollama and LMStudio Providers

**Component Design Date**: August 21, 2025  
**Scope**: Detailed component specifications for local model providers

## Component Hierarchy

```
src/provider/
├── local/                    # New local provider module
│   ├── mod.rs               # Module exports and main LocalProvider
│   ├── discovery.rs         # ServiceDiscovery implementation
│   ├── ollama.rs           # OllamaProvider implementation
│   ├── lmstudio.rs         # LMStudioProvider implementation
│   ├── config.rs           # Local provider configuration
│   ├── error.rs            # Local provider error types
│   └── types.rs            # Local provider specific types
└── mod.rs                   # Updated with local provider exports
```

## 1. LocalProvider (Coordinator)

### Interface Definition

```rust
// src/provider/local/mod.rs

use super::{ModelProvider, ProviderError, ProviderResult};
use crate::provider::types::*;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Coordinator for local model providers (Ollama, LMStudio)
/// 
/// Provides auto-discovery, routing, and fallback for local AI services
pub struct LocalProvider {
    /// Service discovery and health monitoring
    discovery: Arc<ServiceDiscovery>,
    
    /// Ollama provider client (if available)
    ollama: Option<Arc<OllamaProvider>>,
    
    /// LMStudio provider client (if available)
    lmstudio: Option<Arc<LMStudioProvider>>,
    
    /// Provider configuration
    config: LocalProviderConfig,
    
    /// Shared HTTP client for connection pooling
    http_client: reqwest::Client,
    
    /// Cached model information across all providers
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
}
```

### Core Methods

```rust
impl LocalProvider {
    /// Create new local provider with auto-discovery
    pub async fn new() -> ProviderResult<Self> {
        let config = LocalProviderConfig::default();
        Self::with_config(config).await
    }
    
    /// Create with custom configuration
    pub async fn with_config(config: LocalProviderConfig) -> ProviderResult<Self> {
        let http_client = Self::create_http_client(&config)?;
        let discovery = ServiceDiscovery::new(config.discovery_config.clone());
        
        let mut provider = Self {
            discovery: Arc::new(discovery),
            ollama: None,
            lmstudio: None,
            config,
            http_client,
            model_cache: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Perform initial service discovery
        provider.discover_services().await?;
        
        Ok(provider)
    }
    
    /// Manually refresh available services
    pub async fn refresh_services(&mut self) -> ProviderResult<()> {
        self.discover_services().await
    }
    
    /// Get list of available local services
    pub async fn available_services(&self) -> Vec<ServiceType> {
        self.discovery.available_services().await
    }
    
    /// Route request to optimal provider
    async fn route_request(&self, model: &ModelId) -> ProviderResult<&dyn ModelProvider> {
        // Implementation details in routing section
    }
}
```

### ModelProvider Implementation

```rust
#[async_trait]
impl ModelProvider for LocalProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        let mut all_models = Vec::new();
        
        // Aggregate from all available providers
        if let Some(ollama) = &self.ollama {
            if let Ok(models) = ollama.list_models().await {
                all_models.extend(models);
            }
        }
        
        if let Some(lmstudio) = &self.lmstudio {
            if let Ok(models) = lmstudio.list_models().await {
                all_models.extend(models);
            }
        }
        
        // Remove duplicates and sort
        all_models.sort_by(|a, b| a.id.cmp(&b.id));
        all_models.dedup_by(|a, b| a.id == b.id);
        
        Ok(all_models)
    }
    
    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let provider = self.route_request(&request.model).await?;
        provider.complete(request).await
    }
    
    async fn embed(&self, request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        let provider = self.route_request(&request.model).await?;
        provider.embed(request).await
    }
    
    async fn supports_model(&self, model: &ModelId) -> bool {
        // Check all available providers
        if let Some(ollama) = &self.ollama {
            if ollama.supports_model(model).await {
                return true;
            }
        }
        
        if let Some(lmstudio) = &self.lmstudio {
            if lmstudio.supports_model(model).await {
                return true;
            }
        }
        
        false
    }
    
    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        if let Ok(provider) = self.route_request(model).await {
            return provider.model_capabilities(model).await;
        }
        None
    }
    
    fn name(&self) -> &str {
        "local"
    }
}
```

## 2. ServiceDiscovery

### Interface Definition

```rust
// src/provider/local/discovery.rs

use super::types::*;
use super::error::LocalProviderError;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Service discovery and health monitoring for local providers
pub struct ServiceDiscovery {
    /// Configuration for discovery behavior
    config: DiscoveryConfig,
    
    /// Known services and their status
    services: RwLock<HashMap<ServiceType, ServiceInfo>>,
    
    /// HTTP client for health checks
    health_client: reqwest::Client,
    
    /// Last discovery run timestamp
    last_discovery: RwLock<Option<Instant>>,
}

/// Configuration for service discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Services to discover
    pub enabled_services: Vec<ServiceType>,
    
    /// Timeout for initial discovery
    pub discovery_timeout: Duration,
    
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    
    /// Whether to cache discovery results
    pub cache_enabled: bool,
    
    /// Cache TTL for service information
    pub cache_ttl: Duration,
}

/// Health checking configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub interval: Duration,
    
    /// Timeout for health check requests
    pub timeout: Duration,
    
    /// Maximum consecutive failures before marking unavailable
    pub max_failures: u32,
    
    /// Delay before retrying failed services
    pub retry_delay: Duration,
}
```

### Service Types and Info

```rust
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

/// Performance metrics for a service
#[derive(Debug, Clone)]
pub struct ServiceMetrics {
    /// Average response time for health checks
    pub avg_response_time: Duration,
    
    /// Number of models available
    pub model_count: usize,
    
    /// Last request timestamp
    pub last_request: Option<Instant>,
    
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}
```

### Core Discovery Methods

```rust
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
            last_discovery: RwLock::new(None),
        }
    }
    
    /// Discover all enabled services
    pub async fn discover_services(&self) -> Result<Vec<ServiceType>, LocalProviderError> {
        let mut discovered = Vec::new();
        
        for service_type in &self.config.enabled_services {
            if let Ok(info) = self.discover_service(service_type.clone()).await {
                let mut services = self.services.write().await;
                services.insert(service_type.clone(), info);
                discovered.push(service_type.clone());
            }
        }
        
        *self.last_discovery.write().await = Some(Instant::now());
        Ok(discovered)
    }
    
    /// Discover a specific service
    async fn discover_service(&self, service_type: ServiceType) -> Result<ServiceInfo, LocalProviderError> {
        let endpoint = self.get_default_endpoint(&service_type);
        
        // Probe the service
        let health_start = Instant::now();
        let health_response = self.health_client
            .get(&format!("{}/health", endpoint))
            .send()
            .await;
            
        let response_time = health_start.elapsed();
        
        match health_response {
            Ok(response) if response.status().is_success() => {
                // Query available models
                let models = self.query_models(&service_type, &endpoint).await
                    .unwrap_or_default();
                
                Ok(ServiceInfo {
                    service_type,
                    endpoint,
                    version: None, // TODO: Extract from response
                    models,
                    status: ServiceStatus::Available,
                    last_health_check: Instant::now(),
                    consecutive_failures: 0,
                    metrics: ServiceMetrics {
                        avg_response_time: response_time,
                        model_count: models.len(),
                        last_request: Some(Instant::now()),
                        success_rate: 1.0,
                    },
                })
            }
            _ => Err(LocalProviderError::ServiceUnavailable(
                format!("{:?} service not available at {}", service_type, endpoint)
            ))
        }
    }
    
    /// Get default endpoint for service type
    fn get_default_endpoint(&self, service_type: &ServiceType) -> String {
        match service_type {
            ServiceType::Ollama => "http://localhost:11434".to_string(),
            ServiceType::LMStudio => "http://localhost:1234".to_string(),
        }
    }
    
    /// Query available models from a service
    async fn query_models(&self, service_type: &ServiceType, endpoint: &str) -> Result<Vec<String>, LocalProviderError> {
        let models_url = match service_type {
            ServiceType::Ollama => format!("{}/api/tags", endpoint),
            ServiceType::LMStudio => format!("{}/api/v0/models", endpoint),
        };
        
        let response = self.health_client
            .get(&models_url)
            .send()
            .await
            .map_err(|e| LocalProviderError::NetworkError(e))?;
            
        if !response.status().is_success() {
            return Err(LocalProviderError::ServiceError(
                format!("Failed to query models: HTTP {}", response.status())
            ));
        }
        
        // Parse response based on service type
        match service_type {
            ServiceType::Ollama => {
                let ollama_response: OllamaModelsResponse = response.json().await
                    .map_err(|e| LocalProviderError::ParseError(e.to_string()))?;
                Ok(ollama_response.models.into_iter().map(|m| m.name).collect())
            }
            ServiceType::LMStudio => {
                let lmstudio_response: Vec<LMStudioModel> = response.json().await
                    .map_err(|e| LocalProviderError::ParseError(e.to_string()))?;
                Ok(lmstudio_response.into_iter().map(|m| m.id).collect())
            }
        }
    }
    
    /// Get list of currently available services
    pub async fn available_services(&self) -> Vec<ServiceType> {
        let services = self.services.read().await;
        services.iter()
            .filter(|(_, info)| info.status == ServiceStatus::Available)
            .map(|(service_type, _)| service_type.clone())
            .collect()
    }
    
    /// Get service info by type
    pub async fn get_service_info(&self, service_type: &ServiceType) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.get(service_type).cloned()
    }
    
    /// Start background health checking
    pub async fn start_health_monitoring(&self) {
        let discovery = Arc::new(self);
        let interval = self.config.health_check.interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                if let Err(e) = discovery.perform_health_checks().await {
                    log::warn!("Health check failed: {}", e);
                }
            }
        });
    }
    
    /// Perform health checks on all known services
    async fn perform_health_checks(&self) -> Result<(), LocalProviderError> {
        let service_types: Vec<ServiceType> = {
            let services = self.services.read().await;
            services.keys().cloned().collect()
        };
        
        for service_type in service_types {
            self.health_check_service(&service_type).await?;
        }
        
        Ok(())
    }
    
    /// Health check a specific service
    async fn health_check_service(&self, service_type: &ServiceType) -> Result<(), LocalProviderError> {
        let endpoint = {
            let services = self.services.read().await;
            if let Some(info) = services.get(service_type) {
                info.endpoint.clone()
            } else {
                return Err(LocalProviderError::ServiceNotFound(format!("{:?}", service_type)));
            }
        };
        
        let health_url = match service_type {
            ServiceType::Ollama => format!("{}/api/tags", endpoint),
            ServiceType::LMStudio => format!("{}/api/v0/models", endpoint),
        };
        
        let start_time = Instant::now();
        let result = self.health_client
            .get(&health_url)
            .send()
            .await;
            
        let response_time = start_time.elapsed();
        
        // Update service status based on result
        let mut services = self.services.write().await;
        if let Some(info) = services.get_mut(service_type) {
            match result {
                Ok(response) if response.status().is_success() => {
                    info.status = ServiceStatus::Available;
                    info.last_health_check = Instant::now();
                    info.consecutive_failures = 0;
                    info.metrics.avg_response_time = 
                        (info.metrics.avg_response_time + response_time) / 2;
                    info.metrics.last_request = Some(Instant::now());
                }
                _ => {
                    info.consecutive_failures += 1;
                    if info.consecutive_failures >= self.config.health_check.max_failures {
                        info.status = ServiceStatus::Unavailable;
                    } else {
                        info.status = ServiceStatus::Degraded;
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

## 3. OllamaProvider

### Interface Definition

```rust
// src/provider/local/ollama.rs

use super::types::*;
use super::error::LocalProviderError;
use crate::provider::{ModelProvider, ProviderError, ProviderResult};
use crate::provider::types::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Ollama-specific provider implementation
pub struct OllamaProvider {
    /// HTTP client for API requests
    client: reqwest::Client,
    
    /// Base URL for Ollama API
    base_url: String,
    
    /// Provider configuration
    config: OllamaConfig,
    
    /// Cached model information
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
    
    /// Cache timestamp for invalidation
    cache_timestamp: Arc<RwLock<Option<std::time::Instant>>>,
}

/// Ollama-specific configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Request timeout
    pub timeout: std::time::Duration,
    
    /// Enable streaming responses
    pub streaming: bool,
    
    /// Model cache TTL
    pub cache_ttl: std::time::Duration,
    
    /// Custom parameters for Ollama
    pub custom_params: HashMap<String, serde_json::Value>,
}
```

### Ollama API Types

```rust
/// Ollama chat completion request
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
}

/// Ollama message format
#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama chat completion response
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    created_at: String,
    message: OllamaMessage,
    done: bool,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_count: Option<u32>,
    prompt_eval_duration: Option<u64>,
    eval_count: Option<u32>,
    eval_duration: Option<u64>,
}

/// Ollama models list response
#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModelInfo>,
}

/// Ollama model information
#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    size: u64,
    digest: String,
    modified_at: String,
}
```

### Core Implementation

```rust
impl OllamaProvider {
    /// Create new Ollama provider
    pub fn new(endpoint: String) -> ProviderResult<Self> {
        let config = OllamaConfig::default();
        Self::with_config(endpoint, config)
    }
    
    /// Create with custom configuration
    pub fn with_config(endpoint: String, config: OllamaConfig) -> ProviderResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;
            
        Ok(Self {
            client,
            base_url: endpoint,
            config,
            model_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_timestamp: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Check if model cache is valid
    async fn is_cache_valid(&self) -> bool {
        if let Some(timestamp) = *self.cache_timestamp.read().await {
            timestamp.elapsed() < self.config.cache_ttl
        } else {
            false
        }
    }
    
    /// Refresh model cache
    async fn refresh_model_cache(&self) -> ProviderResult<()> {
        let response = self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
            
        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(
                format!("Failed to fetch models: HTTP {}", response.status())
            ));
        }
        
        let ollama_response: OllamaModelsResponse = response.json().await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;
        
        let mut cache = self.model_cache.write().await;
        cache.clear();
        
        for model in ollama_response.models {
            let model_info = ModelInfo {
                id: model.name.clone(),
                object: "model".to_string(),
                created: 0, // Ollama doesn't provide creation timestamp
                owned_by: "ollama".to_string(),
                capabilities: self.infer_model_capabilities(&model.name),
            };
            cache.insert(model.name, model_info);
        }
        
        *self.cache_timestamp.write().await = Some(std::time::Instant::now());
        Ok(())
    }
    
    /// Infer model capabilities from model name
    fn infer_model_capabilities(&self, model_name: &str) -> ModelCapabilities {
        // Basic capability inference based on model name patterns
        let supports_chat = true; // Most Ollama models support chat
        let supports_embeddings = model_name.contains("embed");
        let supports_vision = model_name.contains("vision") || model_name.contains("llava");
        let supports_tools = !model_name.contains("embed"); // Assume non-embedding models support tools
        
        ModelCapabilities {
            max_tokens: Some(if model_name.contains("32k") { 32000 } else { 4096 }),
            supports_chat,
            supports_completions: true,
            supports_embeddings,
            supports_vision,
            supports_tools,
            supports_streaming: true,
            quality_tier: self.infer_quality_tier(model_name),
            speed_tier: self.infer_speed_tier(model_name),
        }
    }
    
    /// Infer quality tier from model name
    fn infer_quality_tier(&self, model_name: &str) -> QualityTier {
        if model_name.contains("70b") || model_name.contains("405b") {
            QualityTier::Premium
        } else if model_name.contains("13b") || model_name.contains("34b") {
            QualityTier::Standard
        } else {
            QualityTier::Basic
        }
    }
    
    /// Infer speed tier from model name
    fn infer_speed_tier(&self, model_name: &str) -> SpeedTier {
        if model_name.contains("q4_0") || model_name.contains("7b") {
            SpeedTier::Fast
        } else if model_name.contains("q8_0") || model_name.contains("13b") {
            SpeedTier::Balanced
        } else {
            SpeedTier::Thoughtful
        }
    }
    
    /// Convert Patinox request to Ollama format
    fn convert_completion_request(&self, request: &CompletionRequest) -> OllamaChatRequest {
        let messages = request.messages.iter()
            .map(|msg| OllamaMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();
        
        let mut options = serde_json::Map::new();
        if let Some(temp) = request.temperature {
            options.insert("temperature".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(temp).unwrap()
            ));
        }
        if let Some(max_tokens) = request.max_tokens {
            options.insert("num_predict".to_string(), serde_json::Value::Number(
                serde_json::Number::from(max_tokens)
            ));
        }
        
        OllamaChatRequest {
            model: request.model.name().to_string(),
            messages,
            stream: Some(false), // TODO: Support streaming
            options: if options.is_empty() { None } else { Some(serde_json::Value::Object(options)) },
        }
    }
    
    /// Convert Ollama response to Patinox format
    fn convert_completion_response(&self, response: OllamaChatResponse) -> CompletionResponse {
        let usage = Usage {
            prompt_tokens: response.prompt_eval_count.unwrap_or(0),
            completion_tokens: response.eval_count.unwrap_or(0),
            total_tokens: response.prompt_eval_count.unwrap_or(0) + response.eval_count.unwrap_or(0),
        };
        
        CompletionResponse {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: response.model,
            choices: vec![CompletionChoice {
                index: 0,
                message: CompletionMessage::assistant(response.message.content),
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(usage),
        }
    }
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        if !self.is_cache_valid().await {
            self.refresh_model_cache().await?;
        }
        
        let cache = self.model_cache.read().await;
        Ok(cache.values().cloned().collect())
    }
    
    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let ollama_request = self.convert_completion_request(&request);
        
        let response = self.client
            .post(&format!("{}/api/chat", self.base_url))
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
            
        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(
                format!("Ollama API error: HTTP {}", response.status())
            ));
        }
        
        let ollama_response: OllamaChatResponse = response.json().await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;
            
        Ok(self.convert_completion_response(ollama_response))
    }
    
    async fn embed(&self, request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        // Ollama embeddings implementation
        let embed_request = serde_json::json!({
            "model": request.model.name(),
            "prompt": request.input
        });
        
        let response = self.client
            .post(&format!("{}/api/embeddings", self.base_url))
            .json(&embed_request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;
            
        if !response.status().is_success() {
            return Err(ProviderError::NetworkError(
                format!("Ollama embeddings error: HTTP {}", response.status())
            ));
        }
        
        // Parse embeddings response and convert to Patinox format
        // Implementation depends on Ollama's embeddings response format
        todo!("Implement embeddings response parsing")
    }
    
    async fn supports_model(&self, model: &ModelId) -> bool {
        if !self.is_cache_valid().await {
            if self.refresh_model_cache().await.is_err() {
                return false;
            }
        }
        
        let cache = self.model_cache.read().await;
        cache.contains_key(model.name())
    }
    
    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        if !self.is_cache_valid().await {
            if self.refresh_model_cache().await.is_err() {
                return None;
            }
        }
        
        let cache = self.model_cache.read().await;
        cache.get(model.name()).map(|info| info.capabilities.clone())
    }
    
    fn name(&self) -> &str {
        "ollama"
    }
}
```

## 4. LMStudioProvider

### Interface Definition

```rust
// src/provider/local/lmstudio.rs

/// LMStudio-specific provider implementation
pub struct LMStudioProvider {
    /// HTTP client for API requests  
    client: reqwest::Client,
    
    /// Base URL for LMStudio API
    base_url: String,
    
    /// Provider configuration
    config: LMStudioConfig,
    
    /// Cached model information
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
    
    /// Cache timestamp for invalidation
    cache_timestamp: Arc<RwLock<Option<std::time::Instant>>>,
}

/// LMStudio-specific configuration
#[derive(Debug, Clone)]
pub struct LMStudioConfig {
    /// Request timeout
    pub timeout: std::time::Duration,
    
    /// Use OpenAI-compatible endpoints vs LMStudio-specific
    pub use_openai_compat: bool,
    
    /// Model cache TTL
    pub cache_ttl: std::time::Duration,
    
    /// Custom parameters for LMStudio
    pub custom_params: HashMap<String, serde_json::Value>,
}
```

### LMStudio API Types

```rust
/// LMStudio model information from /api/v0/models
#[derive(Debug, Deserialize)]
struct LMStudioModel {
    id: String,
    object: String,
    #[serde(rename = "type")]
    model_type: String,
    publisher: String,
    architecture: String,
    quantization: String,
    context_length: u32,
    state: String, // "loaded" | "unloaded"
}

/// OpenAI-compatible chat completion request
#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// OpenAI-compatible message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible chat completion response
#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
```

### Implementation Strategy

The LMStudioProvider implementation will:

1. **Leverage OpenAI Compatibility**: Use `/v1/chat/completions` endpoints for maximum compatibility
2. **Enhanced Model Management**: Use `/api/v0/models` for detailed model information
3. **Performance Optimization**: Cache model states and capabilities
4. **Error Handling**: Map LMStudio-specific errors to Patinox error types

**Key Implementation Pattern**: Similar to OllamaProvider but leveraging OpenAI-compatible endpoints where possible.

## 5. Configuration Integration

### Extended Provider Enum

```rust
// Update in src/provider/config.rs

pub enum Provider {
    // ... existing variants
    
    /// Generic local provider with auto-discovery
    Local {
        endpoint: String,
        model_path: Option<String>,
        preferred_service: Option<LocalService>,
        auto_discover: bool,
    },
    
    /// Ollama-specific provider
    Ollama {
        endpoint: String,
        models_path: Option<String>,
        config: Option<OllamaConfig>,
    },
    
    /// LMStudio-specific provider  
    LMStudio {
        endpoint: String,
        models_path: Option<String>,
        config: Option<LMStudioConfig>,
    },
}

/// Local service type preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalService {
    Ollama,
    LMStudio,
    Auto, // Auto-detect best service
}
```

### Environment Variable Support

```rust
/// Environment variables for local provider configuration
const OLLAMA_ENDPOINT: &str = "OLLAMA_ENDPOINT";
const LMSTUDIO_ENDPOINT: &str = "LMSTUDIO_ENDPOINT"; 
const LOCAL_MODELS_PATH: &str = "LOCAL_MODELS_PATH";
const LOCAL_PROVIDER_PREFERENCE: &str = "LOCAL_PROVIDER_PREFERENCE";
```

## Success Criteria

### Component Quality Standards

1. **Complete Implementation**: All `ModelProvider` trait methods fully implemented
2. **Comprehensive Testing**: Unit tests for each component with >95% coverage
3. **Error Handling**: All error paths properly handled and mapped
4. **Performance**: Response times within 10% of direct API calls
5. **Documentation**: Complete API documentation with examples
6. **Integration**: Seamless integration with existing provider framework

### Integration Success Metrics

1. **Zero Breaking Changes**: Existing provider functionality unchanged
2. **Configuration Compatibility**: Works with existing configuration patterns
3. **Monitoring Integration**: Metrics properly integrated with telemetry system
4. **Test Coverage**: Maintains project's comprehensive testing standards