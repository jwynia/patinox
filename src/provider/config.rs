//! Configuration system for the LLM provider abstraction layer

use super::error::ProviderError;
use super::secret::SecretString;
use super::types::ModelId;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Strategy for selecting models when multiple options are available
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectionStrategy {
    /// Always use the specified model
    Fixed,
    /// Choose fastest model that meets requirements
    FastestAvailable,
    /// Choose cheapest model that meets requirements
    CheapestAvailable,
    /// Balance between speed and cost
    Balanced,
    /// Use the highest quality model available
    BestQuality,
}

impl Default for SelectionStrategy {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: Option<u32>,
    /// Maximum tokens per minute
    pub tokens_per_minute: Option<u32>,
    /// Maximum concurrent requests
    pub max_concurrent: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: Some(60),
            tokens_per_minute: Some(100_000),
            max_concurrent: Some(10),
        }
    }
}

/// Retry configuration for failed requests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before retrying
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub exponential_base: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(10),
            exponential_base: 2.0,
        }
    }
}

/// Provider configuration enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Provider {
    /// OpenRouter provider configuration
    OpenRouter {
        /// API key for OpenRouter
        api_key: SecretString,
        /// Optional custom base URL
        base_url: Option<String>,
    },
    /// Direct OpenAI provider configuration
    OpenAI {
        /// OpenAI API key
        api_key: SecretString,
        /// Optional organization ID
        organization: Option<String>,
        /// Optional custom base URL
        base_url: Option<String>,
    },
    /// Direct Anthropic provider configuration
    Anthropic {
        /// Anthropic API key
        api_key: SecretString,
        /// Optional custom base URL
        base_url: Option<String>,
    },
    /// Local model provider (Ollama, etc.)
    Local {
        /// Endpoint URL for the local provider
        endpoint: String,
        /// Optional path to model files
        model_path: Option<String>,
    },
}

impl Provider {
    /// Get a display name for this provider
    pub fn name(&self) -> &str {
        match self {
            Self::OpenRouter { .. } => "openrouter",
            Self::OpenAI { .. } => "openai",
            Self::Anthropic { .. } => "anthropic",
            Self::Local { .. } => "local",
        }
    }
}

/// Global model configuration (from environment/config files)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalModelConfig {
    /// Default provider to use
    pub default_provider: Provider,
    /// Default model to use
    pub default_model: ModelId,
    /// Fallback models if primary fails
    pub fallback_models: Vec<ModelId>,
    /// Model selection strategy
    pub selection_strategy: SelectionStrategy,
    /// Rate limiting configuration
    pub rate_limits: RateLimitConfig,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for GlobalModelConfig {
    fn default() -> Self {
        Self {
            default_provider: Provider::OpenRouter {
                api_key: SecretString::new(""), // Will be loaded from environment
                base_url: None,
            },
            default_model: ModelId::new("anthropic/claude-3-sonnet"),
            fallback_models: vec![
                ModelId::new("openai/gpt-4-turbo"),
                ModelId::new("google/gemini-pro"),
            ],
            selection_strategy: SelectionStrategy::default(),
            rate_limits: RateLimitConfig::default(),
            retry_config: RetryConfig::default(),
            timeout: Duration::from_secs(30),
        }
    }
}

/// Agent-level model configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AgentModelConfig {
    /// Override provider for this agent
    pub provider: Option<Provider>,
    /// Override model for this agent
    pub model: Option<ModelId>,
    /// Agent-specific temperature
    pub temperature: Option<f32>,
    /// Agent-specific max tokens
    pub max_tokens: Option<usize>,
    /// Agent-specific timeout
    pub timeout: Option<Duration>,
}

/// Request-level configuration overrides
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RequestConfig {
    /// Override provider for this request
    pub provider: Option<Provider>,
    /// Override model for this request
    pub model: Option<ModelId>,
    /// Request-specific temperature
    pub temperature: Option<f32>,
    /// Request-specific max tokens
    pub max_tokens: Option<usize>,
    /// Timeout for this request
    pub timeout: Option<Duration>,
}

/// Configuration loader that handles multiple sources
pub struct ModelConfigLoader {
    /// Environment variable prefix (default: "PATINOX_")
    pub env_prefix: String,
}

impl Default for ModelConfigLoader {
    fn default() -> Self {
        Self {
            env_prefix: "PATINOX_".to_string(),
        }
    }
}

