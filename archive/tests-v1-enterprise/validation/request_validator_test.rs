//! Tests for Request Validator
//!
//! These tests define the contract for the request validator that performs
//! input sanitization, bounds checking, and basic validation.
//! Written FIRST using TDD methodology.

use patinox::error::PatinoxError;
use patinox::traits::validator::{
    ValidationContent, ValidationRequest, ValidationResponse, ValidationStage, ValidatorConfig,
    ValidationModifications,
};
use patinox::validation::validators::RequestValidator;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_request_validator_creation() {
    // Arrange & Act
    let validator = RequestValidator::new(RequestValidatorConfig::default());

    // Assert
    assert_eq!(validator.name(), "request-validator");
    assert!(validator.config().enabled);
    assert_eq!(validator.config().priority, 0); // Runs first
    assert!(validator
        .config()
        .stages
        .contains(&ValidationStage::PreExecution));
}

#[tokio::test]
async fn test_request_validator_enforces_message_length_limits() {
    // Arrange
    let config = RequestValidatorConfig {
        max_message_length: 100,
        min_message_length: 5,
        ..Default::default()
    };
    let validator = RequestValidator::new(config);

    // Test message too long
    let long_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "a".repeat(150), // Exceeds limit
        },
        context: HashMap::new(),
    };

    // Test message too short
    let short_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "hi".to_string(), // Below minimum
        },
        context: HashMap::new(),
    };

    // Test valid message
    let valid_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "This is a valid message".to_string(),
        },
        context: HashMap::new(),
    };

    // Act & Assert
    let long_response = validator.validate(long_request).await;
    assert!(long_response.is_ok());
    assert!(!long_response.unwrap().approved, "Long message should be rejected");

    let short_response = validator.validate(short_request).await;
    assert!(short_response.is_ok());
    assert!(!short_response.unwrap().approved, "Short message should be rejected");

    let valid_response = validator.validate(valid_request).await;
    assert!(valid_response.is_ok());
    assert!(valid_response.unwrap().approved, "Valid message should be approved");
}

#[tokio::test]
async fn test_request_validator_sanitizes_html_content() {
    // Arrange
    let config = RequestValidatorConfig {
        sanitize_html: true,
        ..Default::default()
    };
    let validator = RequestValidator::new(config);

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Hello <script>alert('xss')</script> world".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = validator.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Should approve after sanitization");
    assert!(validation_response.modifications.is_some(), "Should provide modifications");

    let modifications = validation_response.modifications.unwrap();
    assert!(modifications.modified_content.contains("Hello") &&
           !modifications.modified_content.contains("<script>"),
           "Should sanitize HTML tags");
}

#[tokio::test]
async fn test_request_validator_blocks_prohibited_content() {
    // Arrange
    let config = RequestValidatorConfig {
        prohibited_patterns: vec![
            "password".to_string(),
            "credit.*card".to_string(),
            r"\b\d{4}-\d{4}-\d{4}-\d{4}\b".to_string(), // Credit card pattern
        ],
        ..Default::default()
    };
    let validator = RequestValidator::new(config);

    let prohibited_requests = vec![
        "What's my password?",
        "My credit card number is 1234-5678-9012-3456",
        "Can you store my credit card info?",
    ];

    for prohibited_content in prohibited_requests {
        let request = ValidationRequest {
            agent_id: Uuid::new_v4(),
            request_id: Uuid::new_v4(),
            stage: ValidationStage::PreExecution,
            content: ValidationContent::UserMessage {
                message: prohibited_content.to_string(),
            },
            context: HashMap::new(),
        };

        // Act
        let response = validator.validate(request).await;

        // Assert
        assert!(response.is_ok(), "Validation should complete successfully");
        let validation_response = response.unwrap();
        assert!(!validation_response.approved,
               "Prohibited content '{}' should be rejected", prohibited_content);
        assert!(validation_response.reason.is_some());
    }
}

#[tokio::test]
async fn test_request_validator_normalizes_unicode() {
    // Arrange
    let config = RequestValidatorConfig {
        normalize_unicode: true,
        ..Default::default()
    };
    let validator = RequestValidator::new(config);

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "café naïve résumé".to_string(), // Contains accented characters
        },
        context: HashMap::new(),
    };

    // Act
    let response = validator.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Unicode content should be approved");

    if let Some(modifications) = validation_response.modifications {
        // Should normalize to ASCII equivalents or consistent Unicode form
        assert!(!modifications.modified_content.is_empty());
    }
}

