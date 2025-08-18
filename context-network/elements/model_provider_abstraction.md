# Model and Provider Abstraction

## Purpose
This document defines the model and provider abstraction layer that enables flexible, cascading configuration of LLM providers and models throughout the Patinox framework.

## Classification
- **Domain:** Technical Architecture
- **Stability:** Semi-stable
- **Abstraction:** Detailed
- **Confidence:** High

## Content

### Design Philosophy

The model/provider abstraction follows these principles:

1. **Provider Agnostic**: The framework doesn't care whether you're using OpenRouter, direct OpenAI, Anthropic, or local models
2. **Cascading Overrides**: Configuration cascades from global → agent → request level
3. **Zero Required Config**: Sensible defaults that work out of the box
4. **Model Routing**: Support for router services like OpenRouter that handle provider selection
5. **Capability Awareness**: Different models have different capabilities (vision, tools, context length)

### Core Abstractions

```rust
/// Identifies a specific model, potentially with provider routing
#[derive(Clone, Debug, PartialEq)]
pub struct ModelId {
    /// The model identifier (e.g., "claude-3-opus", "gpt-4", "anthropic/claude-3-opus")
    pub name: String,
    /// Optional explicit provider override
    pub provider_hint: Option<String>,
}

impl ModelId {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            provider_hint: None,
        }
    }
    
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider_hint = Some(provider.into());
        self
    }
}

/// Represents a provider that can route to multiple underlying providers
pub enum Provider {
    /// OpenRouter - routes to multiple providers
    OpenRouter {
        api_key: SecretString,
        base_url: Option<String>,
    },
    /// Direct OpenAI API
    OpenAI {
        api_key: SecretString,
        organization: Option<String>,
        base_url: Option<String>,
    },
    /// Direct Anthropic API
    Anthropic {
        api_key: SecretString,
        base_url: Option<String>,
    },
    /// Local model (Ollama, llama.cpp, etc.)
    Local {
        endpoint: String,
        model_path: Option<PathBuf>,
    },
    /// Custom provider implementation
    Custom(Box<dyn ModelProvider>),
}

/// Core trait that all providers must implement
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Get available models from this provider
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;
    
    /// Create a completion
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError>;
    
    /// Create embeddings
    async fn embed(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, ProviderError>;
    
    /// Check if provider supports a specific model
    async fn supports_model(&self, model: &ModelId) -> bool;
    
    /// Get model capabilities
    async fn model_capabilities(&self, model: &ModelId) -> Option<ModelCapabilities>;
}
```

### Model Capabilities

Different models have different capabilities that the framework needs to be aware of:

```rust
#[derive(Clone, Debug, Default)]
pub struct ModelCapabilities {
    /// Maximum context length in tokens
    pub max_tokens: usize,
    /// Whether the model supports function/tool calling
    pub supports_tools: bool,
    /// Whether the model supports vision/images
    pub supports_vision: bool,
    /// Whether the model supports streaming responses
    pub supports_streaming: bool,
    /// Cost per 1K input tokens (if known)
    pub input_cost_per_1k: Option<f64>,
    /// Cost per 1K output tokens (if known)
    pub output_cost_per_1k: Option<f64>,
    /// Model's speed tier (for routing decisions)
    pub speed_tier: SpeedTier,
    /// Model's quality tier (for routing decisions)
    pub quality_tier: QualityTier,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpeedTier {
    Instant,    // < 500ms typical
    Fast,       // < 2s typical
    Standard,   // < 5s typical
    Slow,       // > 5s typical
}

#[derive(Clone, Debug, PartialEq)]
pub enum QualityTier {
    Lite,       // Fast, cheap, lower quality
    Standard,   // Balanced
    Premium,    // High quality, slower, expensive
    Ultra,      // Best available (GPT-4, Claude Opus)
}
```

### Configuration Hierarchy

Configuration cascades through three levels, with each level overriding the previous:

