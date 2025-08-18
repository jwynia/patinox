# Rust Patterns

## Purpose
This document captures Rust-specific patterns and safety mechanisms that Patinox leverages to create compile-time guarantees, prevent entire classes of errors, and enable zero-cost abstractions for monitoring and validation.

## Classification
- **Domain:** Technical Architecture
- **Stability:** Semi-stable
- **Abstraction:** Detailed
- **Confidence:** Established

## Content

### Typestate Pattern for Agent States

The typestate pattern encodes workflow states as distinct types, making invalid agent state transitions unrepresentable at compile time:

```rust
use std::marker::PhantomData;

// State types (zero-sized at runtime)
struct Initialized;
struct Validated;
struct Executing;
struct Completed;

struct Agent<S> {
    config: AgentConfig,
    telemetry: TelemetryCollector,
    _state: PhantomData<S>,
}

// Only available in Initialized state
impl Agent<Initialized> {
    pub fn validate(self) -> Result<Agent<Validated>, ValidationError> {
        // Validation logic here
        // Consumes self, returns new state
        Ok(Agent {
            config: self.config,
            telemetry: self.telemetry,
            _state: PhantomData,
        })
    }
}

// Only available in Validated state
impl Agent<Validated> {
    pub fn execute(self) -> Agent<Executing> {
        // Can only execute after validation
        Agent {
            config: self.config,
            telemetry: self.telemetry.start_execution(),
            _state: PhantomData,
        }
    }
}

// Impossible to call execute() on unvalidated agent - won't compile!
```

### Builder Pattern with Phantom Types

Ensures complete configuration before agent creation:

```rust
use std::marker::PhantomData;

// Marker types for builder state
struct NoModel;
struct HasModel;
struct NoTools;
struct HasTools;

struct AgentBuilder<M, T> {
    model: Option<ModelConfig>,
    tools: Option<Vec<Tool>>,
    _model: PhantomData<M>,
    _tools: PhantomData<T>,
}

impl AgentBuilder<NoModel, NoTools> {
    pub fn new() -> Self {
        AgentBuilder {
            model: None,
            tools: None,
            _model: PhantomData,
            _tools: PhantomData,
        }
    }
}

impl<T> AgentBuilder<NoModel, T> {
    pub fn with_model(self, model: ModelConfig) -> AgentBuilder<HasModel, T> {
        AgentBuilder {
            model: Some(model),
            tools: self.tools,
            _model: PhantomData,
            _tools: self._tools,
        }
    }
}

impl<M> AgentBuilder<M, NoTools> {
    pub fn with_tools(self, tools: Vec<Tool>) -> AgentBuilder<M, HasTools> {
        AgentBuilder {
            model: self.model,
            tools: Some(tools),
            _model: self._model,
            _tools: PhantomData,
        }
    }
}

// build() only available when fully configured
impl AgentBuilder<HasModel, HasTools> {
    pub fn build(self) -> Agent<Initialized> {
        Agent::new(
            self.model.unwrap(),
            self.tools.unwrap(),
        )
    }
}

// Won't compile - can't build without model and tools!
// let agent = AgentBuilder::new().build();

// Must configure completely:
let agent = AgentBuilder::new()
    .with_model(model_config)
    .with_tools(vec![tool1, tool2])
    .build();
```

### Tower Middleware Pattern for Validation

Composable validation layers using Tower's Service trait:

