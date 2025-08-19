# Core Trait Signatures Proposal

## Purpose
Define concrete trait signatures for the four core abstractions in Patinox, based on architectural decisions and MVP requirements.

## Classification
- **Domain:** Interface Design
- **Stability:** Semi-stable
- **Abstraction:** Structural
- **Confidence:** Evolving

## Design Principles

Based on our architectural decisions:
- **Object Safety**: All traits must support `Box<dyn Trait>`
- **Async Support**: Use `async_trait` for async methods
- **Error Integration**: Use `PatinoxError` throughout
- **Minimal Typestate**: Focus on key state transitions
- **Tower Compatibility**: Validators implement `Service` trait

## Core Error Types

```rust
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PatinoxError {
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Execution failed: {0}")]
    Execution(#[from] ExecutionError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Anti-jailbreak check failed: {reason}")]
    AntiJailbreak { reason: String },
    
    #[error("Rate limit exceeded: {limit} requests per {window}")]
    RateLimit { limit: u32, window: String },
    
    #[error("Circuit breaker open: {service}")]
    CircuitBreaker { service: String },
    
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Tool execution failed: {tool_name}")]
    ToolFailed { tool_name: String, source: Box<dyn std::error::Error + Send + Sync> },
    
    #[error("LLM provider error: {provider}")]
    ProviderError { provider: String, source: Box<dyn std::error::Error + Send + Sync> },
    
    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },
    
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("HTTP error: {status}")]
    Http { status: u16, body: String },
    
    #[error("Connection failed: {endpoint}")]
    Connection { endpoint: String },
    
    #[error("Serialization error")]
    Serialization(#[source] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },
    
    #[error("Failed to load config from {path}")]
    LoadFailed { path: String, source: Box<dyn std::error::Error + Send + Sync> },
}

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, backoff_ms: u64 },
    Fallback { fallback_action: String },
    CircuitBreak { timeout_ms: u64 },
    Fail,
}

impl PatinoxError {
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Validation(ValidationError::RateLimit { .. }) => {
                RecoveryStrategy::Retry { max_attempts: 3, backoff_ms: 1000 }
            }
            Self::Validation(_) => RecoveryStrategy::Fail,
            Self::Execution(ExecutionError::Timeout { .. }) => {
                RecoveryStrategy::Retry { max_attempts: 2, backoff_ms: 500 }
            }
            Self::Execution(_) => RecoveryStrategy::Fallback { 
                fallback_action: "default_response".to_string() 
            },
            Self::Network(_) => RecoveryStrategy::CircuitBreak { timeout_ms: 30000 },
            Self::Configuration(_) => RecoveryStrategy::Fail,
        }
    }
}
```

## Agent Trait

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core agent abstraction with minimal typestate pattern
/// 
/// This trait defines the minimum interface contract while allowing
/// implementations to be as sophisticated as needed. The state enum
/// covers essential states that all agents need, but implementations
/// can augment with detailed internal states as needed.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent identifier
    fn id(&self) -> Uuid;
    
    /// Current agent state (simplified for compatibility)
    /// 
    /// Implementations with detailed internal states should map
    /// their internal state to one of these core states.
    fn state(&self) -> AgentState;
    
    /// Agent configuration
    fn config(&self) -> &AgentConfig;
    
    /// Start the agent (Created -> Started)
    async fn start(&mut self) -> Result<(), PatinoxError>;
    
    /// Stop the agent (any state -> Stopped)
    async fn stop(&mut self) -> Result<(), PatinoxError>;
    
    /// Execute a request through the agent pipeline
    async fn execute(&mut self, request: AgentRequest) -> Result<AgentResponse, PatinoxError>;
    
    /// Get available tools for this agent
    fn available_tools(&self) -> Vec<String>;
    
    /// Health check for monitoring
    /// 
    /// Implementations can include detailed state information
    /// in the metadata for debugging and monitoring.
    async fn health(&self) -> HealthStatus;
}

