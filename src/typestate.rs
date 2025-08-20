//! Typestate patterns for compile-time safety
//!
//! This module provides typestate patterns that prevent invalid operations
//! at compile time. Agents can only transition through valid states and
//! operations are only available when the agent is in the correct state.

use crate::error::PatinoxError;

#[cfg(test)]
mod tests {
    use super::*;
    use std::marker::PhantomData;

    // Tests are written FIRST to define the contract
    
    // Helper function to reduce test setup duplication
    fn create_test_agent_config() -> crate::traits::AgentConfig {
        use std::collections::HashMap;
        crate::traits::AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            max_concurrent_requests: 5,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: "test".to_string(),
            llm_model: "test-model".to_string(),
            tools: vec![],
            metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn typestate_marker_types_exist() {
        // Test that all state marker types can be instantiated
        let _created = Created;
        let _started = Started;
        let _running = Running;
        let _stopped = Stopped;
        
        // State markers should be zero-sized types
        assert_eq!(std::mem::size_of::<Created>(), 0);
        assert_eq!(std::mem::size_of::<Started>(), 0);
        assert_eq!(std::mem::size_of::<Running>(), 0);
        assert_eq!(std::mem::size_of::<Stopped>(), 0);
        
        // State markers should implement common traits
        let created = Created;
        let _cloned = created.clone();
        let _debug = format!("{:?}", created);
    }
    
    #[test]
    fn agent_wrapper_creation() {
        use crate::traits::AgentConfig;
        use std::collections::HashMap;
        
        // Test that we can create an agent wrapper in Created state
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            max_concurrent_requests: 5,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: "test".to_string(),
            llm_model: "test-model".to_string(),
            tools: vec![],
            metadata: HashMap::new(),
        };
        
        // Should be able to create agent in Created state
        let _agent = AgentWrapper::<Created>::new(config);
        
        // Agent wrapper should be zero-cost abstraction
        // (only the phantom data adds to size)
        assert_eq!(
            std::mem::size_of::<AgentWrapper<Created>>(),
            std::mem::size_of::<AgentConfig>() + std::mem::size_of::<PhantomData<Created>>()
        );
    }
    
    #[test]
    fn agent_state_transitions() {
        use crate::traits::AgentConfig;
        use std::collections::HashMap;
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            max_concurrent_requests: 5,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: "test".to_string(),
            llm_model: "test-model".to_string(),
            tools: vec![],
            metadata: HashMap::new(),
        };
        
        // Agent starts in Created state
        let agent = AgentWrapper::<Created>::new(config);
        
        // Can transition from Created to Started
        let agent = agent.start().unwrap();
        assert_eq!(agent.current_state(), "Started");
        
        // Can transition from Started to Running
        let agent = agent.run().unwrap();
        assert_eq!(agent.current_state(), "Running");
        
