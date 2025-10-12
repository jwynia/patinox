//! Tests for Hallucination Detector
//!
//! These tests define the contract for the hallucination detector that validates
//! LLM responses for accuracy and consistency. Written FIRST using TDD methodology.

use patinox::error::PatinoxError;
use patinox::traits::validator::{
    ValidationContent, ValidationRequest, ValidationResponse, ValidationStage, ValidatorConfig,
};
use patinox::validation::validators::HallucinationDetector;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_hallucination_detector_creation() {
    // Arrange & Act
    let detector = HallucinationDetector::new(
        Arc::new(MockLlmProvider::new()),
        HallucinationConfig::default(),
    );

    // Assert
    assert_eq!(detector.name(), "hallucination-detector");
    assert!(detector.config().enabled);
    assert_eq!(detector.config().priority, 2); // After anti-jailbreak but high priority
    assert!(detector
        .config()
        .stages
        .contains(&ValidationStage::PostExecution));
}

#[tokio::test]
async fn test_hallucination_detector_validates_llm_responses() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new_with_response(
        "ACCURATE: The response appears to be factually correct and well-grounded.",
    ));
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::LlmResponse {
            message: "The capital of France is Paris, which has been the capital since 1792.".to_string(),
            tool_calls: vec![],
        },
        context: HashMap::new(),
    };

    // Act
    let response = detector.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Accurate response should be approved");
    assert!(validation_response.reason.is_some());
}

#[tokio::test]
async fn test_hallucination_detector_flags_inaccurate_content() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new_with_response(
        "INACCURATE: This response contains factual errors and unsupported claims.",
    ));
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::LlmResponse {
            message: "The capital of France is London, and the Eiffel Tower is in Berlin.".to_string(),
            tool_calls: vec![],
        },
        context: HashMap::new(),
    };

    // Act
    let response = detector.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(!validation_response.approved, "Inaccurate content should be flagged");
    assert!(validation_response.reason.is_some());
    assert!(validation_response
        .reason
        .unwrap()
        .contains("inaccurate"));
}

#[tokio::test]
async fn test_hallucination_detector_flags_unsupported_claims() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new_with_response(
        "UNSUPPORTED: This response makes claims without sufficient evidence or context.",
    ));
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::LlmResponse {
            message: "I have access to real-time data and can see current stock prices.".to_string(),
            tool_calls: vec![],
        },
        context: HashMap::new(),
    };

    // Act
    let response = detector.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(!validation_response.approved, "Unsupported claims should be flagged");
    assert!(validation_response.reason.is_some());
}

#[tokio::test]
async fn test_hallucination_detector_handles_context_awareness() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new_with_response(
        "CONTEXT_APPROPRIATE: Response is consistent with provided context.",
    ));
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    let mut context = HashMap::new();
    context.insert(
        "conversation_history".to_string(),
        serde_json::json!([
            {"role": "user", "content": "What did I just ask about?"},
            {"role": "assistant", "content": "You asked about the capital of France."}
        ]),
    );

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::LlmResponse {
            message: "You just asked about the capital of France.".to_string(),
            tool_calls: vec![],
        },
        context,
    };

    // Act
    let response = detector.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Context-appropriate response should be approved");
}

#[tokio::test]
async fn test_hallucination_detector_validates_tool_call_consistency() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new_with_response(
        "TOOL_CONSISTENT: The response correctly describes the tool calls made.",
    ));
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    let tool_calls = vec![
        patinox::traits::tool::ToolCall {
            id: "call-1".to_string(),
            name: "get_weather".to_string(),
            parameters: serde_json::json!({"location": "Paris"}),
        }
    ];

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::LlmResponse {
            message: "I'll check the weather in Paris for you.".to_string(),
            tool_calls: tool_calls.clone(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = detector.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Consistent tool usage should be approved");
}

#[tokio::test]
async fn test_hallucination_detector_flags_inconsistent_tool_calls() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new_with_response(
        "TOOL_INCONSISTENT: The response does not match the tool calls being made.",
    ));
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    let tool_calls = vec![
        patinox::traits::tool::ToolCall {
            id: "call-1".to_string(),
            name: "get_weather".to_string(),
            parameters: serde_json::json!({"location": "Paris"}),
        }
    ];

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::LlmResponse {
            message: "I've sent you an email with the information.".to_string(), // Inconsistent!
            tool_calls,
        },
        context: HashMap::new(),
    };

    // Act
    let response = detector.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(!validation_response.approved, "Inconsistent tool calls should be flagged");
}

