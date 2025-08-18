# Technology Stack

## Purpose
This document details the production-proven Rust libraries and tools that form the foundation of Patinox, including justifications for each choice and integration strategies.

## Classification
- **Domain:** Technical Architecture
- **Stability:** Semi-stable
- **Abstraction:** Detailed
- **Confidence:** Established

## Content

### Core Dependencies

#### Async Runtime
**Tokio** (v1.40+)
- **Purpose**: Async runtime for concurrent execution
- **Why**: Industry standard, excellent performance, comprehensive ecosystem
- **Usage**: All async operations, work-stealing scheduler, timer management
- **Downloads**: 200M+ (most used Rust async runtime)

#### LLM Integration
**async-openai** (v0.24+)
- **Purpose**: OpenAI API client with async support
- **Why**: 1.1M+ downloads, streaming support, function calling
- **Usage**: Direct OpenAI provider implementation
- **Features**: Streaming, embeddings, vision, assistants API

**OpenRouter Client** (Custom implementation)
- **Purpose**: Universal LLM routing service integration
- **Why**: Access to 100+ models through single API, automatic failover
- **Usage**: Primary provider for model flexibility
- **Features**: Provider routing, cost optimization, fallback handling
- **Models**: Claude, GPT-4, Gemini, Llama, Mistral, and more

**Rig** (v0.3+)
- **Purpose**: High-level LLM application framework
- **Why**: Unified interface across providers, RAG support built-in
- **Usage**: Provider abstraction patterns, prompt management
- **Backed by**: Dria, Linera Protocol

**Additional Provider Support**:
- **Anthropic SDK**: Direct Claude API access when needed
- **Google AI SDK**: Gemini model integration
- **Ollama Client**: Local model support
- **Custom Providers**: Extensible trait-based system

#### Validation Framework
**Tower** (v0.5+)
- **Purpose**: Middleware framework for composable services
- **Why**: Battle-tested in production HTTP services, elegant composition
- **Usage**: Validation pipeline, circuit breakers, rate limiting
- **Pattern**: Same as Axum/Hyper web frameworks

#### Observability
**OpenTelemetry** (v0.27+)
- **Purpose**: Distributed tracing, metrics, and logging
- **Why**: Industry standard, vendor-agnostic, minimal overhead
- **Usage**: Full observability stack
- **Exporters**: OTLP, Jaeger, Prometheus, Datadog

**tracing** (v0.1+)
- **Purpose**: Structured, contextual logging
- **Why**: Tokio ecosystem standard, compile-time filtering
- **Usage**: Application-level logging and diagnostics
- **Integration**: tracing-opentelemetry bridge

#### Vector Databases
**Qdrant Client** (v1.12+)
- **Purpose**: Vector similarity search
- **Why**: Rust-native, excellent performance, advanced filtering
- **Usage**: Semantic memory, RAG retrieval
- **Features**: Quantization, distributed mode, hybrid search

**LanceDB** (v0.13+)
- **Purpose**: Embedded vector database
- **Why**: Zero-deployment, multi-modal support, columnar storage
- **Usage**: Local development, edge deployment
- **Features**: SQL interface, automatic versioning

### Supporting Libraries

#### Serialization
**serde** (v1.0+)
- **Purpose**: Serialization/deserialization framework
- **Why**: De facto Rust standard, extensive format support
- **Usage**: JSON, TOML, MessagePack, bincode
- **Downloads**: 400M+ (most downloaded Rust crate)

#### Error Handling
**thiserror** (v2.0+)
- **Purpose**: Error type derivation
- **Why**: Clean error definitions, automatic trait implementations
- **Usage**: All custom error types

**anyhow** (v1.0+)
- **Purpose**: Flexible error handling in applications
- **Why**: Excellent for application-level errors
- **Usage**: Binary targets, examples

#### HTTP/Web
**reqwest** (v0.12+)
- **Purpose**: HTTP client
- **Why**: Async support, connection pooling, built on hyper
- **Usage**: External API calls, webhooks

**axum** (v0.7+)
- **Purpose**: Web framework (for meta-layer API)
- **Why**: Tower ecosystem, excellent performance, type-safe
- **Usage**: Admin API, telemetry endpoints

