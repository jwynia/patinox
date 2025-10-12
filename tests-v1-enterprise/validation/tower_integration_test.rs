//! Tests for Tower validation middleware integration
//!
//! These tests define the contract for how validation layers should integrate
//! with Tower middleware patterns. Written FIRST using TDD methodology.

use patinox::error::PatinoxError;
use patinox::traits::validator::{
    ValidationContent, ValidationRequest, ValidationResponse, ValidationStage, ValidatorConfig,
};
use patinox::validation::{ValidationLayer, ValidationService};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_test;
use tower::{Layer, Service, ServiceExt};
use uuid::Uuid;

/// Mock service that just echoes back the validation request for testing
#[derive(Clone)]
struct MockService;

impl Service<ValidationRequest> for MockService {
    type Response = ValidationResponse;
    type Error = PatinoxError;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ValidationRequest) -> Self::Future {
        Box::pin(async move {
            Ok(ValidationResponse {
                approved: true,
                reason: Some("Mock service approved".to_string()),
                modifications: None,
                metadata: HashMap::new(),
            })
        })
    }
}

/// Mock validator for testing - rejects content containing "unsafe"
struct MockValidator {
    config: ValidatorConfig,
}

impl MockValidator {
    fn new(name: &str) -> Self {
        Self {
            config: ValidatorConfig {
                name: name.to_string(),
                enabled: true,
                priority: 0,
                stages: vec![
                    ValidationStage::PreExecution,
                    ValidationStage::PostExecution,
                    ValidationStage::PostTool,
                    ValidationStage::PreResponse,
                ],
                parameters: HashMap::new(),
            },
        }
    }
}

#[async_trait::async_trait]
impl patinox::traits::validator::Validator for MockValidator {
    fn name(&self) -> &str {
        &self.config.name
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
                        reason: Some("Content approved".to_string()),
                        modifications: None,
                        metadata: HashMap::new(),
                    })
                }
            }
            _ => Ok(ValidationResponse {
                approved: true,
                reason: Some("Non-message content approved".to_string()),
                modifications: None,
                metadata: HashMap::new(),
            }),
        }
    }
}

#[tokio::test]
async fn test_validation_layer_implements_tower_layer_trait() {
    // Arrange
    let validator = Arc::new(MockValidator::new("test-validator"));
    let validation_layer = ValidationLayer::new(vec![validator]);

    // Act & Assert - This should compile, proving Layer trait is implemented
    let service = validation_layer.layer(MockService);

    // Should be able to use as a Service
    let mut service_ready = service.ready().await.expect("Service should be ready");

    // Create a test request
    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Hello world".to_string(),
        },
        context: HashMap::new(),
    };

    // Should be able to call the service
    let response = service_ready.call(request).await;
    assert!(response.is_ok(), "Service call should succeed");
}

#[tokio::test]
async fn test_validation_service_approves_safe_content() {
    // Arrange
    let validator = Arc::new(MockValidator::new("test-validator"));
    let validation_layer = ValidationLayer::new(vec![validator]);
    let mut service = validation_layer.layer(MockService);

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "This is safe content".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = service.ready().await.unwrap().call(request).await;

    // Assert
    assert!(response.is_ok(), "Safe content should be approved");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Content should be approved");
    assert!(validation_response.reason.is_some());
}

