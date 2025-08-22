# Architecture Overview: Ollama and LMStudio Providers

**Architecture Date**: August 21, 2025  
**Scope**: Local model provider integration design  
**Status**: Architecture Design Phase

## Architecture Strategy: Hybrid Approach

Based on research findings, I recommend a **hybrid approach** that balances flexibility with maintainability:

### Core Design Principles

1. **Leverage Existing Infrastructure**: Build upon proven provider abstraction patterns
2. **Service Auto-Discovery**: Runtime detection of available local services
3. **Graceful Degradation**: Handle missing/unavailable services elegantly
4. **Performance Optimization**: Efficient connection management and model handling
5. **Testing Compatibility**: Maintain comprehensive test coverage standards

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Patinox Provider Layer                   │
├─────────────────────────────────────────────────────────────┤
│  create_default_provider() → Provider::Local Detection     │
├─────────────────────────────────────────────────────────────┤
│                  LocalProvider (Coordinator)               │
│  ┌─────────────────┐    ┌─────────────────────────────────┐ │
│  │ Service         │    │ Provider Routing                │ │
│  │ Discovery       │    │ - Ollama: localhost:11434      │ │
│  │ - Probe ports   │    │ - LMStudio: localhost:1234     │ │
│  │ - Check health  │    │ - Auto-select best match       │ │
│  │ - Cache results │    │ - Fallback handling             │ │
│  └─────────────────┘    └─────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│        OllamaProvider           │      LMStudioProvider     │
│  ┌─────────────────────────┐    │  ┌───────────────────────┐ │
│  │ API: /api/chat          │    │  │ API: /v1/chat/...     │ │
│  │ Streaming: Native       │    │  │ OpenAI Compatible     │ │
│  │ Models: Ollama format   │    │  │ Models: GGUF/GGML     │ │
│  │ Metrics: Detailed       │    │  │ Metrics: Enhanced     │ │
│  └─────────────────────────┘    │  └───────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                     HTTP Transport Layer                    │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ reqwest::Client with connection pooling                 │ │
│  │ - Timeout handling    - Connection reuse               │ │
│  │ - Retry strategies    - Health checking                │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. LocalProvider (Coordinator)

**Role**: Primary coordinator that discovers and routes to appropriate local services

```rust
pub struct LocalProvider {
    service_discovery: ServiceDiscovery,
    ollama_client: Option<OllamaProvider>,
    lmstudio_client: Option<LMStudioProvider>,
    config: LocalProviderConfig,
    http_client: reqwest::Client,
}
```

**Responsibilities**:
- Service discovery and health checking
- Provider routing and fallback logic
- Configuration management
- Performance monitoring and metrics aggregation

### 2. ServiceDiscovery

**Role**: Detect and manage available local services

```rust
pub struct ServiceDiscovery {
    known_services: HashMap<ServiceType, ServiceInfo>,
    health_check_interval: Duration,
    last_discovery: Option<Instant>,
}

pub enum ServiceType {
    Ollama,
    LMStudio,
}

pub struct ServiceInfo {
    endpoint: String,
    version: Option<String>,
    available_models: Vec<String>,
    last_health_check: Instant,
    status: ServiceStatus,
}
```

**Responsibilities**:
- Port probing (11434 for Ollama, 1234 for LMStudio)
- Health checking and service validation
- Model discovery and capability detection
- Cache management and refresh strategies

### 3. OllamaProvider

**Role**: Ollama-specific API client optimized for Ollama's REST API

```rust
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    config: OllamaConfig,
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
}
```

**Key Features**:
- Native Ollama API integration (`/api/chat`, `/api/generate`)
- Streaming support with async streams
- Detailed performance metrics extraction
- Model management (pull, delete, list)
- Custom parameter mapping

### 4. LMStudioProvider

**Role**: LMStudio-specific client leveraging OpenAI compatibility