```rust
/// Global configuration (from environment/config files)
pub struct GlobalModelConfig {
    /// Default provider to use
    pub default_provider: Provider,
    /// Default model to use
    pub default_model: ModelId,
    /// Fallback models if primary fails
    pub fallback_models: Vec<ModelId>,
    /// Model selection strategy
    pub selection_strategy: SelectionStrategy,
    /// Rate limiting configuration
    pub rate_limits: RateLimitConfig,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Agent-level configuration
pub struct AgentModelConfig {
    /// Override provider for this agent
    pub provider: Option<Provider>,
    /// Override model for this agent
    pub model: Option<ModelId>,
    /// Agent-specific temperature
    pub temperature: Option<f32>,
    /// Agent-specific max tokens
    pub max_tokens: Option<usize>,
    /// Required model capabilities
    pub required_capabilities: ModelCapabilities,
}

/// Request-level configuration
pub struct RequestConfig {
    /// Override provider for this request
    pub provider: Option<Provider>,
    /// Override model for this request
    pub model: Option<ModelId>,
    /// Request-specific parameters
    pub parameters: Option<CompletionParameters>,
    /// Timeout for this request
    pub timeout: Option<Duration>,
}
```

### Model Selection Strategy

When using a routing provider like OpenRouter, we can specify strategies:

```rust
pub enum SelectionStrategy {
    /// Always use the specified model
    Fixed,
    /// Choose fastest model that meets requirements
    FastestAvailable,
    /// Choose cheapest model that meets requirements
    CheapestAvailable,
    /// Balance between speed and cost
    Balanced,
    /// Use the highest quality model available
    BestQuality,
    /// Custom selection function
    Custom(Box<dyn Fn(&[ModelInfo]) -> Option<ModelId> + Send + Sync>),
}
```

### Usage Examples

#### Basic Usage (All Defaults)
```rust
// Uses global defaults for everything
let agent = Agent::new();
let response = agent.complete("Hello, world!").await?;
```

#### Override Model Only
```rust
let agent = Agent::new();
let response = agent
    .complete("Explain quantum computing")
    .with_model("claude-3-opus")  // Just change the model
    .await?;
```

#### Override Provider and Model
```rust
let agent = Agent::new();
let response = agent
    .complete("Generate a haiku")
    .with_provider(Provider::OpenRouter { 
        api_key: get_api_key(),
        base_url: None 
    })
    .with_model("anthropic/claude-3-haiku")  // OpenRouter format
    .await?;
```

#### Agent-Level Configuration
```rust
let agent = Agent::builder()
    .with_model("gpt-4-turbo")  // This agent always uses GPT-4
    .with_temperature(0.7)
    .build();

// All requests from this agent use GPT-4 unless overridden
let response = agent.complete("Hello").await?;
```

#### Capability-Based Selection
```rust
let agent = Agent::builder()
    .require_capability(Capability::Vision)
    .require_capability(Capability::LongContext(32000))
    .with_selection_strategy(SelectionStrategy::CheapestAvailable)
    .build();

// Automatically selects cheapest model with vision and 32K+ context
let response = agent.complete_with_image("What's in this image?", image).await?;
```

### Provider Implementations

#### OpenRouter Implementation
```rust
impl ModelProvider for OpenRouterProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // OpenRouter automatically routes to the best provider
        let url = format!("{}/chat/completions", self.base_url);
        
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", self.api_key));
        headers.insert("HTTP-Referer", "https://patinox.dev");  // Optional
        headers.insert("X-Title", "Patinox Agent");  // Optional
        
        // OpenRouter accepts model in format "provider/model" or just "model"
        let body = json!({
            "model": request.model.name,
            "messages": request.messages,
            "temperature": request.temperature,
            // OpenRouter supports provider preferences
            "provider": {
                "order": request.model.provider_hint,
                "require_parameters": true,
            }
        });
        
        // Make request and handle response
        self.client.post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?
            .json()
            .await
    }
    
    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // OpenRouter provides a models endpoint
        let url = format!("{}/models", self.base_url);
        let models = self.client.get(url)
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .json::<OpenRouterModelsResponse>()
            .await?;
            
        Ok(models.into_iter().map(|m| ModelInfo {
            id: ModelId::new(m.id),
            capabilities: ModelCapabilities {
                max_tokens: m.context_length,
                supports_tools: m.supports_function_calling,
                input_cost_per_1k: Some(m.pricing.prompt),
                output_cost_per_1k: Some(m.pricing.completion),
                // ... map other fields
            }
        }).collect())
    }
}
```

### Configuration Loading

Configuration can come from multiple sources with precedence:

