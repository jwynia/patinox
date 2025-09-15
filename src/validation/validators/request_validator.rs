//! Request validator implementation
//!
//! Provides input sanitization, bounds checking, and basic validation
//! for incoming requests.

use crate::error::PatinoxError;
use crate::traits::validator::{
    ValidationContent, ValidationModifications, ValidationRequest, ValidationResponse,
    ValidationStage, Validator, ValidatorConfig,
};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;

/// Configuration for the request validator
#[derive(Debug, Clone)]
pub struct RequestValidatorConfig {
    pub max_message_length: usize,
    pub min_message_length: usize,
    pub sanitize_html: bool,
    pub normalize_unicode: bool,
    pub prohibited_patterns: Vec<String>,
    pub check_rate_limiting: bool,
    pub max_requests_per_minute: u32,
    pub validate_context: bool,
    pub required_context_keys: Vec<String>,
    pub provide_detailed_info: bool,
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

/// Request validator that performs input sanitization and bounds checking
pub struct RequestValidator {
    config: ValidatorConfig,
    request_config: RequestValidatorConfig,
    prohibited_regexes: Vec<Regex>,
}

impl RequestValidator {
    /// Create a new request validator with the specified configuration
    pub fn new(request_config: RequestValidatorConfig) -> Self {
        // Compile prohibited patterns into regexes
        let prohibited_regexes = request_config
            .prohibited_patterns
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        // Create validator config
        let mut parameters = HashMap::new();
        parameters.insert(
            "max_message_length".to_string(),
            serde_json::json!(request_config.max_message_length),
        );
        parameters.insert(
            "min_message_length".to_string(),
            serde_json::json!(request_config.min_message_length),
        );
        parameters.insert(
            "sanitize_html".to_string(),
            serde_json::json!(request_config.sanitize_html),
        );

        let config = ValidatorConfig {
            name: "request-validator".to_string(),
            enabled: true,
            priority: 0, // Runs first
            stages: vec![ValidationStage::PreExecution],
            parameters,
        };

        Self {
            config,
            request_config,
            prohibited_regexes,
        }
    }

    /// Check if the message length is within bounds
    fn validate_message_length(&self, message: &str) -> Result<(), String> {
        let length = message.len();

        if length > self.request_config.max_message_length {
            return Err(format!(
                "Message too long: {} characters (max: {})",
                length, self.request_config.max_message_length
            ));
        }

        if length < self.request_config.min_message_length {
            return Err(format!(
                "Message too short: {} characters (min: {})",
                length, self.request_config.min_message_length
            ));
        }

        Ok(())
    }

    /// Check for prohibited content patterns
    fn check_prohibited_content(&self, message: &str) -> Result<(), String> {
        for regex in &self.prohibited_regexes {
            if regex.is_match(message) {
                return Err(format!(
                    "Message contains prohibited content matching pattern: {}",
                    regex.as_str()
                ));
            }
        }
        Ok(())
    }

    /// Sanitize HTML content from the message
    fn sanitize_html(&self, message: &str) -> String {
        if !self.request_config.sanitize_html {
            return message.to_string();
        }

        // Simple HTML tag removal (in production, use a proper HTML sanitizer)
        let html_regex = Regex::new(r"<[^>]*>").unwrap();
        html_regex.replace_all(message, "").to_string()
    }

    /// Normalize unicode content
    fn normalize_unicode(&self, message: &str) -> String {
        if !self.request_config.normalize_unicode {
            return message.to_string();
        }

        // Simple normalization (in production, use proper Unicode normalization)
        message
            .chars()
            .map(|c| match c {
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
                'ç' => 'c',
                _ => c,
            })
            .collect()
    }

    /// Check rate limiting based on context
    fn check_rate_limiting(
        &self,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        if !self.request_config.check_rate_limiting {
            return Ok(());
        }

        if let Some(recent_requests) = context.get("recent_requests") {
            if let Some(requests_array) = recent_requests.as_array() {
                if requests_array.len() > self.request_config.max_requests_per_minute as usize {
                    return Err("Rate limit exceeded".to_string());
                }
            }
        }

        Ok(())
    }

    /// Validate required context keys
    fn validate_context(&self, context: &HashMap<String, serde_json::Value>) -> Result<(), String> {
        if !self.request_config.validate_context {
            return Ok(());
        }

        for required_key in &self.request_config.required_context_keys {
            if !context.contains_key(required_key) {
                return Err(format!("Missing required context key: {}", required_key));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Validator for RequestValidator {
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
        // Only validate UserMessage content for request validation
        let message = match &request.content {
            ValidationContent::UserMessage { message } => message,
            _ => {
                // Approve non-user messages by default
                return Ok(ValidationResponse {
                    approved: true,
                    reason: Some("Non-user message content approved by default".to_string()),
                    modifications: None,
                    metadata: HashMap::new(),
                });
            }
        };

        // Perform all validation checks
        if let Err(error) = self.validate_message_length(message) {
            return Ok(ValidationResponse {
                approved: false,
                reason: Some(error),
                modifications: None,
                metadata: HashMap::new(),
            });
        }

        if let Err(error) = self.check_prohibited_content(message) {
            return Ok(ValidationResponse {
                approved: false,
                reason: Some(error),
                modifications: None,
                metadata: HashMap::new(),
            });
        }

        if let Err(error) = self.check_rate_limiting(&request.context) {
            return Ok(ValidationResponse {
                approved: false,
                reason: Some(error),
                modifications: None,
                metadata: HashMap::new(),
            });
        }

        if let Err(error) = self.validate_context(&request.context) {
            return Ok(ValidationResponse {
                approved: false,
                reason: Some(error),
                modifications: None,
                metadata: HashMap::new(),
            });
        }

        // Apply modifications if needed
        let mut modified_message = message.clone();
        let mut has_modifications = false;

        if self.request_config.sanitize_html {
            let sanitized = self.sanitize_html(&modified_message);
            if sanitized != modified_message {
                modified_message = sanitized;
                has_modifications = true;
            }
        }

        if self.request_config.normalize_unicode {
            let normalized = self.normalize_unicode(&modified_message);
            if normalized != modified_message {
                modified_message = normalized;
                has_modifications = true;
            }
        }

        // Build response
        let modifications = if has_modifications {
            Some(ValidationModifications {
                modified_content: modified_message,
                blocked_tool_calls: vec![],
                added_warnings: vec![],
            })
        } else {
            None
        };

        let mut metadata = HashMap::new();
        if self.request_config.provide_detailed_info {
            metadata.insert("message_length".to_string(), message.len().to_string());
            metadata.insert("validation_checks_passed".to_string(), "all".to_string());
        }

        Ok(ValidationResponse {
            approved: true,
            reason: Some("Request validation passed".to_string()),
            modifications,
            metadata,
        })
    }
}
