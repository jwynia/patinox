//! Hallucination detector implementation
//!
//! Validates LLM responses for accuracy, consistency, and grounding.

use crate::error::{PatinoxError, ValidationError};
use crate::provider::{
    types::{CompletionRequest, ModelId},
    ModelProvider,
};
use crate::traits::validator::{
    ValidationContent, ValidationRequest, ValidationResponse, ValidationStage, Validator,
    ValidatorConfig,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for the hallucination detector
#[derive(Debug, Clone)]
pub struct HallucinationConfig {
    pub fact_check_prompt: String,
    pub confidence_threshold: f64,
    pub context_window_size: usize,
    pub require_citations: bool,
}

impl Default for HallucinationConfig {
    fn default() -> Self {
        Self {
            fact_check_prompt:
                "Check this response for factual accuracy and unsupported claims: {}".to_string(),
            confidence_threshold: 0.7,
            context_window_size: 3,
            require_citations: false,
        }
    }
}

/// Hallucination detector that validates LLM responses for accuracy
pub struct HallucinationDetector {
    config: ValidatorConfig,
    hallucination_config: HallucinationConfig,
    llm_provider: Arc<dyn ModelProvider>,
}

impl HallucinationDetector {
    /// Create a new hallucination detector
    pub fn new(
        llm_provider: Arc<dyn ModelProvider>,
        hallucination_config: HallucinationConfig,
    ) -> Self {
        // Create validator config
        let mut parameters = HashMap::new();
        parameters.insert(
            "confidence_threshold".to_string(),
            serde_json::json!(hallucination_config.confidence_threshold),
        );
        parameters.insert(
            "require_citations".to_string(),
            serde_json::json!(hallucination_config.require_citations),
        );

        let config = ValidatorConfig {
            name: "hallucination-detector".to_string(),
            enabled: true,
            priority: 2, // After anti-jailbreak but still high priority
            stages: vec![ValidationStage::PostExecution, ValidationStage::PreResponse],
            parameters,
        };

        Self {
            config,
            hallucination_config,
            llm_provider,
        }
    }

    /// Analyze a response for hallucinations using the LLM provider
    async fn analyze_response(
        &self,
        response: &str,
        tool_calls: &[crate::traits::tool::ToolCall],
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<String, PatinoxError> {
        // Build analysis prompt with context
        let mut analysis_prompt = self
            .hallucination_config
            .fact_check_prompt
            .replace("{}", response);

        // Add tool call context if present
        if !tool_calls.is_empty() {
            analysis_prompt.push_str("\n\nTool calls made: ");
            for tool_call in tool_calls {
                analysis_prompt
                    .push_str(&format!("{}({}), ", tool_call.name, tool_call.parameters));
            }
        }

        // Add conversation context if available
        if let Some(conversation_history) = context.get("conversation_history") {
            analysis_prompt.push_str(&format!(
                "\n\nConversation context: {}",
                conversation_history
            ));
        }

        let request = CompletionRequest {
            model: ModelId::new("gpt-3.5-turbo"), // Default model
            messages: vec![analysis_prompt],
            temperature: Some(0.1), // Low temperature for consistent analysis
            max_tokens: Some(300),
            tools: None,
        };

        match self.llm_provider.complete(request).await {
            Ok(response) => Ok(response.content),
            Err(e) => Err(PatinoxError::Validation(ValidationError::InvalidInput(
                format!("Hallucination detection failed: {}", e),
            ))),
        }
    }

    /// Determine if the LLM analysis indicates hallucination
    fn is_hallucination_detected(&self, analysis_result: &str) -> bool {
        let result_lower = analysis_result.to_lowercase();

        // Check for various indicators of inaccurate content
        result_lower.contains("inaccurate")
            || result_lower.contains("unsupported")
            || result_lower.contains("hallucination")
            || result_lower.contains("false")
            || result_lower.contains("incorrect")
            || result_lower.contains("misleading")
            || result_lower.contains("inconsistent")
    }

    /// Check if tool calls are consistent with the response message
    fn validate_tool_consistency(
        &self,
        _message: &str,
        tool_calls: &[crate::traits::tool::ToolCall],
        analysis_result: &str,
    ) -> bool {
        if tool_calls.is_empty() {
            return true; // No tool calls to validate
        }

        let analysis_lower = analysis_result.to_lowercase();
        analysis_lower.contains("consistent")
            || analysis_lower.contains("appropriate")
            || !analysis_lower.contains("inconsistent")
    }
}

#[async_trait]
impl Validator for HallucinationDetector {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &ValidatorConfig {
        &self.config
    }

    fn should_validate(&self, request: &ValidationRequest) -> bool {
        // Validate PostExecution and PreResponse stages, focusing on LLM responses
        self.config.stages.contains(&request.stage)
            && matches!(request.content, ValidationContent::LlmResponse { .. })
    }

    async fn validate(
        &self,
        request: ValidationRequest,
    ) -> Result<ValidationResponse, PatinoxError> {
        // Only analyze LLM responses
        let (message, tool_calls) = match &request.content {
            ValidationContent::LlmResponse {
                message,
                tool_calls,
            } => (message, tool_calls),
            _ => {
                // Approve non-LLM responses by default
                return Ok(ValidationResponse {
                    approved: true,
                    reason: Some("Non-LLM response content approved by default".to_string()),
                    modifications: None,
                    metadata: HashMap::new(),
                });
            }
        };

        // Analyze the response with the LLM
        let analysis_result = self
            .analyze_response(message, tool_calls, &request.context)
            .await?;

        // Check for hallucinations
        if self.is_hallucination_detected(&analysis_result) {
            return Ok(ValidationResponse {
                approved: false,
                reason: Some(format!(
                    "Potential hallucination detected: {}",
                    analysis_result
                )),
                modifications: None,
                metadata: HashMap::new(),
            });
        }

        // Check tool call consistency
        if !self.validate_tool_consistency(message, tool_calls, &analysis_result) {
            return Ok(ValidationResponse {
                approved: false,
                reason: Some("Tool calls are inconsistent with response content".to_string()),
                modifications: None,
                metadata: HashMap::new(),
            });
        }

        Ok(ValidationResponse {
            approved: true,
            reason: Some("Response appears accurate and well-grounded".to_string()),
            modifications: None,
            metadata: HashMap::new(),
        })
    }
}