```rust
pub struct ModelConfigLoader {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ModelConfigLoader {
    pub async fn load() -> Result<GlobalModelConfig> {
        let mut config = GlobalModelConfig::default();
        
        // 1. Load from config file (lowest precedence)
        if let Ok(file_config) = Self::load_from_file("patinox.toml").await {
            config.merge(file_config);
        }
        
        // 2. Load from environment variables
        if let Ok(env_config) = Self::load_from_env() {
            config.merge(env_config);
        }
        
        // 3. Load from runtime overrides (highest precedence)
        if let Some(override_config) = RUNTIME_OVERRIDES.get() {
            config.merge(override_config);
        }
        
        config.validate()?;
        Ok(config)
    }
}
```

### Example Configuration File

```toml
[model]
# Default provider configuration
default_provider = "openrouter"
default_model = "anthropic/claude-3-sonnet"

# Fallback chain if primary model fails
fallback_models = [
    "openai/gpt-4-turbo",
    "google/gemini-pro",
    "meta-llama/llama-3-70b",
]

# Model selection strategy when using routers
selection_strategy = "balanced"  # or "fastest", "cheapest", "best_quality"

[model.providers.openrouter]
api_key = "${OPENROUTER_API_KEY}"  # Environment variable reference
base_url = "https://openrouter.ai/api/v1"

[model.providers.openai]
api_key = "${OPENAI_API_KEY}"
organization = "${OPENAI_ORG_ID}"

[model.providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"

[model.rate_limits]
requests_per_minute = 60
tokens_per_minute = 100000

[model.retry]
max_attempts = 3
initial_delay_ms = 1000
max_delay_ms = 10000
exponential_base = 2
```

### Environment Variable Configuration

```bash
# Provider selection
export PATINOX_MODEL_PROVIDER=openrouter
export PATINOX_MODEL_DEFAULT=claude-3-opus

# API Keys
export OPENROUTER_API_KEY=sk-or-...
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...

# Advanced configuration
export PATINOX_MODEL_FALLBACKS=gpt-4,claude-3-sonnet,gemini-pro
export PATINOX_MODEL_STRATEGY=balanced
export PATINOX_MODEL_TIMEOUT=30s
```

### Error Handling and Fallbacks

```rust
pub struct ModelExecutor {
    primary: Box<dyn ModelProvider>,
    fallbacks: Vec<Box<dyn ModelProvider>>,
    retry_policy: RetryPolicy,
}

impl ModelExecutor {
    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Try primary provider with retries
        match self.complete_with_retries(&self.primary, &request).await {
            Ok(response) => return Ok(response),
            Err(e) if !e.is_retriable() => return Err(e),
            Err(e) => {
                warn!("Primary provider failed: {}", e);
            }
        }
        
        // Try fallback providers
        for (i, fallback) in self.fallbacks.iter().enumerate() {
            info!("Trying fallback provider {}", i + 1);
            match self.complete_with_retries(fallback, &request).await {
                Ok(response) => return Ok(response),
                Err(e) if !e.is_retriable() => return Err(e),
                Err(e) => {
                    warn!("Fallback {} failed: {}", i + 1, e);
                }
            }
        }
        
        Err(ProviderError::AllProvidersFailed)
    }
}
```

## Benefits of This Design

1. **Flexibility**: Users can use any provider/model combination without code changes
2. **Simplicity**: Zero configuration required for basic usage
3. **Power**: Full control when needed with cascading overrides
4. **Reliability**: Automatic fallbacks and retries
5. **Cost Control**: Can optimize for cost vs. performance
6. **Future Proof**: Easy to add new providers or routing services

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** [elements/configuration_strategy.md]
- **Related Nodes:** 
  - [elements/technology_stack.md] - implements - Provider libraries
  - [elements/monitoring_strategy.md] - monitors - Model performance
  - [foundation/principles.md] - follows - Flexibility principles

## Navigation Guidance
- **Access Context:** Reference when implementing model providers or configuration
- **Common Next Steps:** Review configuration strategy or specific provider implementations
- **Related Tasks:** Provider integration, configuration management, error handling
- **Update Patterns:** Update when adding new providers or selection strategies

## Metadata
- **Created:** 2025-01-17
- **Last Updated:** 2025-01-17
- **Updated By:** Development Team

## Change History
- 2025-01-17: Designed comprehensive model/provider abstraction with cascading configuration