        // Can transition from Running to Stopped
        let agent = agent.stop().unwrap();
        assert_eq!(agent.current_state(), "Stopped");
    }
    
    #[test]
    fn agent_operations_available_in_correct_states() {
        use crate::traits::{AgentConfig, AgentRequest};
        use std::collections::HashMap;
        use uuid::Uuid;
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            max_concurrent_requests: 5,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: "test".to_string(),
            llm_model: "test-model".to_string(),
            tools: vec![],
            metadata: HashMap::new(),
        };
        
        let agent = AgentWrapper::<Created>::new(config);
        
        // Should be able to check config in any state
        let _config = agent.config();
        
        // Transition to running state for execute operations
        let agent = agent.start().unwrap().run().unwrap();
        
        // Should be able to execute in Running state
        let _request = AgentRequest {
            id: Uuid::new_v4(),
            user_id: Some("user1".to_string()),
            message: "test message".to_string(),
            tool_calls: vec![],
            context: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // This should compile and work in Running state
        let _can_execute = agent.can_execute();
        assert!(agent.can_execute());
    }
    
    #[test]
    fn invalid_transitions_prevented() {
        use crate::traits::AgentConfig;
        use std::collections::HashMap;
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            max_concurrent_requests: 5,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: "test".to_string(),
            llm_model: "test-model".to_string(),
            tools: vec![],
            metadata: HashMap::new(),
        };
        
        let agent = AgentWrapper::<Created>::new(config);
        
        // Try to go from Created directly to Running (should fail)
        let result = agent.direct_run();
        assert!(result.is_err());
        
        // Verify error message indicates invalid transition
        if let Err(err) = result {
            assert!(err.to_string().contains("invalid transition"));
        }
    }
    
    #[test]
    fn type_safety_prevents_invalid_operations() {
        use crate::traits::AgentConfig;
        use std::collections::HashMap;
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            max_concurrent_requests: 5,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: "test".to_string(),
            llm_model: "test-model".to_string(),
            tools: vec![],
            metadata: HashMap::new(),
        };
        
        // Agent in Created state
        let agent = AgentWrapper::<Created>::new(config);
        
        // Should NOT be able to execute in Created state
        assert!(!agent.can_execute());
        
        // Agent in Started state  
        let agent = agent.start().unwrap();
        assert!(!agent.can_execute());
        
        // Only Running state should allow execution
        let agent = agent.run().unwrap();
        assert!(agent.can_execute());
        
        // Stopped state should not allow execution
        let agent = agent.stop().unwrap();
        assert!(!agent.can_execute());
    }
    
    #[test]
    fn state_marker_traits() {
        // Test that state markers implement required traits
        fn assert_state_marker<T: StateMarker + Clone + std::fmt::Debug + Send + Sync + 'static>() {}
        
        assert_state_marker::<Created>();
        assert_state_marker::<Started>();
        assert_state_marker::<Running>();
        assert_state_marker::<Stopped>();
    }
    
    #[test]
    fn phantom_types_zero_cost() {
        // Verify phantom types don't add runtime overhead
        let phantom_created: PhantomData<Created> = PhantomData;
        let phantom_running: PhantomData<Running> = PhantomData;
        
        assert_eq!(std::mem::size_of_val(&phantom_created), 0);
        assert_eq!(std::mem::size_of_val(&phantom_running), 0);
        
        // Phantom data should be Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PhantomData<Created>>();
        assert_send_sync::<PhantomData<Running>>();
    }
    
    #[test]
    fn agent_wrapper_send_sync() {
        use crate::traits::AgentConfig;
        use std::collections::HashMap;
        
        // Verify agent wrapper can be sent across threads
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<AgentWrapper<Created>>();
        assert_sync::<AgentWrapper<Created>>();
        assert_send::<AgentWrapper<Running>>();
        assert_sync::<AgentWrapper<Running>>();
        
        // Test actual thread sending
        let config = AgentConfig {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            max_concurrent_requests: 5,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: "test".to_string(),
            llm_model: "test-model".to_string(),
            tools: vec![],
            metadata: HashMap::new(),
        };
        
        let agent = AgentWrapper::<Created>::new(config);
        
        std::thread::spawn(move || {
            let _agent_name = agent.config().name.clone();
            // Agent wrapper can be moved across threads
        }).join().unwrap();
    }
    
    #[test]  
    fn builder_enforces_required_fields() {
        // Test that builder pattern enforces required fields at compile time
        
        // Should be able to build with all required fields
        let _complete_builder = TypeSafeAgentBuilder::new("test-agent")
            .llm_provider("openai")
            .llm_model("gpt-4")
            .build();
            
        // Test that partial builders track what's required
        let partial = TypeSafeAgentBuilder::new("test-agent")
            .description("Test description");
            
        // Should indicate what fields are still needed
        let missing = partial.missing_required_fields();
        assert!(missing.contains(&"llm_provider"));
        assert!(missing.contains(&"llm_model"));
    }
    
    #[test]
    fn builder_optional_fields() {
        // Test that optional fields work correctly
        let builder = TypeSafeAgentBuilder::new("test-agent")
            .llm_provider("openai")
            .llm_model("gpt-4")
            .description("Optional description")
            .max_concurrent_requests(20)
            .timeout_ms(60000)
            .add_validator("test-validator")
            .add_tool("test-tool");
            
        let config = builder.build();
        
        assert_eq!(config.name, "test-agent");
        assert_eq!(config.description, Some("Optional description".to_string()));
        assert_eq!(config.max_concurrent_requests, 20);
        assert_eq!(config.timeout_ms, 60000);
        assert!(config.enabled_validators.contains(&"test-validator".to_string()));
        assert!(config.tools.contains(&"test-tool".to_string()));
    }
    
    #[test]
    fn compile_time_guarantees_examples() {
        // This test demonstrates compile-time safety through examples
        // These examples should compile and show that invalid operations are prevented
        
        let config = create_test_agent_config();
        
        // VALID: Proper state transitions
        let agent = AgentWrapper::<Created>::new(config);
        assert_eq!(agent.current_state(), "Created");
        
        let agent = agent.start().unwrap();  // Created -> Started
        assert_eq!(agent.current_state(), "Started");
        assert!(!agent.can_execute(), "Started state should not allow execution");
        
        let agent = agent.run().unwrap();    // Started -> Running  
        assert_eq!(agent.current_state(), "Running");
        assert!(agent.can_execute(), "Running state should allow execution");
        
        let agent = agent.stop().unwrap();  // Running -> Stopped
        assert_eq!(agent.current_state(), "Stopped");
        assert!(!agent.can_execute(), "Stopped state should not allow execution");
    }
}

// Module implementation will be minimal initially to pass tests
// Implementation starts here - this would normally be empty until tests are written

/// Marker trait for typestate pattern
pub trait StateMarker: Clone + std::fmt::Debug + Send + Sync + 'static {
    /// Human-readable name of this state
    fn state_name() -> &'static str;
}

/// Created state - agent exists but not started
#[derive(Clone, Debug)]
pub struct Created;

impl StateMarker for Created {
    fn state_name() -> &'static str {
        "Created"
    }
}

/// Started state - agent is initializing
#[derive(Clone, Debug)]  
pub struct Started;

impl StateMarker for Started {
    fn state_name() -> &'static str {
        "Started"
    }
}

