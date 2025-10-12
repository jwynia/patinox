//! Validator trait definition and supporting types
//!
//! This module defines the core Validator trait that implements the Tower Service
//! pattern for composable validation middleware. Validators check agent requests
//! and responses at various stages of execution.

use crate::error::PatinoxError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Tests written FIRST to define the contract
#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct TestValidator {
        name: String,
        config: ValidatorConfig,
    }

    #[async_trait]
    impl Validator for TestValidator {
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
            // Simple validation logic for testing
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

    impl TestValidator {
        fn new(name: &str) -> Self {
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

    #[test]
    fn validation_stage_enum_completeness() {
        let stages = vec![
            ValidationStage::PreExecution,
            ValidationStage::PostExecution,
            ValidationStage::PostTool,
            ValidationStage::PreResponse,
        ];

        for stage in stages {
            // Should be cloneable
            let _cloned = stage.clone();
            // Should be debuggable
            let _debug = format!("{:?}", stage);
        }
    }

    #[test]
    fn validation_content_variants() {
        let user_message = ValidationContent::UserMessage {
            message: "Hello".to_string(),
        };

        let llm_response = ValidationContent::LlmResponse {
            message: "Response".to_string(),
            tool_calls: vec![],
        };

        let tool_result = ValidationContent::ToolResult {
            tool_name: "test-tool".to_string(),
            result: crate::traits::tool::ToolResult {
                call_id: "call-1".to_string(),
                success: true,
                data: serde_json::json!({}),
                error: None,
                metadata: HashMap::new(),
            },
        };

        let final_response = ValidationContent::FinalResponse {
            message: "Final".to_string(),
        };

        // All variants should be cloneable and debuggable
        let _user_clone = user_message.clone();
        let _llm_clone = llm_response.clone();
        let _tool_clone = tool_result.clone();
        let _final_clone = final_response.clone();

        assert!(!format!("{:?}", user_message).is_empty());
        assert!(!format!("{:?}", llm_response).is_empty());
        assert!(!format!("{:?}", tool_result).is_empty());
        assert!(!format!("{:?}", final_response).is_empty());
    }

    #[test]
    fn validation_request_structure() {
        let request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "Test message".to_string(),
            },
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("session".to_string(), serde_json::json!("session123"));
                ctx
            },
        };

        // Should be cloneable
        let _cloned = request.clone();
        // Should be debuggable
        assert!(!format!("{:?}", request).is_empty());
        // Should have proper field access
        assert_eq!(request.stage, ValidationStage::PreExecution);
        assert_eq!(request.context["session"], "session123");
    }

    #[test]
    fn validation_response_structure() {
        let response = ValidationResponse {
            approved: true,
            reason: Some("Validation passed".to_string()),
            modifications: Some(ValidationModifications {
                modified_content: "Modified content".to_string(),
                blocked_tool_calls: vec!["dangerous-tool".to_string()],
                added_warnings: vec!["Warning message".to_string()],
            }),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("validator".to_string(), "test-validator".to_string());
                meta
            },
        };

        // Should be cloneable
        let _cloned = response.clone();
        // Should be debuggable
        assert!(!format!("{:?}", response).is_empty());

        // Test field access
        assert!(response.approved);
        assert!(response.reason.is_some());
        assert!(response.modifications.is_some());

        let modifications = response.modifications.expect("Should have modifications");
        assert_eq!(modifications.modified_content, "Modified content");
        assert!(modifications
            .blocked_tool_calls
            .contains(&"dangerous-tool".to_string()));
        assert!(modifications
            .added_warnings
            .contains(&"Warning message".to_string()));
    }

    #[test]
    fn validator_config_serialization() {
        let config = ValidatorConfig {
            name: "test-validator".to_string(),
            enabled: true,
            priority: 5,
            stages: vec![ValidationStage::PreExecution, ValidationStage::PostTool],
            parameters: {
                let mut params = HashMap::new();
                params.insert("max_length".to_string(), serde_json::json!(1000));
                params.insert("check_sentiment".to_string(), serde_json::json!(true));
                params
            },
        };

        let serialized = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: ValidatorConfig =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.priority, config.priority);
        assert_eq!(deserialized.stages.len(), config.stages.len());
        assert_eq!(deserialized.parameters["max_length"], 1000);
        assert_eq!(deserialized.parameters["check_sentiment"], true);
    }

    #[tokio::test]
    async fn validator_basic_functionality() {
        let validator = TestValidator::new("test-validator");

        // Test that validator contract is properly implemented
        assert!(
            !validator.name().is_empty(),
            "Validator name should not be empty"
        );

        let config = validator.config();
        assert!(
            config.enabled,
            "Test validator should be enabled by default"
        );
        assert!(config.priority >= 0, "Priority should be non-negative");
        assert!(
            !config.stages.is_empty(),
            "Validator should handle at least one stage"
        );

        // Test that validator configuration is consistent
        assert_eq!(
            validator.name(),
            config.name,
            "Validator name should match config name"
        );
    }

    #[tokio::test]
    async fn validator_should_validate_logic() {
        let validator = TestValidator::new("test-validator");

        let pre_execution_request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "Test".to_string(),
            },
            context: HashMap::new(),
        };

        let post_tool_request = ValidationRequest {
            stage: ValidationStage::PostTool,
            ..pre_execution_request.clone()
        };

        // Should validate PreExecution (configured stage)
        assert!(validator.should_validate(&pre_execution_request));

        // Should not validate PostTool (not configured)
        assert!(!validator.should_validate(&post_tool_request));
    }

    #[tokio::test]
    async fn validator_service_interface() {
        let validator = TestValidator::new("test-validator");

        // Test safe content validation logic
        let safe_request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "Hello world".to_string(),
            },
            context: HashMap::new(),
        };

        let response = validator.validate(safe_request).await;
        assert!(response.is_ok(), "Validator should handle valid requests");

        let validation_response = response.expect("Validation should succeed");
        assert!(
            validation_response.approved,
            "Safe content should be approved"
        );
        assert!(
            validation_response.reason.is_none(),
            "Approved content should not have rejection reason"
        );
        assert!(
            validation_response.modifications.is_none(),
            "Safe content should not need modifications"
        );

        // Test unsafe content detection and rejection
        let unsafe_request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "This is unsafe content".to_string(),
            },
            context: HashMap::new(),
        };

        let response = validator.validate(unsafe_request).await;
        assert!(
            response.is_ok(),
            "Validator should handle unsafe content gracefully"
        );

        let validation_response = response.expect("Validation should complete");
        assert!(
            !validation_response.approved,
            "Unsafe content should be rejected"
        );
        assert!(
            validation_response.reason.is_some(),
            "Rejected content should include reason"
        );
        if let Some(reason) = validation_response.reason {
            assert!(
                reason.contains("Unsafe content"),
                "Rejection reason should be specific"
            );
            assert!(!reason.is_empty(), "Reason should be descriptive");
        }

        // Test validator consistency across multiple calls
        let duplicate_safe_request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "Hello world".to_string(), // Same safe message
            },
            context: HashMap::new(),
        };

        let duplicate_response = validator.validate(duplicate_safe_request).await;
        let duplicate_validation = duplicate_response.expect("Should validate consistently");
        assert!(
            duplicate_validation.approved,
            "Validator should be consistent for same input"
        );

        // Test different content types are handled
        let tool_content_request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::LlmResponse {
                message: "AI response".to_string(),
                tool_calls: vec![],
            },
            context: HashMap::new(),
        };

        let tool_response = validator.validate(tool_content_request).await;
        assert!(
            tool_response.is_ok(),
            "Validator should handle different content types"
        );
        let tool_validation = tool_response.expect("Should validate tool content");
        assert!(
            tool_validation.approved,
            "Non-user content should have different validation rules"
        );
    }

    #[tokio::test]
    async fn validator_object_safety() {
        // Test that we can create trait objects
        let validator: Box<dyn Validator> = Box::new(TestValidator::new("boxed-validator"));

        // Test that trait object methods work
        let _name = validator.name();
        let _config = validator.config();

        let request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: "Test".to_string(),
            },
            context: HashMap::new(),
        };

        let _should_validate = validator.should_validate(&request);

        // Test that we can store multiple validators in a collection
        let validators: Vec<Box<dyn Validator>> = vec![
            Box::new(TestValidator::new("validator1")),
            Box::new(TestValidator::new("validator2")),
        ];

        assert_eq!(validators.len(), 2);
    }

    #[tokio::test]
    async fn validator_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Box<dyn Validator>>();
        assert_sync::<Box<dyn Validator>>();

        // Test that we can pass trait objects across thread boundaries
        let validator: Box<dyn Validator> = Box::new(TestValidator::new("thread-test"));
        let validator_name = validator.name().to_string();

        tokio::spawn(async move {
            let _name = validator.name();
            // Validator trait object can be moved across threads
        })
        .await
        .unwrap();

        assert_eq!(validator_name, "thread-test");
    }

    #[test]
    fn validation_modifications_structure() {
        let modifications = ValidationModifications {
            modified_content: "Updated content".to_string(),
            blocked_tool_calls: vec!["tool1".to_string(), "tool2".to_string()],
            added_warnings: vec!["Warning 1".to_string(), "Warning 2".to_string()],
        };

        // Should be cloneable
        let cloned = modifications.clone();
        assert_eq!(cloned.modified_content, modifications.modified_content);
        assert_eq!(cloned.blocked_tool_calls.len(), 2);
        assert_eq!(cloned.added_warnings.len(), 2);

        // Should be debuggable
        assert!(!format!("{:?}", modifications).is_empty());
    }

    #[test]
    fn validation_stage_equality() {
        assert_eq!(ValidationStage::PreExecution, ValidationStage::PreExecution);
        assert_ne!(
            ValidationStage::PreExecution,
            ValidationStage::PostExecution
        );

        // Test that stages can be used in collections
        let stages = [
            ValidationStage::PreExecution,
            ValidationStage::PostExecution,
            ValidationStage::PostTool,
            ValidationStage::PreResponse,
        ];

        assert!(stages.contains(&ValidationStage::PreExecution));
        assert!(stages.contains(&ValidationStage::PostTool));
    }
}

