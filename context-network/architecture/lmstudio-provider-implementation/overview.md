# Architecture Overview: LMStudio Provider Implementation

## High-Level Architecture

### System Context
The LMStudio provider integrates into the existing Patinox provider ecosystem as a local LLM provider, sitting alongside Ollama, OpenAI, Anthropic, and OpenRouter providers.

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   User Code     │───▶│  ModelProvider   │───▶│  LMStudio      │
│                 │    │  Trait Interface │    │  Provider       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                          │
                              ▼                          ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │  Provider Error  │    │  LMStudio API   │
                       │  Handling        │    │  HTTP Client    │
                       └──────────────────┘    └─────────────────┘
                              │                          │
                              ▼                          ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │  Local Provider  │    │  localhost:1234 │
                       │  Service         │    │  LMStudio       │
                       │  Discovery       │    │  Service        │
                       └──────────────────┘    └─────────────────┘
```

### Component Integration

**LMStudio Provider** integrates with:
1. **ModelProvider Trait**: Implements standard provider interface
2. **Error Handling System**: Uses established ProviderError hierarchy  
3. **Service Discovery**: Leverages local provider foundation
4. **HTTP Client**: Uses reqwest with established patterns
5. **Configuration System**: Integrates with cascading config management

### Key Design Principles

**1. Pattern Reuse**: Maximum leverage of established patterns
- TDD methodology from Ollama implementation
- OpenAI API format patterns for request/response
- Local provider integration patterns from service discovery

**2. API Compatibility**: OpenAI-standard compliance
- Uses `/v1/models` and `/v1/chat/completions` endpoints  
- Standard request/response formats
- Compatible with OpenAI client libraries

**3. Error Handling Consistency**: Unified error experience
- Maps HTTP errors to ProviderError types
- Consistent error messages across providers
- Proper error context and recovery suggestions

## Component Architecture

### LMStudioProvider Structure

```rust
pub struct LMStudioProvider {
    /// HTTP client for API requests
    client: reqwest::Client,
    
    /// Base URL for LMStudio API (default: http://localhost:1234)
    base_url: String,
    
    /// Cached model information (for performance optimization)
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
}
```

### Core Methods Implementation

**1. list_models() - Model Discovery**
```rust
async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>>
```
- **Endpoint**: `GET /v1/models`
- **Response Format**: OpenAI-compatible model list
- **Caching**: Populates model_cache for performance
- **Error Handling**: Network and parsing errors → ProviderError

**2. complete() - Text Completion**  
```rust
async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse>
```
- **Endpoint**: `POST /v1/chat/completions`
- **Request Format**: OpenAI-compatible chat completion
- **Response Format**: OpenAI-compatible response
- **Validation**: Input validation with clear error messages

**3. supports_model() - Model Availability**
```rust
async fn supports_model(&self, model: &ModelId) -> bool
```
- **Implementation**: Query cached model list
- **Fallback**: Refresh cache if model not found
- **Performance**: Avoid redundant API calls

**4. model_capabilities() - Model Metadata**
```rust
async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities>
```
- **Implementation**: Extract from cached model information
- **Data Source**: Model metadata from `/v1/models` response
- **Default Values**: Reasonable defaults for missing information

## Integration Architecture

### Service Discovery Integration

```rust
// Service Discovery Flow
Service Discovery ──┐
                   ▼
    ┌─────────────────────────┐
    │   Port 1234 Check       │──┐ Available ──▶ LMStudio Provider
    │   Health Probe          │  │
    └─────────────────────────┘  ▼ Unavailable ──▶ Service Not Found Error
```

**Integration Points**:
- Service discovery foundation handles port scanning
- LMStudio provider focuses on API implementation once service located
- Graceful degradation when service unavailable

### Error Handling Architecture

```rust
// Error Flow Architecture
HTTP Response ──┐
               ▼
    ┌─────────────────────────┐
    │   Status Code Check     │
    └─────────────────────────┘
               │
               ▼
    ┌─────────────────────────┐
    │   Error Mapping         │──┐ 404/500 ──▶ NetworkError
    │   (Following Guide)     │  │ Timeout ──▶ NetworkError  
    └─────────────────────────┘  │ Parse Fail ─▶ ApiError
                                ▼ Validation ─▶ InvalidRequest
              ┌─────────────────────────┐
              │   ProviderError         │
              │   Generation            │
              └─────────────────────────┘