#[tokio::test]
async fn test_hallucination_detector_respects_stage_filtering() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new());
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    // Should validate PostExecution and PreResponse stages
    let post_execution_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::LlmResponse {
            message: "Test response".to_string(),
            tool_calls: vec![],
        },
        context: HashMap::new(),
    };

    let pre_execution_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Test message".to_string(),
        },
        context: HashMap::new(),
    };

    // Act & Assert
    assert!(detector.should_validate(&post_execution_request));
    assert!(!detector.should_validate(&pre_execution_request));
}

#[tokio::test]
async fn test_hallucination_detector_handles_provider_errors() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new_failing());
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::LlmResponse {
            message: "Any response".to_string(),
            tool_calls: vec![],
        },
        context: HashMap::new(),
    };

    // Act
    let response = detector.validate(request).await;

    // Assert
    assert!(response.is_err(), "Should propagate provider errors");
    match response.unwrap_err() {
        PatinoxError::ValidationError(msg) => {
            assert!(msg.contains("Hallucination detection failed"));
        }
        other => panic!("Expected ValidationError, got {:?}", other),
    }
}

#[tokio::test]
async fn test_hallucination_detector_with_custom_config() {
    // Arrange
    let config = HallucinationConfig {
        fact_check_prompt: "Custom fact-checking prompt: {}".to_string(),
        confidence_threshold: 0.8,
        context_window_size: 5,
        require_citations: true,
    };

    let mock_provider = Arc::new(MockLlmProvider::new());
    let detector = HallucinationDetector::new(mock_provider, config);

    // Act & Assert
    assert_eq!(detector.name(), "hallucination-detector");

    // Verify config was applied
    let validator_config = detector.config();
    assert!(validator_config.parameters.contains_key("confidence_threshold"));
    assert!(validator_config.parameters.contains_key("require_citations"));
}

#[tokio::test]
async fn test_hallucination_detector_handles_non_llm_content() {
    // Arrange
    let mock_provider = Arc::new(MockLlmProvider::new());
    let detector = HallucinationDetector::new(mock_provider, HallucinationConfig::default());

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PostExecution,
        content: ValidationContent::UserMessage {
            message: "User message".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = detector.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Should handle non-LLM content gracefully");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Non-LLM content should be approved by default");
}

// Mock implementations for testing (reusing from anti_jailbreak tests with minor variations)

use patinox::provider::{ModelProvider, ProviderResult};
use patinox::provider::types::{CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelInfo, ModelId, ModelCapabilities};

struct MockLlmProvider {
    response: String,
    should_fail: bool,
}

impl MockLlmProvider {
    fn new() -> Self {
        Self {
            response: "ACCURATE: This appears to be factually correct content.".to_string(),
            should_fail: false,
        }
    }

    fn new_with_response(response: &str) -> Self {
        Self {
            response: response.to_string(),
            should_fail: false,
        }
    }

    fn new_failing() -> Self {
        Self {
            response: String::new(),
            should_fail: true,
        }
    }
}

#[async_trait::async_trait]
impl ModelProvider for MockLlmProvider {
    async fn list_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        Ok(vec![])
    }

    async fn complete(&self, _request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        if self.should_fail {
            return Err(patinox::provider::ProviderError::NetworkError(
                "Mock failure".to_string(),
            ));
        }

        Ok(CompletionResponse {
            model: ModelId::new("mock-model"),
            content: self.response.clone(),
            usage: None,
            finish_reason: "stop".to_string(),
        })
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        unimplemented!("Not needed for hallucination testing")
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

// Config types that need to be implemented
#[derive(Debug, Clone)]
struct HallucinationConfig {
    fact_check_prompt: String,
    confidence_threshold: f64,
    context_window_size: usize,
    require_citations: bool,
}

impl Default for HallucinationConfig {
    fn default() -> Self {
        Self {
            fact_check_prompt: "Check this response for factual accuracy and unsupported claims: {}".to_string(),
            confidence_threshold: 0.7,
            context_window_size: 3,
            require_citations: false,
        }
    }
}