/// Validator trait for request validation
///
/// Note: This trait is designed to be object-safe while still being compatible
/// with Tower Service. Individual implementations can implement Service if needed,
/// but the core trait focuses on the validation interface.
#[async_trait]
pub trait Validator: Send + Sync {
    /// Validator name for identification
    fn name(&self) -> &str;

    /// Validator configuration
    fn config(&self) -> &ValidatorConfig;

    /// Whether this validator should run for the given request
    fn should_validate(&self, request: &ValidationRequest) -> bool;

    /// Validate a request
    async fn validate(
        &self,
        request: ValidationRequest,
    ) -> Result<ValidationResponse, PatinoxError>;
}

#[derive(Debug, Clone)]
pub struct ValidationRequest {
    pub agent_id: Uuid,
    pub request_id: Uuid,
    pub stage: ValidationStage,
    pub content: ValidationContent,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStage {
    PreExecution,  // Before LLM call
    PostExecution, // After LLM call, before tool execution
    PostTool,      // After tool execution
    PreResponse,   // Before sending response to user
}

#[derive(Debug, Clone)]
pub enum ValidationContent {
    UserMessage {
        message: String,
    },
    LlmResponse {
        message: String,
        tool_calls: Vec<crate::traits::tool::ToolCall>,
    },
    ToolResult {
        tool_name: String,
        result: crate::traits::tool::ToolResult,
    },
    FinalResponse {
        message: String,
    },
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