/// Core agent states - minimal but extensible
/// 
/// These five states cover the essential lifecycle that all agents share.
/// Implementations can have richer internal state machines that map to these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentState {
    /// Agent exists but hasn't been started
    Created,
    /// Agent is starting up (loading config, connecting to services)
    Started,
    /// Agent is actively processing requests
    Running,
    /// Agent has been stopped and cannot process requests
    Stopped,
    /// Agent encountered an error and may need intervention
    Error { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub description: Option<String>,
    pub max_concurrent_requests: u32,
    pub timeout_ms: u64,
    pub enabled_validators: Vec<String>,
    pub llm_provider: String,
    pub llm_model: String,
    pub tools: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub message: String,
    pub tool_calls: Vec<ToolCall>,
    pub context: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub request_id: Uuid,
    pub message: String,
    pub tool_results: Vec<ToolResult>,
    pub usage: Usage,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}
```

## Tool Trait

```rust
/// Core tool abstraction for agent actions
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool identifier
    fn name(&self) -> &str;
    
    /// Tool description for LLM function calling
    fn description(&self) -> &str;
    
    /// JSON schema for tool parameters
    fn parameters_schema(&self) -> serde_json::Value;
    
    /// Execute the tool with given parameters
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, PatinoxError>;
    
    /// Tool metadata for discovery and categorization
    fn metadata(&self) -> ToolMetadata;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub call_id: String,
    pub parameters: serde_json::Value,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub category: String,
    pub tags: Vec<String>,
    pub version: String,
    pub author: Option<String>,
    pub dangerous: bool, // Requires extra validation
}
```

## Validator Trait (Tower Service)

```rust
use tower::{Service, ServiceExt};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Validator that implements Tower Service for composability
pub trait Validator: Service<ValidationRequest, Response = ValidationResponse, Error = PatinoxError> + Send + Sync {
    /// Validator name for identification
    fn name(&self) -> &str;
    
    /// Validator configuration
    fn config(&self) -> &ValidatorConfig;
    
    /// Whether this validator should run for the given request
    fn should_validate(&self, request: &ValidationRequest) -> bool;
}

#[derive(Debug, Clone)]
pub struct ValidationRequest {
    pub agent_id: Uuid,
    pub request_id: Uuid,
    pub stage: ValidationStage,
    pub content: ValidationContent,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum ValidationStage {
    PreExecution,   // Before LLM call
    PostExecution,  // After LLM call, before tool execution
    PostTool,       // After tool execution
    PreResponse,    // Before sending response to user
}

#[derive(Debug, Clone)]
pub enum ValidationContent {
    UserMessage { message: String },
    LlmResponse { message: String, tool_calls: Vec<ToolCall> },
    ToolResult { tool_name: String, result: ToolResult },
    FinalResponse { message: String },
}

#[derive(Debug, Clone)]
pub struct ValidationResponse {
    pub approved: bool,
    pub reason: Option<String>,
    pub modifications: Option<ValidationModifications>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ValidationModifications {
    pub modified_content: String,
    pub blocked_tool_calls: Vec<String>,
    pub added_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub name: String,
    pub enabled: bool,
    pub priority: i32, // Lower numbers run first
    pub stages: Vec<ValidationStage>,
    pub parameters: HashMap<String, serde_json::Value>,
}
```

## Monitor Trait

```rust
/// Asynchronous monitoring for agent behavior analysis
#[async_trait]
pub trait Monitor: Send + Sync {
    /// Monitor identifier
    fn name(&self) -> &str;
    
    /// Start monitoring an agent execution
    async fn start_monitoring(&self, execution_id: Uuid, agent_id: Uuid) -> Result<(), PatinoxError>;
    
    /// Record an event during execution
    async fn record_event(&self, event: MonitorEvent) -> Result<(), PatinoxError>;
    
    /// Complete monitoring for an execution
    async fn complete_monitoring(&self, execution_id: Uuid, summary: ExecutionSummary) -> Result<(), PatinoxError>;
    
    /// Query monitoring data for analysis
    async fn query_events(&self, query: MonitorQuery) -> Result<Vec<MonitorEvent>, PatinoxError>;
    