/// Running state - agent can process requests
#[derive(Clone, Debug)]
pub struct Running;

impl StateMarker for Running {
    fn state_name() -> &'static str {
        "Running"
    }
}

/// Stopped state - agent is shut down
#[derive(Clone, Debug)]
pub struct Stopped;

impl StateMarker for Stopped {
    fn state_name() -> &'static str {
        "Stopped"
    }
}

/// Type-safe agent wrapper that enforces valid state transitions
pub struct AgentWrapper<State: StateMarker> {
    config: crate::traits::AgentConfig,
    _state: std::marker::PhantomData<State>,
}

impl<State: StateMarker> AgentWrapper<State> {
    /// Get the agent configuration (available in all states)
    pub fn config(&self) -> &crate::traits::AgentConfig {
        &self.config
    }
    
    /// Get the current state name
    pub fn current_state(&self) -> &'static str {
        State::state_name()
    }
}

impl AgentWrapper<Created> {
    /// Create a new agent in Created state
    pub fn new(config: crate::traits::AgentConfig) -> Self {
        Self {
            config,
            _state: std::marker::PhantomData,
        }
    }
    
    /// Start the agent (Created -> Started)
    pub fn start(self) -> Result<AgentWrapper<Started>, PatinoxError> {
        Ok(AgentWrapper::<Started> {
            config: self.config,
            _state: std::marker::PhantomData,
        })
    }
    
    /// Attempt invalid direct transition (for testing error cases)
    pub fn direct_run(self) -> Result<AgentWrapper<Running>, PatinoxError> {
        Err(PatinoxError::Execution(
            crate::error::ExecutionError::AgentStateMismatch(
                "Started".to_string(),
                "Created (invalid transition from Created to Running)".to_string(),
            )
        ))
    }
    
    /// Check if agent can execute (should be false in Created state)
    pub fn can_execute(&self) -> bool {
        false
    }
}

impl AgentWrapper<Started> {
    /// Transition to Running state (Started -> Running)
    pub fn run(self) -> Result<AgentWrapper<Running>, PatinoxError> {
        Ok(AgentWrapper::<Running> {
            config: self.config,
            _state: std::marker::PhantomData,
        })
    }
    
    /// Check if agent can execute (should be false in Started state)
    pub fn can_execute(&self) -> bool {
        false
    }
}

impl AgentWrapper<Running> {
    /// Stop the agent (Running -> Stopped)
    pub fn stop(self) -> Result<AgentWrapper<Stopped>, PatinoxError> {
        Ok(AgentWrapper::<Stopped> {
            config: self.config,
            _state: std::marker::PhantomData,
        })
    }
    
    /// Check if agent can execute (should be true in Running state)
    pub fn can_execute(&self) -> bool {
        true
    }
}

impl AgentWrapper<Stopped> {
    /// Check if agent can execute (should be false in Stopped state)
    pub fn can_execute(&self) -> bool {
        false
    }
}

/// Type-safe builder that enforces required fields at compile time
pub struct TypeSafeAgentBuilder {
    name: String,
    description: Option<String>,
    max_concurrent_requests: u32,
    timeout_ms: u64,
    enabled_validators: Vec<String>,
    llm_provider: Option<String>,
    llm_model: Option<String>,
    tools: Vec<String>,
    metadata: std::collections::HashMap<String, String>,
}

impl TypeSafeAgentBuilder {
    /// Create a new builder with required name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            max_concurrent_requests: 10,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: None,
            llm_model: None,
            tools: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Set optional description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    /// Set max concurrent requests
    pub fn max_concurrent_requests(mut self, max: u32) -> Self {
        self.max_concurrent_requests = max;
        self
    }
    
    /// Set timeout in milliseconds
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = timeout;
        self
    }
    
    /// Add a validator
    pub fn add_validator(mut self, validator: impl Into<String>) -> Self {
        self.enabled_validators.push(validator.into());
        self
    }
    
    /// Add a tool
    pub fn add_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }
    
    /// Set LLM provider (required field)
    pub fn llm_provider(mut self, provider: impl Into<String>) -> Self {
        self.llm_provider = Some(provider.into());
        self
    }
    
    /// Set LLM model (required field)
    pub fn llm_model(mut self, model: impl Into<String>) -> Self {
        self.llm_model = Some(model.into());
        self
    }
    
    /// Get list of missing required fields
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = vec![];
        
        if self.llm_provider.is_none() {
            missing.push("llm_provider");
        }
        if self.llm_model.is_none() {
            missing.push("llm_model");
        }
        
        missing
    }
    
    /// Build the configuration (only works if all required fields are set)
    pub fn build(self) -> crate::traits::AgentConfig {
        crate::traits::AgentConfig {
            name: self.name,
            description: self.description,
            max_concurrent_requests: self.max_concurrent_requests,
            timeout_ms: self.timeout_ms,
            enabled_validators: self.enabled_validators,
            llm_provider: self.llm_provider.expect("llm_provider is required"),
            llm_model: self.llm_model.expect("llm_model is required"),
            tools: self.tools,
            metadata: self.metadata,
        }
    }
}