```

### Request/Response Flow

```rust
// API Request Flow
CompletionRequest ──┐
                   ▼
    ┌─────────────────────────┐
    │   Request Validation    │──┐ Invalid ──▶ InvalidRequest Error
    └─────────────────────────┘  │
                                ▼ Valid
    ┌─────────────────────────┐
    │   OpenAI Format         │
    │   Conversion            │
    └─────────────────────────┘
                │
                ▼
    ┌─────────────────────────┐
    │   HTTP Client           │──┐ Network Error ──▶ NetworkError
    │   (reqwest)             │  │ Success
    └─────────────────────────┘  ▼
                      ┌─────────────────────────┐
                      │   Response Processing   │
                      │   & Conversion          │
                      └─────────────────────────┘
                                │
                                ▼
                      ┌─────────────────────────┐
                      │   CompletionResponse    │
                      └─────────────────────────┘
```

## Data Flow Architecture

### Model Listing Flow

```
User Request ──▶ list_models() ──▶ GET /v1/models ──▶ LMStudio Service
                      │                                     │
                      ▼                                     │
                Cache Check ──┐ Hit ──▶ Return Cached      │
                             │                             │
                             ▼ Miss                        │
                      HTTP Request ◀─────────────────────────┘
                             │
                             ▼
                    Response Processing ──▶ Cache Update ──▶ Return Models
```

### Completion Request Flow

```
CompletionRequest ──▶ Validation ──▶ OpenAI Format ──▶ HTTP Request ──▶ LMStudio
                            │             Conversion           │
                            │                                  │
                    InvalidRequest ◀──────────────────────────  │
                         Error                                 │
                                                              ▼
User Response ◀── CompletionResponse ◀── Response Processing ◀─┘
     │                                        │
     │                            NetworkError/ApiError
     ▼                                        │
Success Result                               ▼
                                      Error Response
```

## Performance Architecture

### Caching Strategy

**Model Cache Design**:
- **Cache Key**: Model name/ID
- **Cache Value**: ModelInfo with capabilities
- **Invalidation**: Time-based (configurable TTL)
- **Thread Safety**: Arc<RwLock<HashMap>> for concurrent access

**Performance Optimizations**:
- Cache model list to avoid repeated `/v1/models` calls
- Use cached data for `supports_model()` and `model_capabilities()`
- Lazy cache population on first model list request

### Connection Management

**HTTP Client Configuration**:
- **Timeout**: 30 seconds (configurable)
- **Connection Pooling**: Handled by reqwest
- **Keep-Alive**: Enabled for local connections
- **Retry Logic**: Basic retry for transient failures

## Security Architecture

### Local Service Security

**Security Considerations**:
- **No Authentication**: LMStudio typically runs without auth on localhost
- **Network Exposure**: Only localhost access by default
- **Input Validation**: Validate all user inputs before API calls
- **Error Information**: Sanitize error messages to prevent information leakage

**Security Patterns**:
- Validate endpoint URLs to prevent SSRF
- Sanitize model names and input parameters
- Log security-relevant events (unusual endpoints, large requests)

## Testing Architecture

### Test Organization

```rust
// Test Module Structure
mod lmstudio_provider_tests {
    // Unit Tests (11+ tests)
    mod unit_tests {
        test_provider_creation()
        test_endpoint_validation()
        test_error_mapping()
        test_request_formatting()
        test_response_parsing()
        test_model_caching()
        // ... additional unit tests
    }
    
    // Integration Tests (5+ tests)  
    mod integration_tests {
        #[ignore = "requires running LMStudio service"]
        test_real_service_integration()
        
        #[ignore = "requires LMStudio with models"]
        test_model_listing()
        
        // ... additional integration tests
    }
}
```

### Test Data Management

**Mock Data Strategy**:
- OpenAI-compatible mock responses
- Realistic model information structures
- Error response simulation
- Service unavailable scenarios

## Deployment Architecture

### Configuration Management

**Configuration Sources** (Priority Order):
1. Environment Variables (`LMSTUDIO_ENDPOINT`, `LMSTUDIO_TIMEOUT`)
2. Configuration Files (JSON/TOML)
3. Default Values (localhost:1234, 30s timeout)

**Configuration Structure**:
```rust
pub struct LMStudioConfig {
    pub endpoint: String,           // Default: "http://localhost:1234"
    pub timeout_seconds: u64,       // Default: 30
    pub cache_ttl_minutes: u32,     // Default: 5
    pub max_retries: u32,           // Default: 3
}
```

### Service Discovery Integration

**Discovery Process**:
1. Check configured endpoint first
2. Fall back to service discovery port scanning
3. Validate service availability with health check
4. Cache successful endpoint for session

This architecture provides a comprehensive foundation for implementing the LMStudio provider while maintaining consistency with established patterns and ensuring high-quality integration with the existing Patinox framework.