```rust
pub struct LMStudioProvider {
    client: reqwest::Client,
    base_url: String,
    config: LMStudioConfig,
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
}
```

**Key Features**:
- OpenAI-compatible API usage (`/v1/chat/completions`)
- Enhanced LMStudio endpoints (`/api/v0/models`)
- Structured output support
- Model state management (load/unload)
- Performance metric integration

## Integration with Existing Framework

### Configuration Integration

**Extend Provider Enum**:
```rust
pub enum Provider {
    // ... existing variants
    Local {
        endpoint: String,
        model_path: Option<String>,
        // NEW: Specify preferred service
        preferred_service: Option<LocalService>,
        // NEW: Auto-discovery settings
        auto_discover: bool,
    },
    // NEW: Service-specific configurations
    Ollama {
        endpoint: String,
        models_path: Option<String>,
    },
    LMStudio {
        endpoint: String,
        models_path: Option<String>,
    },
}
```

**Configuration Cascading**:
```
Global Config → Agent Config → Request Config
      ↓              ↓              ↓
Local Service Discovery → Provider Selection → API Call
```

### ModelProvider Trait Implementation

All local providers implement the existing `ModelProvider` trait:

```rust
#[async_trait::async_trait]
impl ModelProvider for LocalProvider {
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        // Aggregate from all available services
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        // Route to best available provider
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, ProviderError> {
        // Route to provider with embedding support
    }

    async fn supports_model(&self, model: &ModelId) -> bool {
        // Check across all available services
    }

    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities> {
        // Query capabilities from appropriate service
    }

    fn name(&self) -> &str {
        "local"
    }
}
```

## Service Discovery Architecture

### Discovery Process

1. **Port Probing**: Check standard ports (11434, 1234) for responsive services
2. **Service Validation**: Send health check requests to validate service type
3. **Model Discovery**: Query each service for available models
4. **Capability Assessment**: Determine what each service can do (chat, embeddings, etc.)
5. **Cache Results**: Store discovery results with TTL for performance

### Health Checking Strategy

```rust
pub struct HealthCheckConfig {
    pub initial_check: Duration,
    pub periodic_interval: Duration,
    pub failure_retry_delay: Duration,
    pub max_consecutive_failures: u32,
}
```

**Health Check Implementation**:
- Lightweight endpoint probes every 30 seconds
- Model list refresh every 5 minutes
- Exponential backoff on failures
- Circuit breaker pattern for persistently failing services

## Request Routing Logic

### Model-Based Routing

```rust
pub async fn route_request(&self, model: &ModelId) -> Result<&dyn ModelProvider, ProviderError> {
    // 1. Check if model explicitly specifies provider
    if let Some(provider_hint) = model.provider_hint() {
        return self.get_provider_by_hint(provider_hint);
    }
    
    // 2. Find providers that support this model
    let supporting_providers = self.find_supporting_providers(model).await?;
    
    // 3. Select best provider based on:
    //    - Model availability and loading status
    //    - Current performance metrics
    //    - Provider-specific optimizations
    //    - Configuration preferences
    
    self.select_optimal_provider(supporting_providers, model)
}
```

### Fallback Strategy

1. **Primary Provider**: Use service that best matches the model
2. **Secondary Fallback**: Try alternative service if primary fails
3. **Graceful Degradation**: Return clear error if no services available
4. **Recovery Handling**: Auto-retry on transient failures

## Performance Optimization

### Connection Management

```rust
pub struct LocalProviderConfig {
    pub connection_pool_size: usize,
    pub request_timeout: Duration,
    pub connection_timeout: Duration,
    pub keep_alive_timeout: Duration,
    pub max_retries: u32,
}
```

### Model Caching Strategy

- **Model Info Cache**: Cache model lists and capabilities (5-minute TTL)
- **Connection Pooling**: Reuse HTTP connections to local services
- **Request Batching**: Group compatible requests when possible
- **Response Streaming**: Minimize memory usage for large responses

### Metrics Integration