```rust
use tower::{Service, ServiceBuilder, Layer};
use std::task::{Context, Poll};

// Custom validation layer
pub struct ValidationLayer {
    validators: Vec<Box<dyn Validator>>,
}

impl<S> Layer<S> for ValidationLayer {
    type Service = ValidationService<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        ValidationService {
            inner,
            validators: self.validators.clone(),
        }
    }
}

pub struct ValidationService<S> {
    inner: S,
    validators: Vec<Box<dyn Validator>>,
}

impl<S> Service<AgentRequest> for ValidationService<S>
where
    S: Service<AgentRequest>,
{
    type Response = S::Response;
    type Error = ValidationError;
    type Future = ValidatedFuture<S::Future>;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|_| ValidationError::ServiceNotReady)
    }
    
    fn call(&mut self, req: AgentRequest) -> Self::Future {
        // Run synchronous validators
        for validator in &self.validators {
            if let Err(e) = validator.validate(&req) {
                return ValidatedFuture::rejected(e);
            }
        }
        
        // Pass to inner service
        ValidatedFuture::accepted(self.inner.call(req))
    }
}

// Stack multiple layers
let service = ServiceBuilder::new()
    .layer(MonitorLayer::new(metrics))
    .layer(ValidationLayer::new(validators))
    .layer(CircuitBreakerLayer::new(threshold))
    .layer(RateLimitLayer::new(limit))
    .service(agent_executor);
```

### Const Generics for Zero-Cost Configuration

Compile-time configuration with no runtime overhead:

```rust
// Monitoring levels as const values
pub struct MonitoringConfig<const LEVEL: u8> {
    collectors: Vec<Box<dyn Collector>>,
}

impl<const LEVEL: u8> MonitoringConfig<LEVEL> {
    pub const fn is_enabled() -> bool {
        LEVEL > 0
    }
}

pub struct Agent<const MONITORING: u8 = 0> {
    config: AgentConfig,
    monitoring: MonitoringConfig<MONITORING>,
}

impl<const MONITORING: u8> Agent<MONITORING> {
    pub async fn execute(&self, prompt: &str) -> Result<Response> {
        // Compile-time branch elimination
        if MonitoringConfig::<MONITORING>::is_enabled() {
            self.monitoring.start_span("execute");
        }
        
        let result = self.inner_execute(prompt).await;
        
        if MonitoringConfig::<MONITORING>::is_enabled() {
            self.monitoring.end_span("execute");
        }
        
        result
    }
}

// Zero overhead when disabled
type ProductionAgent = Agent<0>;  // No monitoring code included

// Full monitoring in development
type DevelopmentAgent = Agent<2>; // All monitoring active
```

### Actor Model with Actix

Message-passing concurrency without shared mutable state:

```rust
use actix::prelude::*;

// Messages agents can receive
#[derive(Message)]
#[rtype(result = "Result<Response, AgentError>")]
struct ExecuteTask {
    prompt: String,
    context: Context,
}

#[derive(Message)]
#[rtype(result = "()")]
struct UpdateConfig {
    config: AgentConfig,
}

// Agent as an actor
struct AgentActor {
    config: AgentConfig,
    state: AgentState,
    monitors: Vec<Addr<MonitorActor>>,
}

impl Actor for AgentActor {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        // Initialize on startup
        self.register_with_supervisor(ctx);
    }
}

impl Handler<ExecuteTask> for AgentActor {
    type Result = ResponseFuture<Result<Response, AgentError>>;
    
    fn handle(&mut self, msg: ExecuteTask, ctx: &mut Context<Self>) -> Self::Result {
        // No shared state - each agent is isolated
        let monitors = self.monitors.clone();
        
        Box::pin(async move {
            // Notify monitors asynchronously
            for monitor in monitors {
                monitor.do_send(TaskStarted { task_id });
            }
            
            // Execute task
            let result = execute_with_llm(msg.prompt).await;
            
            // Update monitors
            for monitor in monitors {
                monitor.do_send(TaskCompleted { task_id, result: result.clone() });
            }
            
            result
        })
    }
}

// Supervision hierarchy
struct SupervisorActor {
    agents: Vec<Addr<AgentActor>>,
}

impl Supervisor for SupervisorActor {
    fn restarting(&mut self, addr: &Addr<AgentActor>) {
        // Handle agent restart
        log::warn!("Agent {:?} crashed, restarting", addr);
    }
}
```

### Zero-Copy Integration with Vector Databases

Efficient memory usage through borrowing and direct serialization:

