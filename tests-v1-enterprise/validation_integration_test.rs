//! Integration tests for the complete validation pipeline
//!
//! These tests ensure that the Tower validation middleware integrates properly
//! with the complete Patinox framework and all validators work together.

use patinox::error::PatinoxError;
use patinox::traits::validator::{
    ValidationContent, ValidationRequest, ValidationResponse, ValidationStage,
};
use patinox::validation::{ValidationLayer, ValidationPipeline};
use std::collections::HashMap;
use std::sync::Arc;
use tower::{Layer, Service, ServiceExt};
use uuid::Uuid;

#[tokio::test]
async fn test_complete_validation_pipeline_integration() {
    // Arrange - Create a complete pipeline with all validators
    let pipeline = ValidationPipeline::builder()
        .add_request_validator(Default::default())
        .add_anti_jailbreak_validator(Arc::new(MockLlmProvider::new()), Default::default())
        .add_hallucination_detector(Arc::new(MockLlmProvider::new()), Default::default())
        .build();

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "This is a safe, valid message for testing".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = pipeline.validate(request).await;

    // Assert
    assert!(
        response.is_ok(),
        "Complete pipeline should validate successfully"
    );
    let validation_response = response.unwrap();
    assert!(
        validation_response.approved,
        "Safe content should pass all validators"
    );
}

#[tokio::test]
async fn test_validation_pipeline_stops_on_first_rejection() {
    // Arrange - Create pipeline that will reject at the first validator
    let pipeline = ValidationPipeline::builder()
        .add_request_validator(RequestValidatorConfig {
            max_message_length: 10, // Very short limit
            ..Default::default()
        })
        .add_anti_jailbreak_validator(Arc::new(MockLlmProvider::new()), Default::default())
        .build();

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "This message is definitely too long for the configured limit".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = pipeline.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Pipeline should complete validation");
    let validation_response = response.unwrap();
    assert!(
        !validation_response.approved,
        "Should be rejected by first validator"
    );
    assert!(validation_response.reason.unwrap().contains("too long"));
}

#[tokio::test]
async fn test_validation_pipeline_with_tower_service_integration() {
    // Arrange - Test integration with Tower Service trait
    let validation_layer =
        ValidationLayer::with_default_validators(Arc::new(MockLlmProvider::new()));

    let mut service = validation_layer.layer(MockAgentService::new());
    let _ready_service = service.ready().await.expect("Service should be ready");

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Test message for Tower integration".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = service.call(request).await;

    // Assert
    assert!(response.is_ok(), "Tower service integration should work");
}

#[tokio::test]
async fn test_validation_pipeline_preserves_error_context() {
    // Arrange - Create pipeline with a failing validator
    let pipeline = ValidationPipeline::builder()
        .add_custom_validator(Arc::new(FailingValidator::new()))
        .build();

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Any message".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = pipeline.validate(request).await;

    // Assert
    assert!(response.is_err(), "Should propagate validator errors");
    match response.unwrap_err() {
        PatinoxError::Validation(msg) => {
            assert!(format!("{:?}", msg).contains("Test validator failure"));
            // Should preserve original error context
        }
        other => panic!("Expected ValidationError, got {:?}", other),
    }
}

// Mock implementations for integration testing

use patinox::provider::types::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelCapabilities,
    ModelId, ModelInfo, StreamingResponse,
};
use patinox::provider::{ModelProvider, ProviderResult};

struct MockLlmProvider;

impl MockLlmProvider {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ModelProvider for MockLlmProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        Ok(vec![])
    }

    async fn complete(&self, _request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        Ok(CompletionResponse {
            model: ModelId::new("mock-model"),
            content: "SAFE: This appears to be normal content.".to_string(),
            usage: None,
            finish_reason: "stop".to_string(),
        })
    }

    async fn stream_completion(
        &self,
        _request: CompletionRequest,
    ) -> ProviderResult<StreamingResponse> {
        unimplemented!("Not needed for validation testing")
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        unimplemented!("Not needed for validation testing")
    }

    async fn supports_model(&self, _model: &ModelId) -> bool {
        true
    }

    async fn model_capabilities(&self, _model: &ModelId) -> Option<ModelCapabilities> {
        None
    }

    fn name(&self) -> &str {
        "mock-provider"
    }
}

#[derive(Clone)]
struct MockAgentService;

impl MockAgentService {
    fn new() -> Self {
        Self
    }
}

impl tower::Service<ValidationRequest> for MockAgentService {
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
                reason: Some("Mock agent service approved".to_string()),
                modifications: None,
                metadata: HashMap::new(),
            })
        })
    }
}

use patinox::traits::validator::{Validator, ValidatorConfig};

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
                parameters: std::collections::HashMap::new(),
            },
        }
    }
}

#[async_trait::async_trait]
impl Validator for FailingValidator {
    fn name(&self) -> &str {
        "failing-validator"
    }

    fn config(&self) -> &ValidatorConfig {
        &self.config
    }

    fn should_validate(&self, _request: &ValidationRequest) -> bool {
        true
    }

    async fn validate(
        &self,
        _request: ValidationRequest,
    ) -> Result<ValidationResponse, PatinoxError> {
        Err(PatinoxError::Validation(
            patinox::error::ValidationError::InvalidInput("Test validator failure".to_string()),
        ))
    }
}

// Placeholder config types (will be implemented in the actual code)
use patinox::validation::validators::RequestValidatorConfig;

// Note: These are just type references for the tests.
// The actual implementations will be created during the implementation phase.