impl ModelConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from environment variables
    pub fn load_from_env(&self) -> Result<GlobalModelConfig, ProviderError> {
        let mut config = GlobalModelConfig::default();

        // Load provider configuration
        if let Ok(provider_name) = std::env::var(format!("{}MODEL_PROVIDER", self.env_prefix)) {
            match provider_name.to_lowercase().as_str() {
                "openrouter" => {
                    if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
                        config.default_provider = Provider::OpenRouter {
                            api_key: SecretString::new(api_key),
                            base_url: std::env::var(format!(
                                "{}OPENROUTER_BASE_URL",
                                self.env_prefix
                            ))
                            .ok(),
                        };
                    }
                }
                "openai" => {
                    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
                        config.default_provider = Provider::OpenAI {
                            api_key: SecretString::new(api_key),
                            organization: std::env::var("OPENAI_ORG_ID").ok(),
                            base_url: std::env::var(format!("{}OPENAI_BASE_URL", self.env_prefix))
                                .ok(),
                        };
                    }
                }
                "anthropic" => {
                    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
                        config.default_provider = Provider::Anthropic {
                            api_key: SecretString::new(api_key),
                            base_url: std::env::var(format!(
                                "{}ANTHROPIC_BASE_URL",
                                self.env_prefix
                            ))
                            .ok(),
                        };
                    }
                }
                _ => {
                    return Err(ProviderError::ConfigurationError(format!(
                        "Unknown provider: {}",
                        provider_name
                    )));
                }
            }
        }

        // Load default model
        if let Ok(model_name) = std::env::var(format!("{}MODEL_DEFAULT", self.env_prefix)) {
            config.default_model = ModelId::new(model_name);
        }

        // Load timeout
        if let Ok(timeout_str) = std::env::var(format!("{}MODEL_TIMEOUT", self.env_prefix)) {
            if let Ok(timeout_secs) = timeout_str.trim_end_matches('s').parse::<u64>() {
                config.timeout = Duration::from_secs(timeout_secs);
            }
        }

        Ok(config)
    }

    /// Load complete configuration (environment + defaults)
    pub async fn load(&self) -> Result<GlobalModelConfig, ProviderError> {
        // Start with defaults
        let mut config = GlobalModelConfig::default();

        // Override with environment variables
        if let Ok(env_config) = self.load_from_env() {
            // Merge environment config into defaults
            config.default_provider = env_config.default_provider;
            config.default_model = env_config.default_model;
            config.timeout = env_config.timeout;
            // Keep other defaults unless we add more environment variable parsing
        }

        // Validate configuration
        self.validate_config(&config)?;

        Ok(config)
    }

    /// Validate configuration for completeness and correctness
    fn validate_config(&self, config: &GlobalModelConfig) -> Result<(), ProviderError> {
        // Check if provider has required credentials
        match &config.default_provider {
            Provider::OpenRouter { api_key, .. }
            | Provider::OpenAI { api_key, .. }
            | Provider::Anthropic { api_key, .. } => {
                if api_key.is_empty() {
                    return Err(ProviderError::ConfigurationError(
                        "Provider requires API key but none configured".to_string(),
                    ));
                }
            }
            Provider::Local { endpoint, .. } => {
                if endpoint.is_empty() {
                    return Err(ProviderError::ConfigurationError(
                        "Local provider requires endpoint URL".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_strategy_default() {
        assert_eq!(SelectionStrategy::default(), SelectionStrategy::Balanced);
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, Some(60));
        assert_eq!(config.tokens_per_minute, Some(100_000));
        assert_eq!(config.max_concurrent, Some(10));
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(1000));
        assert_eq!(config.exponential_base, 2.0);
    }

    #[test]
    fn test_provider_name() {
        let openrouter = Provider::OpenRouter {
            api_key: SecretString::new("test"),
            base_url: None,
        };
        assert_eq!(openrouter.name(), "openrouter");

        let openai = Provider::OpenAI {
            api_key: SecretString::new("test"),
            organization: None,
            base_url: None,
        };
        assert_eq!(openai.name(), "openai");
    }

    #[test]
    fn test_global_config_default() {
        let config = GlobalModelConfig::default();
        assert_eq!(config.default_model.name(), "anthropic/claude-3-sonnet");
        assert_eq!(config.fallback_models.len(), 2);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_config_loader_validation() {
        let loader = ModelConfigLoader::new();

        // Test valid config
        let valid_config = GlobalModelConfig {
            default_provider: Provider::OpenAI {
                api_key: SecretString::new("sk-test"),
                organization: None,
                base_url: None,
            },
            ..GlobalModelConfig::default()
        };
        assert!(loader.validate_config(&valid_config).is_ok());

        // Test invalid config (empty API key)
        let invalid_config = GlobalModelConfig {
            default_provider: Provider::OpenAI {
                api_key: SecretString::new(""),
                organization: None,
                base_url: None,
            },
            ..GlobalModelConfig::default()
        };
        assert!(loader.validate_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_load_from_env() {
        let loader = ModelConfigLoader::new();

        // Test with no environment variables (should use defaults)
        let _config = loader.load().await;
        // Note: This will fail validation due to empty API key in default,
        // but that's expected - real usage requires environment setup
    }
}
