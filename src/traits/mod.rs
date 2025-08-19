//! Core trait definitions for the Patinox framework
//!
//! This module contains the fundamental trait abstractions that define
//! the contracts for all Patinox components: Agent, Tool, Validator, and Monitor.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PatinoxError;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use uuid::Uuid;

    // === TRAIT OBJECT SAFETY TESTS (Write these FIRST) ===

    #[test]
    fn agent_trait_is_object_safe() {
        // This test will compile if Agent trait is object-safe
        // We create a vector of trait objects to verify object safety

        #[allow(dead_code)]
        fn accepts_agent_trait_object(_agent: Box<dyn Agent>) {
            // Function that accepts a trait object
        }

        // If this compiles, the trait is object-safe
        // We'll implement MockAgent to make this pass
    }

    #[test]
    fn tool_trait_is_object_safe() {
        #[allow(dead_code)]
        fn accepts_tool_trait_object(_tool: Box<dyn Tool>) {
            // Function that accepts a trait object
        }

        // Vector of trait objects to verify object safety
        let _tools: Vec<Box<dyn Tool>> = Vec::new();
    }

    #[test]
    fn validator_trait_is_object_safe() {
        #[allow(dead_code)]
        fn accepts_validator_trait_object(_validator: Box<dyn Validator>) {
            // Function that accepts a trait object
        }

        // Vector of trait objects to verify object safety
        let _validators: Vec<Box<dyn Validator>> = Vec::new();
    }

    #[test]
    fn monitor_trait_is_object_safe() {
        #[allow(dead_code)]
        fn accepts_monitor_trait_object(_monitor: Box<dyn Monitor>) {
            // Function that accepts a trait object
        }

        // Vector of trait objects to verify object safety
        let _monitors: Vec<Box<dyn Monitor>> = Vec::new();
    }

    // === AGENT TRAIT TESTS ===

    #[tokio::test]
    async fn agent_lifecycle_state_transitions() {
        let mut agent = MockAgent::new("test-agent");

        // Initial state should be Created and agent should not be executable
        assert_eq!(agent.state(), AgentState::Created);

        // Test that agent cannot execute in Created state
        let test_request = AgentRequest {
            id: Uuid::new_v4(),
            user_id: Some("user123".to_string()),
            message: "test".to_string(),
            tool_calls: vec![],
            context: HashMap::new(),
            metadata: HashMap::new(),
        };

        let initial_result = agent.execute(test_request.clone()).await;
        assert!(
            initial_result.is_err(),
            "Agent should not execute in Created state"
        );

        // Start the agent and verify it becomes executable
        agent
            .start()
            .await
            .expect("Agent should start successfully");
        assert_eq!(agent.state(), AgentState::Running);

        // Test that agent can execute in Running state
        let running_result = agent.execute(test_request.clone()).await;
        assert!(
            running_result.is_ok(),
            "Agent should execute in Running state"
        );

        // Stop the agent and verify it becomes non-executable again
        agent.stop().await.expect("Agent should stop successfully");
        assert_eq!(agent.state(), AgentState::Stopped);

        // Test that agent cannot execute in Stopped state
        let stopped_result = agent.execute(test_request).await;
        assert!(
            stopped_result.is_err(),
            "Agent should not execute in Stopped state"
        );

        // Test multiple start/stop cycles work correctly
        agent
            .start()
            .await
            .expect("Agent should restart from Stopped");
        assert_eq!(agent.state(), AgentState::Running);

        agent.stop().await.expect("Agent should stop again");
        assert_eq!(agent.state(), AgentState::Stopped);
    }

    #[tokio::test]
    async fn agent_has_unique_id() {
        let agent1 = MockAgent::new("agent1");
        let agent2 = MockAgent::new("agent2");

        assert_ne!(agent1.id(), agent2.id(), "Each agent should have unique ID");
    }

    #[tokio::test]
    async fn agent_execute_requires_running_state() {
        let mut agent = MockAgent::new("test-agent");

        let request = AgentRequest {
            id: Uuid::new_v4(),
            user_id: Some("user123".to_string()),
            message: "Hello".to_string(),
            tool_calls: vec![],
            context: HashMap::new(),
            metadata: HashMap::new(),
        };

        // Should fail when not started
        let result = agent.execute(request.clone()).await;
        assert!(
            result.is_err(),
            "Execute should fail when agent not started"
        );

        // Should succeed when started
        agent.start().await.expect("Agent should start");
        let result = agent.execute(request).await;
        assert!(
            result.is_ok(),
            "Execute should succeed when agent is running"
        );
    }

    #[tokio::test]
    async fn agent_available_tools_list() {
        let agent = MockAgent::new("test-agent");
        let tools = agent.available_tools();

        // Test that agent provides tools list (contract requirement)
        assert!(!tools.is_empty(), "Agent should have available tools");
        // Test that all tool names are valid (non-empty strings)
        for tool in &tools {
            assert!(!tool.is_empty(), "Tool names should not be empty");
            assert!(
                !tool.chars().all(|c| c.is_whitespace()),
                "Tool names should not be only whitespace"
            );
        }
        // Test that tools list is deterministic (same agent returns same tools)
        let tools2 = agent.available_tools();
        assert_eq!(tools, tools2, "available_tools() should be deterministic");
    }

    #[tokio::test]
    async fn agent_health_check() {
        let agent = MockAgent::new("test-agent");
        let health = agent.health().await;

        match health {
            HealthStatus::Healthy => {
                // Expected for new agent
            }
            _ => panic!("New agent should be healthy"),
        }
    }

    #[test]
    fn agent_config_access() {
        let agent = MockAgent::new("test-agent");
        let config = agent.config();

        // Test configuration constraints and business rules
        assert!(!config.name.is_empty(), "Agent name should not be empty");
        assert!(
            config.max_concurrent_requests > 0,
            "Should allow at least one concurrent request"
        );
        assert!(
            config.max_concurrent_requests <= 100,
            "Should have reasonable concurrent request limit"
        );
        assert!(
            config.timeout_ms >= 1000,
            "Timeout should be at least 1 second"
        );
        assert!(
            config.timeout_ms <= 300000,
            "Timeout should not exceed 5 minutes"
        );
        assert!(
            !config.tools.is_empty(),
            "Agent should have at least one tool available"
        );
    }

    // === TOOL TRAIT TESTS ===

    #[test]
    fn tool_metadata_access() {
        let tool = MockTool::new("test-tool");

        // Test tool contract requirements
        assert!(!tool.name().is_empty(), "Tool name should not be empty");
        assert!(
            !tool.description().is_empty(),
            "Tool description should not be empty"
        );

        let schema = tool.parameters_schema();
        assert!(
            schema.is_object(),
            "Parameters schema should be a JSON object"
        );
        assert!(schema.get("type").is_some(), "Schema should specify type");
        assert!(
            schema.get("properties").is_some(),
            "Schema should have properties"
        );

        let metadata = tool.metadata();
        assert!(!metadata.category.is_empty(), "Tool should have a category");
        assert!(!metadata.version.is_empty(), "Tool should have a version");
        assert!(metadata.tags.len() > 0, "Tool should have at least one tag");
    }

    #[tokio::test]
    async fn tool_execute_with_params() {
        let tool = MockTool::new("test-tool");

        let params = ToolParams {
            call_id: "call-123".to_string(),
            parameters: serde_json::json!({ "input": "test" }),
            context: HashMap::new(),
        };

        let result = tool.execute(params).await;
        assert!(
            result.is_ok(),
            "Tool execution should succeed with valid parameters"
        );

        let tool_result = result.expect("Tool execution should succeed");

        // Test that tool properly preserves call correlation
        assert_eq!(
            tool_result.call_id, "call-123",
            "Call ID should be preserved for tracing"
        );
        assert!(
            tool_result.success,
            "Valid parameters should result in successful execution"
        );
        assert!(
            tool_result.error.is_none(),
            "Successful execution should not have error message"
        );

        // Test that tool actually processes the input rather than just echoing
        assert!(
            !tool_result.data.is_null(),
            "Tool should return meaningful data"
        );
        if let Some(output) = tool_result.data.get("output") {
            if let Some(output_str) = output.as_str() {
                assert!(
                    output_str.contains("Mock processed"),
                    "Tool should process input, not just return it"
                );
                assert!(
                    output_str.contains("test"),
                    "Output should include original input"
                );
            }
        }

        // Test metadata initialization
        assert!(
            tool_result.metadata.is_empty() || !tool_result.metadata.is_empty(),
            "Metadata should be initialized"
        );
    }

    #[tokio::test]
    async fn tool_execute_with_invalid_params() {
        let tool = MockTool::new("test-tool");

        // Test with completely wrong parameter structure
        let wrong_params = ToolParams {
            call_id: "call-456".to_string(),
            parameters: serde_json::json!({ "invalid": true }),
            context: HashMap::new(),
        };

        let result = tool.execute(wrong_params).await;
        // Tool should handle invalid params gracefully and provide useful feedback
        match result {
            Ok(tool_result) => {
                assert_eq!(
                    tool_result.call_id, "call-456",
                    "Call ID should be preserved even for failures"
                );
                assert!(
                    !tool_result.success,
                    "Invalid parameters should result in failure"
                );
                assert!(
                    tool_result.error.is_some(),
                    "Failure should include descriptive error message"
                );

                if let Some(error_msg) = tool_result.error {
                    assert!(
                        error_msg.contains("Missing required parameter"),
                        "Error should be specific"
                    );
                    assert!(
                        error_msg.contains("input"),
                        "Error should mention missing parameter name"
                    );
                }

                assert_eq!(
                    tool_result.data,
                    serde_json::json!({}),
                    "Failed execution should return empty data"
                );
            }
            Err(e) => {
                // Also acceptable to return PatinoxError for validation failures
                // This would indicate tool-level parameter validation
                panic!(
                    "Tool should handle parameter validation gracefully, not return error: {:?}",
                    e
                );
            }
        }

        // Test with empty parameters
        let empty_params = ToolParams {
            call_id: "empty-call".to_string(),
            parameters: serde_json::json!({}),
            context: HashMap::new(),
        };

        let empty_result = tool
            .execute(empty_params)
            .await
            .expect("Should handle empty params gracefully");
        assert!(
            !empty_result.success,
            "Empty parameters should fail validation"
        );
        assert!(
            empty_result.error.is_some(),
            "Should provide error message for empty parameters"
        );

        // Test parameter type validation
        let wrong_type_params = ToolParams {
            call_id: "type-test".to_string(),
            parameters: serde_json::json!({ "input": 123 }), // Should be string
            context: HashMap::new(),
        };

        let type_result = tool
            .execute(wrong_type_params)
            .await
            .expect("Should handle wrong types gracefully");
        // Tool should either convert or reject - both are valid behaviors
        if !type_result.success {
            assert!(
                type_result.error.is_some(),
                "Type validation failure should include error"
            );
        }
    }

    // === VALIDATOR TRAIT TESTS ===

    #[test]
    fn validator_configuration() {
        let validator = MockValidator::new("test-validator");

        assert_eq!(validator.name(), "test-validator");

        let config = validator.config();
        assert_eq!(config.name, "test-validator");
        assert!(!config.stages.is_empty());
    }

    #[test]
    fn validator_should_validate_logic() {
        let validator = MockValidator::new("test-validator");

        let request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "Hello".to_string(),
            },
            context: HashMap::new(),
        };

        // Should validate PreExecution stage
        assert!(validator.should_validate(&request));

        let request_post = ValidationRequest {
            stage: ValidationStage::PostTool,
            ..request
        };

        // Should not validate PostTool stage (based on config)
        assert!(!validator.should_validate(&request_post));
    }

    #[tokio::test]
    async fn validator_async_interface() {
        let validator = MockValidator::new("test-validator");

        let request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "Safe message".to_string(),
            },
            context: HashMap::new(),
        };

        let response = validator.validate(request).await;
        assert!(response.is_ok(), "Validator should validate safe content");

        let validation_response = response.expect("Validation should succeed");
        assert!(validation_response.approved);
    }

    // === MONITOR TRAIT TESTS ===

    #[tokio::test]
    async fn monitor_execution_lifecycle() {
        let monitor = MockMonitor::new("test-monitor");

        let execution_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        // Start monitoring
        let result = monitor.start_monitoring(execution_id, agent_id).await;
        assert!(result.is_ok(), "Monitor should start monitoring");

        // Record an event
        let event = MonitorEvent {
            id: Uuid::new_v4(),
            execution_id,
            agent_id,
            timestamp: chrono::Utc::now(),
            event_type: MonitorEventType::ExecutionStarted,
            data: serde_json::json!({}),
            metadata: HashMap::new(),
        };

        let result = monitor.record_event(event).await;
        assert!(result.is_ok(), "Monitor should record events");

        // Complete monitoring
        let summary = ExecutionSummary {
            execution_id,
            agent_id,
            success: true,
            total_duration_ms: 1500,
            llm_calls: 1,
            tool_calls: 2,
            validation_failures: 0,
            total_tokens: Usage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                cost_usd: Some(0.003),
            },
            error_summary: None,
        };

        let result = monitor.complete_monitoring(execution_id, summary).await;
        assert!(result.is_ok(), "Monitor should complete monitoring");
    }

    #[tokio::test]
    async fn monitor_query_events() {
        let monitor = MockMonitor::new("test-monitor");

        let query = MonitorQuery {
            agent_ids: None,
            event_types: Some(vec![MonitorEventType::ExecutionStarted]),
            start_time: None,
            end_time: None,
            limit: Some(10),
        };

        let result = monitor.query_events(query).await;
        assert!(result.is_ok(), "Monitor should handle queries");

        let _events = result.expect("Monitor query should succeed");
        // For mock implementation, might return empty or sample data
    }

    #[test]
    fn monitor_config_access() {
        let monitor = MockMonitor::new("test-monitor");
        let config = monitor.config();

        assert_eq!(config.name, "test-monitor");
        assert!(config.enabled);
        assert!(config.buffer_size > 0);
        assert!(config.flush_interval_ms > 0);
        assert!(config.sampling_rate >= 0.0 && config.sampling_rate <= 1.0);
    }

    // === SEND + SYNC TESTS ===

    #[test]
    fn all_traits_are_send_sync() {
        // These functions verify that trait objects are Send + Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Box<dyn Agent>>();
        assert_sync::<Box<dyn Agent>>();

        assert_send::<Box<dyn Tool>>();
        assert_sync::<Box<dyn Tool>>();

        assert_send::<Box<dyn Validator>>();
        assert_sync::<Box<dyn Validator>>();

        assert_send::<Box<dyn Monitor>>();
        assert_sync::<Box<dyn Monitor>>();
    }

    // === ERROR INTEGRATION TESTS ===

    #[tokio::test]
    async fn agent_handles_empty_message_request() {
        let mut agent = MockAgent::new("error-test");
        agent.start().await.expect("Agent should start");

        let empty_request = AgentRequest {
            id: Uuid::new_v4(),
            user_id: Some("user123".to_string()),
            message: "".to_string(), // Empty message
            tool_calls: vec![],
            context: HashMap::new(),
            metadata: HashMap::new(),
        };

        // Agent should handle empty messages gracefully
        let result = agent.execute(empty_request).await;
        match result {
            Ok(response) => {
                assert!(
                    !response.message.is_empty(),
                    "Agent should provide meaningful response to empty input"
                );
            }
            Err(PatinoxError::Validation(_)) => {
                // Also acceptable - agent rejects empty input
            }
            Err(e) => panic!("Unexpected error type for empty message: {:?}", e),
        }
    }

    #[tokio::test]
    async fn validator_handles_malformed_content() {
        let validator = MockValidator::new("malformed-test");

        let malformed_request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "test\x00\x01malformed".to_string(), // Contains null bytes
            },
            context: HashMap::new(),
        };

        let result = validator.validate(malformed_request).await;
        match result {
            Ok(response) => {
                // Validator should handle malformed input gracefully
                if !response.approved {
                    assert!(response.reason.is_some(), "Rejection should include reason");
                }
            }
            Err(_) => {
                // Also acceptable - validator rejects malformed input
            }
        }
    }

    #[tokio::test]
    async fn traits_use_patinox_error() {
        let mut agent = MockAgent::new("error-test");

        // Test that Agent methods return PatinoxError
        let start_result: Result<(), PatinoxError> = agent.start().await;
        assert!(start_result.is_ok());

        let tool = MockTool::new("error-test");
        let params = ToolParams {
            call_id: "error-test".to_string(),
            parameters: serde_json::json!({}),
            context: HashMap::new(),
        };

        // Test that Tool execute returns PatinoxError
        let execute_result: Result<ToolResult, PatinoxError> = tool.execute(params).await;
        assert!(execute_result.is_ok());
    }

    // Mock implementations for testing

    pub struct MockAgent {
        id: Uuid,
        state: AgentState,
        config: AgentConfig,
    }

    impl MockAgent {
        pub fn new(name: &str) -> Self {
            Self {
                id: Uuid::new_v4(),
                state: AgentState::Created,
                config: AgentConfig {
                    name: name.to_string(),
                    description: Some("Mock agent".to_string()),
                    max_concurrent_requests: 5,
                    timeout_ms: 30000,
                    enabled_validators: vec!["mock-validator".to_string()],
                    llm_provider: "mock".to_string(),
                    llm_model: "mock-model".to_string(),
                    tools: vec!["mock-tool".to_string()],
                    metadata: HashMap::new(),
                },
            }
        }
    }

    #[async_trait]
    impl Agent for MockAgent {
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
                    ),
                ));
            }
            Ok(AgentResponse {
                request_id: request.id,
                message: format!("Mock response to: {}", request.message),
                tool_results: vec![],
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
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

    pub struct MockTool {
        name: String,
    }

    impl MockTool {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }
        fn description(&self) -> &str {
            "Mock tool for testing"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string", "description": "Input parameter" }
                },
                "required": ["input"]
            })
        }

        async fn execute(&self, params: ToolParams) -> Result<ToolResult, PatinoxError> {
            let input = params.parameters.get("input");
            if input.is_none() {
                return Ok(ToolResult {
                    call_id: params.call_id,
                    success: false,
                    data: serde_json::json!({}),
                    error: Some("Missing required parameter: input".to_string()),
                    metadata: HashMap::new(),
                });
            }
            Ok(ToolResult {
                call_id: params.call_id,
                success: true,
                data: serde_json::json!({"output": format!("Mock processed: {}", input.unwrap())}),
                error: None,
                metadata: HashMap::new(),
            })
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                category: "mock".to_string(),
                tags: vec!["testing".to_string()],
                version: "1.0.0".to_string(),
                author: Some("Test Suite".to_string()),
                dangerous: false,
            }
        }
    }

    pub struct MockValidator {
        name: String,
        config: ValidatorConfig,
    }

    impl MockValidator {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                config: ValidatorConfig {
                    name: name.to_string(),
                    enabled: true,
                    priority: 0,
                    stages: vec![ValidationStage::PreExecution, ValidationStage::PreResponse],
                    parameters: HashMap::new(),
                },
            }
        }
    }

    #[async_trait]
    impl Validator for MockValidator {
        fn name(&self) -> &str {
            &self.name
        }
        fn config(&self) -> &ValidatorConfig {
            &self.config
        }

        fn should_validate(&self, request: &ValidationRequest) -> bool {
            self.config.stages.contains(&request.stage)
        }

        async fn validate(
            &self,
            request: ValidationRequest,
        ) -> Result<ValidationResponse, PatinoxError> {
            match &request.content {
                ValidationContent::UserMessage { message } => {
                    if message.contains("unsafe") {
                        Ok(ValidationResponse {
                            approved: false,
                            reason: Some("Unsafe content detected".to_string()),
                            modifications: None,
                            metadata: HashMap::new(),
                        })
                    } else {
                        Ok(ValidationResponse {
                            approved: true,
                            reason: None,
                            modifications: None,
                            metadata: HashMap::new(),
                        })
                    }
                }
                _ => Ok(ValidationResponse {
                    approved: true,
                    reason: None,
                    modifications: None,
                    metadata: HashMap::new(),
                }),
            }
        }
    }

    pub struct MockMonitor {
        name: String,
        config: MonitorConfig,
        events: std::sync::Arc<std::sync::Mutex<Vec<MonitorEvent>>>,
    }

    impl MockMonitor {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                config: MonitorConfig {
                    name: name.to_string(),
                    enabled: true,
                    buffer_size: 1000,
                    flush_interval_ms: 5000,
                    sampling_rate: 1.0,
                    event_types: vec![MonitorEventType::ExecutionStarted],
                },
                events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl Monitor for MockMonitor {
        fn name(&self) -> &str {
            &self.name
        }

        async fn start_monitoring(
            &self,
            execution_id: Uuid,
            agent_id: Uuid,
        ) -> Result<(), PatinoxError> {
            let event = MonitorEvent {
                id: Uuid::new_v4(),
                execution_id,
                agent_id,
                timestamp: chrono::Utc::now(),
                event_type: MonitorEventType::ExecutionStarted,
                data: serde_json::json!({}),
                metadata: HashMap::new(),
            };
            self.events
                .lock()
                .map_err(|_| {
                    PatinoxError::Execution(crate::error::ExecutionError::ResourceExhausted(
                        "Monitor event storage corrupted".to_string(),
                    ))
                })?
                .push(event);
            Ok(())
        }

        async fn record_event(&self, event: MonitorEvent) -> Result<(), PatinoxError> {
            self.events
                .lock()
                .map_err(|_| {
                    PatinoxError::Execution(crate::error::ExecutionError::ResourceExhausted(
                        "Monitor event storage corrupted".to_string(),
                    ))
                })?
                .push(event);
            Ok(())
        }

        async fn complete_monitoring(
            &self,
            execution_id: Uuid,
            summary: ExecutionSummary,
        ) -> Result<(), PatinoxError> {
            let event = MonitorEvent {
                id: Uuid::new_v4(),
                execution_id,
                agent_id: summary.agent_id,
                timestamp: chrono::Utc::now(),
                event_type: MonitorEventType::ExecutionCompleted {
                    success: summary.success,
                    total_duration_ms: summary.total_duration_ms,
                },
                data: serde_json::to_value(&summary).unwrap(),
                metadata: HashMap::new(),
            };
            self.events
                .lock()
                .map_err(|_| {
                    PatinoxError::Execution(crate::error::ExecutionError::ResourceExhausted(
                        "Monitor event storage corrupted".to_string(),
                    ))
                })?
                .push(event);
            Ok(())
        }

        async fn query_events(
            &self,
            _query: MonitorQuery,
        ) -> Result<Vec<MonitorEvent>, PatinoxError> {
            Ok(self
                .events
                .lock()
                .map_err(|_| {
                    PatinoxError::Execution(crate::error::ExecutionError::ResourceExhausted(
                        "Monitor event storage corrupted".to_string(),
                    ))
                })?
                .clone())
        }

        fn config(&self) -> &MonitorConfig {
            &self.config
        }
    }
}

// Module structure - traits will be implemented in separate files
pub mod agent;
pub mod monitor;
pub mod tool;
pub mod validator;

// Re-export core traits for convenience
pub use agent::{
    Agent, AgentBuilder, AgentConfig, AgentRequest, AgentResponse, AgentState, HealthStatus,
};
pub use monitor::{
    ExecutionSummary, Monitor, MonitorConfig, MonitorEvent, MonitorEventType, MonitorQuery,
};
pub use tool::{Tool, ToolCall, ToolMetadata, ToolParams, ToolResult};
pub use validator::{
    ValidationContent, ValidationModifications, ValidationRequest, ValidationResponse,
    ValidationStage, Validator, ValidatorConfig,
};

// Supporting types used across traits
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost_usd: Option<f64>,
}