    /// Get monitoring configuration
    fn config(&self) -> &MonitorConfig;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: MonitorEventType,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorEventType {
    ExecutionStarted,
    ValidationPassed { validator: String },
    ValidationFailed { validator: String, reason: String },
    ToolExecuted { tool: String, duration_ms: u64 },
    LlmCalled { provider: String, model: String, tokens: Usage },
    ErrorOccurred { error_type: String, recoverable: bool },
    ExecutionCompleted { success: bool, total_duration_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub success: bool,
    pub total_duration_ms: u64,
    pub llm_calls: u32,
    pub tool_calls: u32,
    pub validation_failures: u32,
    pub total_tokens: Usage,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorQuery {
    pub agent_ids: Option<Vec<Uuid>>,
    pub event_types: Option<Vec<MonitorEventType>>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub name: String,
    pub enabled: bool,
    pub buffer_size: u32,
    pub flush_interval_ms: u64,
    pub sampling_rate: f64, // 0.0 to 1.0
    pub event_types: Vec<MonitorEventType>,
}
```

## Supporting Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost_usd: Option<f64>,
}

/// Builder pattern for agent configuration
pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: AgentConfig {
                name: name.into(),
                description: None,
                max_concurrent_requests: 10,
                timeout_ms: 30000,
                enabled_validators: vec!["anti-jailbreak".to_string()],
                llm_provider: "openai".to_string(),
                llm_model: "gpt-4".to_string(),
                tools: vec![],
                metadata: HashMap::new(),
            },
        }
    }
    
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.config.description = Some(desc.into());
        self
    }
    
    pub fn max_concurrent_requests(mut self, max: u32) -> Self {
        self.config.max_concurrent_requests = max;
        self
    }
    
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.config.timeout_ms = timeout;
        self
    }
    
    pub fn add_validator(mut self, validator: impl Into<String>) -> Self {
        self.config.enabled_validators.push(validator.into());
        self
    }
    
    pub fn add_tool(mut self, tool: impl Into<String>) -> Self {
        self.config.tools.push(tool.into());
        self
    }
    
    pub fn llm_provider(mut self, provider: impl Into<String>) -> Self {
        self.config.llm_provider = provider.into();
        self
    }
    
    pub fn llm_model(mut self, model: impl Into<String>) -> Self {
        self.config.llm_model = model.into();
        self
    }
    
    pub fn build(self) -> AgentConfig {
        self.config
    }
}
```

## Usage Examples

```rust
// Agent creation with builder pattern
let config = AgentBuilder::new("customer-support")
    .description("Customer support agent with tool access")
    .add_tool("search-kb")
    .add_tool("create-ticket")
    .add_validator("anti-jailbreak")
    .add_validator("rate-limit")
    .llm_model("gpt-4")
    .max_concurrent_requests(5)
    .build();

// Tool trait object
let tools: Vec<Box<dyn Tool>> = vec![
    Box::new(SearchKnowledgeBase::new()),
    Box::new(CreateTicket::new()),
];

// Validator middleware stack
let validator_stack = ServiceBuilder::new()
    .layer(AntiJailbreakValidator::new())
    .layer(RateLimitValidator::new(100, Duration::from_secs(60)))
    .layer(CircuitBreakerValidator::new("llm-provider", 0.5));

// Monitor setup
let monitor: Box<dyn Monitor> = Box::new(TelemetryMonitor::new());
```

## Object Safety Verification

All traits are designed to be object-safe:
- No associated types with `Self` constraints
- No generic methods
- No `Self` in return types (except for builders)
- All methods use `&self` or `&mut self`
- `Send + Sync` bounds for thread safety

## Integration Points

- **Agent ↔ Tool**: Agent calls tools through the `Tool` trait
- **Agent ↔ Validator**: Requests pass through Tower middleware stack
- **Agent ↔ Monitor**: Agent notifies monitor of execution events
- **Error Handling**: All traits use `PatinoxError` with recovery strategies
- **Async Support**: All I/O operations are async with `async_trait`

## State Augmentation Strategies

The core `AgentState` enum is intentionally minimal to ensure compatibility and simplicity. However, implementations can augment state tracking in several ways without breaking the trait contract:

### 1. Enum Extension with Mapping
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetailedAgentState {
    Created,
    Configuring,
    ValidatingConfig,
    Started,
    Initializing,
    LoadingTools,
    ConnectingToProviders,
    Running,
    Processing { request_id: Uuid },
    WaitingForTool { tool_name: String },
    WaitingForLlm { provider: String },
    Pausing,
    Stopped,
    Error { reason: String },
}

impl From<DetailedAgentState> for AgentState {
    fn from(detailed: DetailedAgentState) -> Self {
        match detailed {
            DetailedAgentState::Created => AgentState::Created,
            DetailedAgentState::Configuring | 
            DetailedAgentState::ValidatingConfig => AgentState::Created,
            DetailedAgentState::Started | 
            DetailedAgentState::Initializing | 
            DetailedAgentState::LoadingTools | 
            DetailedAgentState::ConnectingToProviders => AgentState::Started,
            DetailedAgentState::Running | 
            DetailedAgentState::Processing { .. } | 
            DetailedAgentState::WaitingForTool { .. } | 
            DetailedAgentState::WaitingForLlm { .. } => AgentState::Running,
            DetailedAgentState::Pausing => AgentState::Running,
            DetailedAgentState::Stopped => AgentState::Stopped,
            DetailedAgentState::Error { reason } => AgentState::Error { reason },
        }
    }
}
```

### 2. Optional Extension Traits
```rust
/// Optional trait for agents that need detailed state reporting
pub trait DetailedAgent: Agent {
    type DetailedState: Clone + fmt::Debug;
    
    fn detailed_state(&self) -> &Self::DetailedState;
    fn state_history(&self) -> Vec<(chrono::DateTime<chrono::Utc>, Self::DetailedState)>;
    fn state_transitions(&self) -> u64;
}

/// Optional trait for agents with state machine validation
pub trait StateMachine: Agent {
    fn valid_transitions(&self) -> Vec<(AgentState, AgentState)>;
    fn can_transition_to(&self, target: AgentState) -> bool;
    fn transition_to(&mut self, target: AgentState) -> Result<(), PatinoxError>;
}
```

### 3. Metadata-Rich Health Status
```rust
impl Agent for MyDetailedAgent {
    async fn health(&self) -> HealthStatus {
        HealthStatus::Healthy {
            metadata: hashmap! {
                "internal_state".to_string() => format!("{:?}", self.detailed_state),
                "active_requests".to_string() => self.active_requests.len().to_string(),
                "last_llm_call".to_string() => self.last_llm_call.map(|t| t.to_rfc3339()).unwrap_or_default(),
                "tool_queue_depth".to_string() => self.tool_queue.len().to_string(),
            }
        }
    }
}
```

### 4. Monitoring Integration for State Changes
```rust
impl MyDetailedAgent {
    async fn transition_internal_state(&mut self, new_state: DetailedAgentState) -> Result<(), PatinoxError> {
        let old_state = self.detailed_state.clone();
        self.detailed_state = new_state.clone();
        
        // Record detailed state transition for monitoring
        if let Some(monitor) = &self.monitor {
            let event = MonitorEvent {
                id: Uuid::new_v4(),
                execution_id: self.current_execution_id,
                agent_id: self.id(),
                timestamp: chrono::Utc::now(),
                event_type: MonitorEventType::StateTransition {
                    from: format!("{:?}", old_state),
                    to: format!("{:?}", new_state),
                },
                data: serde_json::json!({
                    "detailed_from": old_state,
                    "detailed_to": new_state,
                    "core_state": AgentState::from(new_state.clone()),
                }),
                metadata: HashMap::new(),
            };
            monitor.record_event(event).await?;
        }
        
        Ok(())
    }
}
```

### 5. Configuration-Driven State Complexity
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    pub enable_detailed_states: bool,
    pub track_state_history: bool,
    pub max_history_entries: usize,
    pub custom_states: Vec<String>,
}
```

This approach ensures that:
- **Simple agents** use the 5 core states with zero overhead
- **Complex agents** can have dozens of internal states while maintaining compatibility
- **Monitoring systems** can capture all state transitions regardless of complexity
- **Debugging tools** get rich state information when available
- **API compatibility** is maintained as agents evolve

## Next Steps

1. Review trait signatures and method names
2. Validate object safety with compilation tests  
3. Create mock implementations for testing
4. Design concrete implementations for MVP
5. Write comprehensive trait documentation with examples

## Relationships
- **Parent Nodes:** [decisions/architectural_decisions_resolved.md]
- **Implements:** All architectural decisions about traits and patterns
- **Enables:** Core implementation work in patinox-core crate
- **Related Nodes:**
  - [elements/architecture_overview.md] - provides context
  - [planning/groomed_foundational_backlog.md] - implementation sequence

## Metadata
- **Created:** 2025-01-19
- **Last Updated:** 2025-01-19
- **Updated By:** Development Team
- **Status:** PROPOSAL - Awaiting Review

## Change History
- 2025-01-19: Initial trait signature proposal based on architectural decisions