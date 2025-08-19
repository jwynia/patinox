//! Agent trait definition and supporting types
//!
//! This module defines the core Agent trait that all agent implementations
//! must implement. The trait is designed to be object-safe and supports
//! async execution with proper error handling.

use crate::error::PatinoxError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Tests are written FIRST to define the contract
#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing (minimal, just to make tests compile)
    struct TestAgent {
        id: Uuid,
        state: AgentState,
        config: AgentConfig,
    }

    #[async_trait]
    impl Agent for TestAgent {
        fn id(&self) -> Uuid {
            self.id
        }

        fn state(&self) -> AgentState {
            self.state.clone()
        }

        fn config(&self) -> &AgentConfig {
            &self.config
        }

        async fn start(&mut self) -> Result<(), PatinoxError> {
            self.state = AgentState::Running;
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), PatinoxError> {
            self.state = AgentState::Stopped;
            Ok(())
        }

        async fn execute(&mut self, request: AgentRequest) -> Result<AgentResponse, PatinoxError> {
            if !matches!(self.state, AgentState::Running) {
                return Err(PatinoxError::Execution(
                    crate::error::ExecutionError::AgentStateMismatch(
                        "Running".to_string(),
                        format!("{:?}", self.state),
                    )
                ));
            }

            Ok(AgentResponse {
                request_id: request.id,
                message: format!("Processed: {}", request.message),
                tool_results: vec![],
                usage: crate::traits::Usage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                    cost_usd: Some(0.001),
                },
                metadata: HashMap::new(),
            })
        }

        fn available_tools(&self) -> Vec<String> {
            vec!["mock-tool".to_string()]
        }

        async fn health(&self) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    impl TestAgent {
        fn new(name: &str) -> Self {
            Self {
                id: Uuid::new_v4(),
                state: AgentState::Created,
                config: AgentConfig {
                    name: name.to_string(),
                    description: Some("Test agent".to_string()),
                    max_concurrent_requests: 5,
                    timeout_ms: 30000,
                    enabled_validators: vec!["test-validator".to_string()],
                    llm_provider: "test".to_string(),
                    llm_model: "test-model".to_string(),
                    tools: vec!["mock-tool".to_string()],
                    metadata: HashMap::new(),
                },
            }
        }
    }

    #[test]
    fn agent_state_enum_completeness() {
        // Test all variants exist and are accessible
        let states = vec![
            AgentState::Created,
            AgentState::Started,
            AgentState::Running,
            AgentState::Stopped,
            AgentState::Error { reason: "test".to_string() },
        ];

        for state in states {
            // Should be able to debug print
            let _debug = format!("{:?}", state);
            // Should be able to clone
            let _cloned = state.clone();
        }
    }

    #[test]
    fn agent_state_equality() {
        let state1 = AgentState::Running;
        let state2 = AgentState::Running;
        assert_eq!(state1, state2);

        let error1 = AgentState::Error { reason: "test".to_string() };
        let error2 = AgentState::Error { reason: "test".to_string() };
        assert_eq!(error1, error2);

        let error3 = AgentState::Error { reason: "different".to_string() };
        assert_ne!(error1, error3);
    }

    #[test]
    fn agent_config_serialization() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test description".to_string()),
            max_concurrent_requests: 10,
            timeout_ms: 5000,
            enabled_validators: vec!["validator1".to_string(), "validator2".to_string()],
            llm_provider: "openai".to_string(),
            llm_model: "gpt-4".to_string(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), "value1".to_string());
                map
            },
        };

        // Test serialization produces valid JSON
        let serialized = serde_json::to_string(&config).expect("Should serialize");
        assert!(!serialized.is_empty(), "Serialized output should not be empty");
        
        // Verify serialized JSON contains expected structure
        let parsed: serde_json::Value = serde_json::from_str(&serialized).expect("Should be valid JSON");
        assert!(parsed.is_object(), "Root should be JSON object");
        assert!(parsed.get("name").is_some(), "Should include name field");
        assert!(parsed.get("max_concurrent_requests").is_some(), "Should include request limit");

        // Test deserialization preserves data integrity
        let deserialized: AgentConfig = serde_json::from_str(&serialized).expect("Should deserialize");
        
        // Validate deserialized data meets business constraints
        assert!(!deserialized.name.is_empty(), "Name should not be empty after deserialization");
        assert!(deserialized.max_concurrent_requests > 0, "Request limit should be positive");
        assert!(deserialized.timeout_ms > 0, "Timeout should be positive");
        assert!(!deserialized.llm_provider.is_empty(), "LLM provider should be specified");
        assert!(!deserialized.llm_model.is_empty(), "LLM model should be specified");
        
        // Validate collections are preserved correctly
        assert_eq!(deserialized.enabled_validators.len(), 2, "Should preserve validator count");
        assert!(deserialized.enabled_validators.contains(&"validator1".to_string()), "Should preserve validator names");
        assert_eq!(deserialized.tools.len(), 2, "Should preserve tool count");
        assert_eq!(deserialized.metadata.len(), 1, "Should preserve metadata");
        
        // Test serialization is deterministic for same data
        let reserialized = serde_json::to_string(&deserialized).expect("Should reserialize");
        let reparsed: serde_json::Value = serde_json::from_str(&reserialized).expect("Should be valid JSON");
        assert_eq!(parsed, reparsed, "Serialization should be deterministic");
    }

    #[test]
    fn agent_builder_pattern() {
        let config = AgentBuilder::new("test-agent")
            .description("Test agent description")
            .max_concurrent_requests(15)
            .timeout_ms(60000)
            .add_validator("anti-jailbreak")
            .add_validator("rate-limit")
            .add_tool("search")
            .add_tool("calculator")
            .llm_provider("openai")
            .llm_model("gpt-4")
            .build();

        // Test builder creates valid, well-formed configuration
        assert!(!config.name.is_empty(), "Agent name should not be empty");
        assert!(config.description.is_some(), "Builder should set description");
        assert!(config.max_concurrent_requests > 0, "Should allow concurrent requests");
        assert!(config.timeout_ms >= 1000, "Timeout should be reasonable");
        
        // Test collection management works correctly
        assert!(config.enabled_validators.len() >= 2, "Should accumulate validators");
        assert!(config.enabled_validators.contains(&"anti-jailbreak".to_string()), 
                "Should preserve validator order/content");
        assert!(config.tools.len() >= 2, "Should accumulate tools");
        assert!(config.tools.contains(&"search".to_string()), 
                "Should preserve tool content");
        
        // Test required fields are set
        assert!(!config.llm_provider.is_empty(), "LLM provider should be specified");
        assert!(!config.llm_model.is_empty(), "LLM model should be specified");
    }

    #[test]
    fn agent_builder_handles_edge_cases() {
        // Test builder with minimal configuration
        let minimal_config = AgentBuilder::new("")
            .build();
            
        // Builder should apply sensible defaults for edge cases
        assert!(minimal_config.max_concurrent_requests > 0, "Should default to positive concurrency");
        assert!(minimal_config.timeout_ms > 0, "Should default to positive timeout");
        assert!(!minimal_config.llm_provider.is_empty(), "Should have default LLM provider");
        assert!(!minimal_config.llm_model.is_empty(), "Should have default LLM model");
        
        // Test builder accumulates items correctly
        let multi_tool_config = AgentBuilder::new("multi-tool")
            .add_tool("tool1")
            .add_tool("tool2")
            .add_tool("tool1") // Duplicate
            .build();
            
        assert_eq!(multi_tool_config.tools.len(), 3, "Should preserve all tool additions");
        assert_eq!(
            multi_tool_config.tools.iter().filter(|&t| t == "tool1").count(),
            2,
            "Should allow duplicate tools"
        );
    }

    #[test]
    fn agent_request_response_serialization() {
        let request = AgentRequest {
            id: Uuid::new_v4(),
            user_id: Some("user123".to_string()),
            message: "Hello world".to_string(),
            tool_calls: vec![],
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("session".to_string(), serde_json::json!("session123"));
                ctx
            },
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&request).expect("Should serialize request");
        let _deserialized: AgentRequest = serde_json::from_str(&serialized).expect("Should deserialize request");

        let response = AgentResponse {
            request_id: request.id,
            message: "Hello back".to_string(),
            tool_results: vec![],
            usage: crate::traits::Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                cost_usd: Some(0.001),
            },
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&response).expect("Should serialize response");
        let _deserialized: AgentResponse = serde_json::from_str(&serialized).expect("Should deserialize response");
    }

    #[test]
    fn health_status_variants() {
        let healthy = HealthStatus::Healthy;
        let degraded = HealthStatus::Degraded { reason: "High latency".to_string() };
        let unhealthy = HealthStatus::Unhealthy { reason: "Connection failed".to_string() };

        // Should be able to debug print all variants
        assert!(!format!("{:?}", healthy).is_empty());
        assert!(!format!("{:?}", degraded).is_empty());
        assert!(!format!("{:?}", unhealthy).is_empty());

        // Should be cloneable
        let _healthy_clone = healthy.clone();
        let _degraded_clone = degraded.clone();
        let _unhealthy_clone = unhealthy.clone();
    }

    #[tokio::test]
    async fn test_agent_object_safety() {
        // Test that we can create trait objects
        let agent: Box<dyn Agent> = Box::new(TestAgent::new("test"));
        
        // Test that trait object methods work
        let _id = agent.id();
        let _state = agent.state();
        let _config = agent.config();
        let _tools = agent.available_tools();
        let _health = agent.health().await;

        // Test that we can store multiple agents in a collection
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(TestAgent::new("agent1")),
            Box::new(TestAgent::new("agent2")),
        ];

        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_agent_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Box<dyn Agent>>();
        assert_sync::<Box<dyn Agent>>();

        // Test that we can pass trait objects across thread boundaries
        let agent: Box<dyn Agent> = Box::new(TestAgent::new("test"));
        let _agent_id = agent.id();

        tokio::spawn(async move {
            let _id = agent.id();
            // Agent trait object can be moved across threads
        }).await.unwrap();
    }
}

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
    
    /// Start the agent (Created -> Started/Running)
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
    pub tool_calls: Vec<crate::traits::tool::ToolCall>,
    pub context: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub request_id: Uuid,
    pub message: String,
    pub tool_results: Vec<crate::traits::tool::ToolResult>,
    pub usage: crate::traits::Usage,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
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
                enabled_validators: vec![],
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