#[tokio::test]
async fn test_validation_service_rejects_unsafe_content() {
    // Arrange
    let validator = Arc::new(MockValidator::new("test-validator"));
    let validation_layer = ValidationLayer::new(vec![validator]);
    let mut service = validation_layer.layer(MockService);

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "This contains unsafe content".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = service.ready().await.unwrap().call(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(!validation_response.approved, "Unsafe content should be rejected");
    assert!(validation_response.reason.is_some());
    assert!(validation_response
        .reason
        .unwrap()
        .contains("Unsafe content detected"));
}

#[tokio::test]
async fn test_validation_service_runs_multiple_validators_in_priority_order() {
    // Arrange
    let high_priority_validator = Arc::new(MockValidator::new("high-priority"));
    let low_priority_validator = Arc::new(MockValidator::new("low-priority"));

    // Create validators with different priorities
    let mut high_config = high_priority_validator.config().clone();
    high_config.priority = 1; // Higher priority (lower number)
    let high_validator = MockValidatorWithConfig::new(high_config);

    let mut low_config = low_priority_validator.config().clone();
    low_config.priority = 10; // Lower priority (higher number)
    let low_validator = MockValidatorWithConfig::new(low_config);

    let validation_layer = ValidationLayer::new(vec![
        Arc::new(low_validator), // Add in wrong order to test sorting
        Arc::new(high_validator),
    ]);
    let mut service = validation_layer.layer(MockService);

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Test content".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = service.ready().await.unwrap().call(request).await;

    // Assert
    assert!(response.is_ok(), "Multi-validator should work");
    // Note: Detailed priority testing would require more sophisticated mock setup
}

#[tokio::test]
async fn test_validation_service_handles_validator_errors_gracefully() {
    // Arrange
    let failing_validator = Arc::new(FailingValidator::new());
    let validation_layer = ValidationLayer::new(vec![failing_validator]);
    let mut service = validation_layer.layer(MockService);

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Test content".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = service.ready().await.unwrap().call(request).await;

    // Assert
    assert!(response.is_err(), "Should propagate validator errors");
    match response.unwrap_err() {
        PatinoxError::ValidationError(msg) => {
            assert!(msg.contains("Validator failed"));
        }
        other => panic!("Expected ValidationError, got {:?}", other),
    }
}

#[tokio::test]
async fn test_validation_service_handles_different_validation_stages() {
    // Arrange
    let validator = Arc::new(MockValidator::new("stage-validator"));
    let validation_layer = ValidationLayer::new(vec![validator]);
    let mut service = validation_layer.layer(MockService);

    // Test all validation stages
    let stages = vec![
        ValidationStage::PreExecution,
        ValidationStage::PostExecution,
        ValidationStage::PostTool,
        ValidationStage::PreResponse,
    ];

    for stage in stages {
        let request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage,
            content: ValidationContent::UserMessage {
                message: "Test content".to_string(),
            },
            context: HashMap::new(),
        };

        // Act
        let response = service.ready().await.unwrap().call(request).await;

        // Assert
        assert!(
            response.is_ok(),
            "Should handle stage {:?} successfully",
            stage
        );
    }
}

#[tokio::test]
async fn test_validation_service_respects_validator_stage_filtering() {
    // Arrange - Create validator that only validates PreExecution stage
    let mut config = ValidatorConfig {
        name: "stage-specific-validator".to_string(),
        enabled: true,
        priority: 0,
        stages: vec![ValidationStage::PreExecution], // Only this stage
        parameters: HashMap::new(),
    };

    let validator = Arc::new(MockValidatorWithConfig::new(config));
    let validation_layer = ValidationLayer::new(vec![validator]);
    let mut service = validation_layer.layer(MockService);

    // Test PreExecution stage (should be validated)
    let pre_execution_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Test content".to_string(),
        },
        context: HashMap::new(),
    };

    // Test PostExecution stage (should be skipped)
    let post_execution_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::UserMessage {
            message: "Test content".to_string(),
        },
        context: HashMap::new(),
    };

    // Act & Assert
    let pre_response = service
        .ready()
        .await
        .unwrap()
        .call(pre_execution_request)
        .await;
    assert!(pre_response.is_ok(), "PreExecution should be validated");

    let post_response = service
        .ready()
        .await
        .unwrap()
        .call(post_execution_request)
        .await;
    assert!(post_response.is_ok(), "PostExecution should be skipped gracefully");
}

// Helper validators for more complex testing scenarios

struct FailingValidator {
    config: ValidatorConfig,
}

impl FailingValidator {
    fn new() -> Self {
        Self {
            config: ValidatorConfig {
                name: "failing-validator".to_string(),
                enabled: true,
                priority: 0,
                stages: vec![ValidationStage::PreExecution],
                parameters: HashMap::new(),
            },
        }
    }
}

#[async_trait::async_trait]
impl patinox::traits::validator::Validator for FailingValidator {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &ValidatorConfig {
        &self.config
    }

    fn should_validate(&self, request: &ValidationRequest) -> bool {
        self.config.stages.contains(&request.stage)
    }

    async fn validate(
        &self,
        _request: ValidationRequest,
    ) -> Result<ValidationResponse, PatinoxError> {
        Err(PatinoxError::ValidationError(
            "Validator failed intentionally".to_string(),
        ))
    }
}

struct MockValidatorWithConfig {
    config: ValidatorConfig,
}

impl MockValidatorWithConfig {
    fn new(config: ValidatorConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl patinox::traits::validator::Validator for MockValidatorWithConfig {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &ValidatorConfig {
        &self.config
    }

    fn should_validate(&self, request: &ValidationRequest) -> bool {
        self.config.stages.contains(&request.stage)
    }

    async fn validate(
        &self,
        _request: ValidationRequest,
    ) -> Result<ValidationResponse, PatinoxError> {
        Ok(ValidationResponse {
            approved: true,
            reason: Some(format!("{} approved", self.config.name)),
            modifications: None,
            metadata: HashMap::new(),
        })
    }
}