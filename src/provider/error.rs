//! Error types for the LLM provider abstraction layer

use crate::error::{
    ConfigurationError, ExecutionError, NetworkError, PatinoxError, ValidationError,
};
use std::time::Duration;
use thiserror::Error;

/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Errors that can occur when interacting with LLM providers
#[derive(Error, Debug, Clone)]
pub enum ProviderError {
    /// Error from the provider's API (HTTP errors, invalid responses, etc.)
    #[error("API error: {0}")]
    ApiError(String),

    /// Network connectivity issues
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Authentication/authorization failures
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Rate limiting from the provider
    #[error("Rate limited")]
    RateLimited {
        /// How long to wait before retrying
        retry_after: Option<Duration>,
    },

    /// The requested model is not available
    #[error("Model not available: {model}")]
    ModelNotAvailable { model: String },

    /// Invalid request parameters
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Provider timeout
    #[error("Request timeout after {duration:?}")]
    Timeout { duration: Duration },

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// All configured providers failed
    #[error("All providers failed")]
    AllProvidersFailed,

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Streaming-related errors
    #[error("Stream error: {0}")]
    StreamError(String),

    /// Response parsing errors
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Unknown provider error
    #[error("Unknown provider error: {0}")]
    Unknown(String),
}

impl ProviderError {
    /// Check if this error is retriable (can be retried with backoff)
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::NetworkError(_) => true,
            Self::RateLimited { .. } => true,
            Self::Timeout { .. } => true,
            Self::ApiError(msg) => {
                // Some API errors are retriable (5xx status codes)
                msg.contains("5") || msg.contains("timeout") || msg.contains("unavailable")
            }
            Self::AuthenticationError(_) => false,
            Self::ModelNotAvailable { .. } => false,
            Self::InvalidRequest(_) => false,
            Self::ConfigurationError(_) => false,
            Self::AllProvidersFailed => false,
            Self::SerializationError(_) => false,
            Self::StreamError(_) => true, // Streaming errors are often retriable
            Self::ParseError(_) => false, // Parse errors typically aren't retriable
            Self::Unknown(_) => false,
        }
    }

    /// Get the retry delay for rate limiting
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            Self::RateLimited { retry_after } => *retry_after,
            _ => None,
        }
    }

    /// Convert to a PatinoxError for integration with the core error system
    pub fn to_patinox_error(self) -> PatinoxError {
        match self {
            Self::ApiError(msg) => PatinoxError::Execution(ExecutionError::ToolExecutionFailed(
                "llm_provider".to_string(),
                format!("Provider API error: {}", msg),
            )),
            Self::NetworkError(msg) => PatinoxError::Network(NetworkError::Timeout(format!(
                "Provider network error: {}",
                msg
            ))),
            Self::AuthenticationError(msg) => PatinoxError::Configuration(
                ConfigurationError::MissingRequired(format!("Provider auth error: {}", msg)),
            ),
            Self::RateLimited { .. } => PatinoxError::Execution(ExecutionError::ResourceExhausted(
                "Provider rate limited".to_string(),
            )),
            Self::ModelNotAvailable { model } => PatinoxError::Configuration(
                ConfigurationError::InvalidFormat(format!("Model not available: {}", model)),
            ),
            Self::InvalidRequest(msg) => PatinoxError::Validation(ValidationError::InvalidInput(
                format!("Invalid provider request: {}", msg),
            )),
            Self::Timeout { .. } => PatinoxError::Network(NetworkError::Timeout(
                "Provider request timeout".to_string(),
            )),
            Self::ConfigurationError(msg) => PatinoxError::Configuration(
                ConfigurationError::InvalidFormat(format!("Provider config error: {}", msg)),
            ),
            Self::AllProvidersFailed => PatinoxError::Execution(ExecutionError::ResourceExhausted(
                "All LLM providers failed".to_string(),
            )),
            Self::SerializationError(msg) => {
                PatinoxError::Execution(ExecutionError::ToolExecutionFailed(
                    "serialization".to_string(),
                    format!("Provider serialization error: {}", msg),
                ))
            }
            Self::StreamError(msg) => PatinoxError::Network(NetworkError::Timeout(format!(
                "Provider stream error: {}",
                msg
            ))),
            Self::ParseError(msg) => PatinoxError::Execution(ExecutionError::ToolExecutionFailed(
                "parsing".to_string(),
                format!("Provider parse error: {}", msg),
            )),
            Self::Unknown(msg) => PatinoxError::Execution(ExecutionError::ToolExecutionFailed(
                "unknown".to_string(),
                format!("Unknown provider error: {}", msg),
            )),
        }
    }
}

impl From<ProviderError> for PatinoxError {
    fn from(error: ProviderError) -> Self {
        error.to_patinox_error()
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError(error.to_string())
    }
}

impl From<reqwest::Error> for ProviderError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::Timeout {
                duration: Duration::from_secs(30), // Default timeout
            }
        } else if error.is_connect() {
            Self::NetworkError(format!("Connection error: {}", error))
        } else if let Some(status) = error.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                Self::AuthenticationError("Invalid API credentials".to_string())
            } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                // Try to extract retry-after header
                Self::RateLimited { retry_after: None }
            } else if status.is_server_error() {
                Self::ApiError(format!("Server error: {}", status))
            } else if status.is_client_error() {
                Self::InvalidRequest(format!("Client error: {}", status))
            } else {
                Self::ApiError(format!("HTTP error: {}", status))
            }
        } else {
            Self::NetworkError(format!("Request error: {}", error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retriable_errors() {
        assert!(ProviderError::NetworkError("connection".to_string()).is_retriable());
        assert!(ProviderError::RateLimited { retry_after: None }.is_retriable());
        assert!(ProviderError::Timeout {
            duration: Duration::from_secs(30)
        }
        .is_retriable());

        assert!(!ProviderError::AuthenticationError("invalid".to_string()).is_retriable());
        assert!(!ProviderError::InvalidRequest("bad params".to_string()).is_retriable());
    }

    #[test]
    fn test_retry_delay() {
        let rate_limited = ProviderError::RateLimited {
            retry_after: Some(Duration::from_secs(60)),
        };
        assert_eq!(rate_limited.retry_delay(), Some(Duration::from_secs(60)));

        let api_error = ProviderError::ApiError("server error".to_string());
        assert_eq!(api_error.retry_delay(), None);
    }

    #[test]
    fn test_to_patinox_error() {
        let provider_error = ProviderError::ApiError("test error".to_string());
        let patinox_error = provider_error.to_patinox_error();

        match patinox_error {
            PatinoxError::Execution { .. } => {} // Expected
            _ => panic!("Expected execution error"),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());

        let provider_error: ProviderError = json_error.unwrap_err().into();
        match provider_error {
            ProviderError::SerializationError(_) => {} // Expected
            _ => panic!("Expected serialization error"),
        }
    }

    #[test]
    fn test_error_display() {
        let api_error = ProviderError::ApiError("test".to_string());
        assert_eq!(format!("{}", api_error), "API error: test");

        let auth_error = ProviderError::AuthenticationError("invalid key".to_string());
        assert_eq!(
            format!("{}", auth_error),
            "Authentication error: invalid key"
        );
    }
}