```rust
use bytes::Bytes;
use qdrant_client::prelude::*;

// Zero-copy embedding wrapper
pub struct Embedding<'a> {
    vector: &'a [f32],
    metadata: &'a serde_json::Value,
}

impl<'a> Embedding<'a> {
    // Borrows data, no allocation
    pub fn new(vector: &'a [f32], metadata: &'a serde_json::Value) -> Self {
        Self { vector, metadata }
    }
    
    // Direct serialization to Qdrant format
    pub fn to_point(&self, id: PointId) -> Point {
        Point {
            id: Some(id),
            vectors: Some(self.vector.into()),
            payload: self.metadata.clone().into(),
        }
    }
}

// Streaming results without collecting
pub async fn semantic_search(
    query: &[f32],
    limit: usize,
) -> impl Stream<Item = Result<SearchResult, QdrantError>> {
    let client = get_qdrant_client();
    
    // Returns async stream, not Vec
    client
        .search_stream(&SearchRequest {
            vector: query.to_vec(),
            limit,
            with_payload: Some(true.into()),
        })
        .await
}
```

### Circuit Breaker Pattern

Type-safe circuit breaker with compile-time state tracking:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

#[repr(u8)]
enum CircuitState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

pub struct CircuitBreaker {
    state: Arc<AtomicU8>,
    failure_count: Arc<AtomicU64>,
    threshold: u64,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error>>>,
    {
        match self.state.load(Ordering::Acquire) {
            0 => self.try_call(f).await,  // Closed
            1 => Err(CircuitBreakerError::Open),  // Open
            2 => self.test_call(f).await,  // HalfOpen
            _ => unreachable!(),
        }
    }
    
    async fn try_call<F, Fut, T>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error>>>,
    {
        match f().await {
            Ok(result) => {
                self.failure_count.store(0, Ordering::Release);
                Ok(result)
            }
            Err(e) => {
                let failures = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
                if failures >= self.threshold {
                    self.open_circuit();
                }
                Err(CircuitBreakerError::CallFailed(e))
            }
        }
    }
}
```

### Validation Pipeline with Compile-Time Configuration

```rust
pub struct ValidationPipeline<const STRICT: bool = false> {
    sync_validators: Vec<Box<dyn SyncValidator>>,
    async_monitors: Arc<RwLock<Vec<Box<dyn AsyncMonitor>>>>,
}

impl<const STRICT: bool> ValidationPipeline<STRICT> {
    pub async fn validate(&self, action: &AgentAction) -> ValidationResult {
        // Synchronous validation (blocking)
        for validator in &self.sync_validators {
            match validator.validate(action) {
                Ok(_) => continue,
                Err(e) if STRICT => return ValidationResult::Rejected(e),
                Err(e) => log::warn!("Validation warning: {}", e),
            }
        }
        
        // Asynchronous monitoring (non-blocking)
        let monitors = self.async_monitors.read().await;
        for monitor in monitors.iter() {
            tokio::spawn({
                let monitor = monitor.clone();
                let action = action.clone();
                async move {
                    if let Err(e) = monitor.analyze(action).await {
                        log::error!("Monitor error: {}", e);
                    }
                }
            });
        }
        
        ValidationResult::Approved
    }
}

// Strict validation in production
type ProductionPipeline = ValidationPipeline<true>;

// Permissive in development
type DevelopmentPipeline = ValidationPipeline<false>;
```

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [elements/technology_stack.md] - implements - Uses these libraries
  - [foundation/principles.md] - embodies - Safety principles
  - [elements/monitoring_strategy.md] - enables - Zero-cost monitoring

## Navigation Guidance
- **Access Context:** Reference when implementing Rust-specific features or patterns
- **Common Next Steps:** Review architecture overview or monitoring strategy
- **Related Tasks:** Pattern implementation, safety verification, performance optimization
- **Update Patterns:** Update when discovering new patterns or best practices

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Created comprehensive Rust patterns document based on research findings