#[tokio::test]
async fn test_request_validator_enforces_rate_limiting_context() {
    // Arrange
    let config = RequestValidatorConfig {
        check_rate_limiting: true,
        max_requests_per_minute: 5,
        ..Default::default()
    };
    let validator = RequestValidator::new(config);

    let agent_id = Uuid::new_v4();

    // Add rate limiting context to simulate previous requests
    let mut context = HashMap::new();
    context.insert(
        "recent_requests".to_string(),
        serde_json::json!([
            {"timestamp": "2023-01-01T12:00:00Z"},
            {"timestamp": "2023-01-01T12:00:10Z"},
            {"timestamp": "2023-01-01T12:00:20Z"},
            {"timestamp": "2023-01-01T12:00:30Z"},
            {"timestamp": "2023-01-01T12:00:40Z"},
            {"timestamp": "2023-01-01T12:00:50Z"}, // 6th request - should trigger limit
        ]),
    );

    let request = ValidationRequest {
        agent_id,
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "This might exceed rate limit".to_string(),
        },
        context,
    };

    // Act
    let response = validator.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(!validation_response.approved, "Should reject due to rate limiting");
    assert!(validation_response.reason.unwrap().contains("rate limit"));
}

#[tokio::test]
async fn test_request_validator_validates_context_structure() {
    // Arrange
    let config = RequestValidatorConfig {
        validate_context: true,
        required_context_keys: vec!["session_id".to_string(), "user_id".to_string()],
        ..Default::default()
    };
    let validator = RequestValidator::new(config);

    // Test missing required context
    let invalid_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Valid message".to_string(),
        },
        context: HashMap::new(), // Missing required keys
    };

    // Test valid context
    let mut valid_context = HashMap::new();
    valid_context.insert("session_id".to_string(), serde_json::json!("session-123"));
    valid_context.insert("user_id".to_string(), serde_json::json!("user-456"));

    let valid_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Valid message".to_string(),
        },
        context: valid_context,
    };

    // Act & Assert
    let invalid_response = validator.validate(invalid_request).await;
    assert!(invalid_response.is_ok());
    assert!(!invalid_response.unwrap().approved, "Should reject missing context");

    let valid_response = validator.validate(valid_request).await;
    assert!(valid_response.is_ok());
    assert!(valid_response.unwrap().approved, "Should approve valid context");
}

#[tokio::test]
async fn test_request_validator_handles_non_message_content() {
    // Arrange
    let validator = RequestValidator::new(RequestValidatorConfig::default());

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::FinalResponse {
            message: "Final response".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = validator.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Should handle non-UserMessage content");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Non-UserMessage should be approved by default");
}

#[tokio::test]
async fn test_request_validator_respects_stage_filtering() {
    // Arrange
    let validator = RequestValidator::new(RequestValidatorConfig::default());

    let pre_execution_request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Test message".to_string(),
        },
        context: HashMap::new(),
    };

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

    // Act & Assert
    assert!(validator.should_validate(&pre_execution_request));
    assert!(!validator.should_validate(&post_execution_request));
}

#[tokio::test]
async fn test_request_validator_provides_detailed_validation_info() {
    // Arrange
    let config = RequestValidatorConfig {
        provide_detailed_info: true,
        ..Default::default()
    };
    let validator = RequestValidator::new(config);

    let request = ValidationRequest {
        agent_id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        stage: ValidationStage::PreExecution,
        content: ValidationContent::UserMessage {
            message: "Test message".to_string(),
        },
        context: HashMap::new(),
    };

    // Act
    let response = validator.validate(request).await;

    // Assert
    assert!(response.is_ok(), "Validation should complete successfully");
    let validation_response = response.unwrap();
    assert!(validation_response.approved, "Valid message should be approved");
    assert!(!validation_response.metadata.is_empty(), "Should provide validation metadata");

    // Check for expected metadata
    assert!(validation_response.metadata.contains_key("message_length"));
    assert!(validation_response.metadata.contains_key("validation_checks_passed"));
}

// Configuration struct that needs to be implemented
#[derive(Debug, Clone)]
struct RequestValidatorConfig {
    max_message_length: usize,
    min_message_length: usize,
    sanitize_html: bool,
    normalize_unicode: bool,
    prohibited_patterns: Vec<String>,
    check_rate_limiting: bool,
    max_requests_per_minute: u32,
    validate_context: bool,
    required_context_keys: Vec<String>,
    provide_detailed_info: bool,
}

impl Default for RequestValidatorConfig {
    fn default() -> Self {
        Self {
            max_message_length: 10000,
            min_message_length: 1,
            sanitize_html: true,
            normalize_unicode: true,
            prohibited_patterns: vec![
                "password".to_string(),
                "api[_-]?key".to_string(),
                "secret".to_string(),
            ],
            check_rate_limiting: false,
            max_requests_per_minute: 60,
            validate_context: false,
            required_context_keys: vec![],
            provide_detailed_info: false,
        }
    }
}