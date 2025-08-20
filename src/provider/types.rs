//! Core types for the LLM provider abstraction layer

use serde::{Deserialize, Serialize};

/// Unique identifier for a language model
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId {
    /// The model name (e.g., "claude-3-opus", "gpt-4-turbo")
    name: String,
    /// Optional provider hint for routing (e.g., "anthropic", "openai")
    provider_hint: Option<String>,
}

impl ModelId {
    /// Create a new model ID
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            provider_hint: None,
        }
    }

    /// Add a provider hint for routing
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider_hint = Some(provider.into());
        self
    }

    /// Get the model name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the provider hint if set
    pub fn provider_hint(&self) -> Option<&str> {
        self.provider_hint.as_deref()
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(provider) = &self.provider_hint {
            write!(f, "{}/{}", provider, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// Speed tier for model performance categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SpeedTier {
    /// Ultra-fast models (< 500ms typical response)
    Instant,
    /// Fast models (< 2s typical response)
    Fast,
    /// Standard speed models (< 5s typical response)
    Standard,
    /// Slower, higher-quality models (> 5s typical response)
    Slow,
}

/// Quality tier for model output categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QualityTier {
    /// Fast, cheap, lower quality models
    Lite,
    /// Balanced performance and cost
    Standard,
    /// High quality, slower, more expensive
    Premium,
    /// Best available quality (e.g., GPT-4, Claude Opus)
    Ultra,
}

/// Model capabilities that influence routing and usage decisions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Maximum context length in tokens
    pub max_tokens: usize,
    /// Whether the model supports function/tool calling
    pub supports_tools: bool,
    /// Whether the model supports vision/image inputs
    pub supports_vision: bool,
    /// Whether the model supports streaming responses
    pub supports_streaming: bool,
    /// Cost per 1K input tokens (if known)
    pub input_cost_per_1k: Option<f64>,
    /// Cost per 1K output tokens (if known)
    pub output_cost_per_1k: Option<f64>,
    /// Performance speed tier
    pub speed_tier: SpeedTier,
    /// Output quality tier
    pub quality_tier: QualityTier,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            supports_tools: false,
            supports_vision: false,
            supports_streaming: false,
            input_cost_per_1k: None,
            output_cost_per_1k: None,
            speed_tier: SpeedTier::Standard,
            quality_tier: QualityTier::Standard,
        }
    }
}

/// Information about an available model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique identifier for this model
    pub id: ModelId,
    /// Human-readable name
    pub name: String,
    /// Model capabilities
    pub capabilities: ModelCapabilities,
}

/// A tool/function call definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the function to call
    pub name: String,
    /// Arguments to pass to the function (JSON string)
    pub arguments: String,
    /// Optional ID for tracking this tool call
    pub id: Option<String>,
}

/// A message in a conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionMessage {
    /// Role of the message sender
    pub role: String,
    /// Text content of the message
    pub content: String,
    /// Optional tool calls in this message
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl CompletionMessage {
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }
}

/// Tool definition for function calling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// Type of tool (usually "function")
    pub r#type: String,
    /// Function definition
    pub function: ToolFunction,
}

/// Function definition for tool calling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolFunction {
    /// Name of the function
    pub name: String,
    /// Description of what the function does
    pub description: String,
    /// JSON schema for the parameters
    pub parameters: serde_json::Value,
}

/// Request for a text completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use for completion
    pub model: ModelId,
    /// Conversation messages
    pub messages: Vec<String>, // Simplified for now - will expand to CompletionMessage later
    /// Sampling temperature (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,
    /// Available tools for function calling
    pub tools: Option<Vec<Tool>>,
}

/// Usage statistics for a completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt
    pub prompt_tokens: usize,
    /// Number of tokens in the completion
    pub completion_tokens: usize,
    /// Total tokens used (prompt + completion)
    pub total_tokens: usize,
}

/// Response from a text completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Model that generated this response
    pub model: ModelId,
    /// Generated text content
    pub content: String,
    /// Token usage information
    pub usage: Option<Usage>,
    /// Reason the completion finished
    pub finish_reason: String,
}

/// Request for generating embeddings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Model to use for embedding
    pub model: ModelId,
    /// Text inputs to embed
    pub input: Vec<String>,
}

/// Response containing embeddings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Generated embeddings (one per input text)
    pub embeddings: Vec<Vec<f64>>,
    /// Model that generated the embeddings
    pub model: ModelId,
    /// Token usage information
    pub usage: Option<Usage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_id_creation() {
        let model = ModelId::new("gpt-4");
        assert_eq!(model.name(), "gpt-4");
        assert!(model.provider_hint().is_none());
    }

    #[test]
    fn test_model_id_with_provider() {
        let model = ModelId::new("claude-3-opus").with_provider("anthropic");
        assert_eq!(model.name(), "claude-3-opus");
        assert_eq!(model.provider_hint(), Some("anthropic"));
    }

    #[test]
    fn test_model_id_display() {
        let model_no_provider = ModelId::new("gpt-4");
        assert_eq!(format!("{}", model_no_provider), "gpt-4");

        let model_with_provider = ModelId::new("claude-3-opus").with_provider("anthropic");
        assert_eq!(
            format!("{}", model_with_provider),
            "anthropic/claude-3-opus"
        );
    }

    #[test]
    fn test_speed_tier_ordering() {
        assert!(SpeedTier::Instant < SpeedTier::Fast);
        assert!(SpeedTier::Fast < SpeedTier::Standard);
        assert!(SpeedTier::Standard < SpeedTier::Slow);
    }

    #[test]
    fn test_quality_tier_ordering() {
        assert!(QualityTier::Lite < QualityTier::Standard);
        assert!(QualityTier::Standard < QualityTier::Premium);
        assert!(QualityTier::Premium < QualityTier::Ultra);
    }

    #[test]
    fn test_model_capabilities_default() {
        let caps = ModelCapabilities::default();
        assert_eq!(caps.max_tokens, 4096);
        assert!(!caps.supports_tools);
        assert!(!caps.supports_vision);
        assert_eq!(caps.speed_tier, SpeedTier::Standard);
        assert_eq!(caps.quality_tier, QualityTier::Standard);
    }

    #[test]
    fn test_completion_message_constructors() {
        let user_msg = CompletionMessage::user("Hello");
        assert_eq!(user_msg.role, "user");
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = CompletionMessage::assistant("Hi there");
        assert_eq!(assistant_msg.role, "assistant");
        assert_eq!(assistant_msg.content, "Hi there");

        let system_msg = CompletionMessage::system("You are helpful");
        assert_eq!(system_msg.role, "system");
        assert_eq!(system_msg.content, "You are helpful");
    }
}