**Performance Metrics Mapping**:
```rust
pub struct LocalModelMetrics {
    pub provider_type: String,           // "ollama" | "lmstudio"
    pub model_name: String,
    pub tokens_per_second: Option<f64>,
    pub time_to_first_token: Option<Duration>,
    pub total_duration: Duration,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}
```

## Error Handling Strategy

### Error Mapping

```rust
pub enum LocalProviderError {
    ServiceUnavailable(String),
    ModelNotFound(String),
    ModelNotLoaded(String),
    ServiceTimeout(Duration),
    InvalidConfiguration(String),
    NetworkError(reqwest::Error),
}

impl From<LocalProviderError> for ProviderError {
    fn from(err: LocalProviderError) -> Self {
        match err {
            LocalProviderError::ServiceUnavailable(msg) => 
                ProviderError::NetworkError(format!("Local service unavailable: {}", msg)),
            // ... other mappings
        }
    }
}
```

### Recovery Strategies

- **Service Unavailable**: Auto-discovery retry, fallback to alternative service
- **Model Not Found**: Search alternative services, suggest similar models
- **Timeout**: Retry with longer timeout, switch to faster model if available
- **Network Error**: Connection pool refresh, service health re-check

## Security Considerations

### Local Trust Model

1. **Service Validation**: Verify responding services are genuine Ollama/LMStudio
2. **Port Security**: Only connect to localhost by default
3. **Request Validation**: Sanitize all requests to prevent injection
4. **Resource Limits**: Implement safeguards against resource exhaustion

### Configuration Security

- **Endpoint Validation**: Validate all configured endpoints
- **Path Traversal Protection**: Sanitize model paths and filenames
- **Network Isolation**: Default to localhost-only connections
- **Service Authentication**: Support future authentication if added to services

## Testing Strategy

### Test Categories

1. **Unit Tests**: Individual provider and component testing
2. **Integration Tests**: Mock local services with comprehensive scenarios
3. **End-to-End Tests**: Real service integration (optional/conditional)
4. **Performance Tests**: Latency, throughput, and resource usage
5. **Failure Tests**: Service unavailability, network issues, model errors

### Mock Service Implementation

```rust
pub struct MockLocalService {
    service_type: ServiceType,
    models: Vec<String>,
    response_delay: Duration,
    failure_rate: f64,
}
```

**Mock Capabilities**:
- Simulate both Ollama and LMStudio APIs
- Configurable response times and failure rates
- Model availability simulation
- Health check endpoint simulation

## Implementation Priority

### Phase 1: Foundation (High Priority)
1. **ServiceDiscovery**: Core discovery and health checking
2. **LocalProvider**: Basic coordinator with routing
3. **OllamaProvider**: Complete Ollama integration
4. **Configuration**: Extend existing config system

### Phase 2: Enhancement (Medium Priority)
1. **LMStudioProvider**: Complete LMStudio integration
2. **Advanced Routing**: Model-based optimization
3. **Performance Monitoring**: Metrics collection and reporting
4. **Comprehensive Testing**: Full test suite

### Phase 3: Optimization (Lower Priority)
1. **Advanced Caching**: Intelligent model caching
2. **Load Balancing**: Multi-instance support
3. **Network Support**: Remote local service support
4. **Advanced Fallback**: Sophisticated failure handling

## Success Metrics

### Functional Success
- [ ] Complete `ModelProvider` trait implementation
- [ ] Auto-discovery of available services
- [ ] Successful request routing and fallback
- [ ] Comprehensive error handling

### Quality Success
- [ ] 95%+ test coverage matching existing providers
- [ ] Performance within 10% of direct API calls
- [ ] Zero security vulnerabilities
- [ ] Documentation quality matching existing providers

### Integration Success
- [ ] Seamless integration with existing provider framework
- [ ] Configuration compatibility with existing patterns
- [ ] Monitoring integration with existing telemetry
- [ ] No breaking changes to existing APIs