#### Testing
**criterion** (v0.5+)
- **Purpose**: Statistical benchmarking
- **Why**: Rigorous performance measurement, regression detection
- **Usage**: Performance validation, optimization verification

**proptest** (v1.6+)
- **Purpose**: Property-based testing
- **Why**: Find edge cases automatically
- **Usage**: Validator testing, state machine verification

**mockito** (v1.6+)
- **Purpose**: HTTP mocking
- **Why**: Simple API mocking for tests
- **Usage**: LLM API testing, external service mocking

### Development Tools

#### Build & CI
**cargo** (built-in)
- Workspace management
- Feature flags
- Cross-compilation

**cargo-release** (v0.25+)
- Automated release process
- Version bumping
- Tag creation

**cargo-audit** (v0.21+)
- Security vulnerability scanning
- CVE database checks

#### Code Quality
**clippy** (built-in)
- Rust linting
- Best practices enforcement

**rustfmt** (built-in)
- Code formatting
- Style consistency

**cargo-tarpaulin** (v0.31+)
- Code coverage reporting
- CI integration

### Infrastructure Tools

#### Documentation
**rustdoc** (built-in)
- API documentation
- Tested code examples

**mdbook** (v0.4+)
- User guide generation
- Tutorials and examples

#### Deployment
**cross** (v0.2+)
- Cross-compilation toolchain
- Multi-architecture builds

**wasm-pack** (v0.13+)
- WebAssembly packaging
- JavaScript bindings generation

**maturin** (v1.8+)
- Python bindings packaging
- PyPI publishing

### Version Management Strategy

```toml
[workspace.dependencies]
# Core async runtime
tokio = { version = "1.40", features = ["full"] }

# LLM integration
async-openai = "0.24"
rig = "0.3"

# Validation and middleware
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.5", features = ["trace", "compression"] }

# Observability
opentelemetry = { version = "0.27", features = ["trace", "metrics"] }
opentelemetry-otlp = "0.27"
tracing = "0.1"
tracing-opentelemetry = "0.27"

# Vector databases
qdrant-client = "1.12"
lancedb = "0.13"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Testing
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.6"
mockito = "1.6"
```

### Integration Patterns

#### LLM Provider Abstraction
```rust
use async_trait::async_trait;

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, Error>;
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, Error>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error>;
    async fn supports_model(&self, model: &ModelId) -> bool;
}

// Provider implementations with cascading configuration
pub struct OpenRouterProvider { /* Universal router */ }
pub struct OpenAIProvider { /* Direct OpenAI */ }
pub struct AnthropicProvider { /* Direct Anthropic */ }
pub struct OllamaProvider { /* Local models */ }

// Cascading configuration example
let response = agent
    .complete("Hello")  // Uses defaults
    .with_model("claude-3-opus")  // Override model only
    .with_provider(Provider::OpenRouter)  // Override provider too
    .await?;
```

#### Tower Middleware Stack
```rust
use tower::ServiceBuilder;

let service = ServiceBuilder::new()
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new())
    .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
    .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .layer(CircuitBreakerLayer::new(0.5, Duration::from_secs(60)))
    .service(agent_service);
```

### Dependency Governance

**Selection Criteria**:
1. Production usage (downloads, stars, corporate backing)
2. Active maintenance (recent commits, responsive issues)
3. Security track record (CVE history, audit status)
4. API stability (version policy, breaking changes)
5. Performance characteristics (benchmarks, overhead)

**Update Policy**:
- Security updates: Immediate
- Minor versions: Monthly review
- Major versions: Quarterly evaluation
- Breaking changes: With major Patinox versions only

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [foundation/structure.md] - implements - Crate structure
  - [elements/rust_patterns.md] - uses - Pattern implementations
  - [planning/roadmap.md] - schedules - Integration timeline

## Navigation Guidance
- **Access Context:** Reference when adding dependencies or evaluating alternatives
- **Common Next Steps:** Review integration patterns or version management
- **Related Tasks:** Dependency updates, security audits, performance optimization
- **Update Patterns:** Update when adding/removing dependencies or major version changes

## Metadata
- **Created:** 2025-01-17
- **Last Updated:** 2025-01-17
- **Updated By:** Development Team

## Change History
- 2025-01-17: Initial technology stack with core dependencies and integration patterns