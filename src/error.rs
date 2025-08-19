//! Core error types for the Patinox framework
//!
//! This module defines the foundational error types that all other components
//! build upon. The error system emphasizes recoverability, context preservation,
//! and clear recovery strategies.

// Tests written FIRST following TDD principles
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::error::Error;

    // Test 1: Basic error type structure and traits
    #[test]
    fn patinox_error_implements_standard_traits() {
        let error = PatinoxError::Validation(ValidationError::InvalidInput("test".to_string()));

        // Must implement standard error traits
        assert!(
            !format!("{}", error).is_empty(),
            "Error must have Display implementation"
        );
        assert!(
            !format!("{:?}", error).is_empty(),
            "Error must have Debug implementation"
        );

        // Should be Send + Sync for multi-threading
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<PatinoxError>();
        assert_sync::<PatinoxError>();
    }

    #[test]
    fn validation_error_basic_functionality() {
        let error = ValidationError::InvalidInput("test input".to_string());
        assert!(!format!("{}", error).is_empty());
        assert!(!format!("{:?}", error).is_empty());
    }

    #[test]
    fn execution_error_basic_functionality() {
        let error =
            ExecutionError::ToolExecutionFailed("tool_name".to_string(), "reason".to_string());
        assert!(!format!("{}", error).is_empty());
        assert!(!format!("{:?}", error).is_empty());
    }

    #[test]
    fn network_error_basic_functionality() {
        let error = NetworkError::Timeout("test timeout".to_string());
        assert!(!format!("{}", error).is_empty());
        assert!(!format!("{:?}", error).is_empty());
    }

    #[test]
    fn configuration_error_basic_functionality() {
        let inner_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = ConfigurationError::FileNotFound("test.toml".to_string(), inner_error);
        assert!(!format!("{}", error).is_empty());
        assert!(!format!("{:?}", error).is_empty());
    }

    // Test 2: Error chain preservation
    #[test]
    fn error_chain_preservation() {
        let inner_error =
            std::io::Error::new(std::io::ErrorKind::NotFound, "config file not found");
        let config_error =
            ConfigurationError::FileNotFound("patinox.toml".to_string(), inner_error);
        let patinox_error = PatinoxError::Configuration(config_error);

        // Error chain should be preserved through all levels
        assert!(
            patinox_error.source().is_some(),
            "PatinoxError should have source"
        );
        let config_source = patinox_error.source().unwrap();
        assert!(
            config_source.source().is_some(),
            "ConfigurationError should have source"
        );
    }

    #[test]
    fn error_context_in_messages() {
        let inner_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let config_error = ConfigurationError::FileNotFound("config.toml".to_string(), inner_error);
        let patinox_error = PatinoxError::Configuration(config_error);

        let error_message = format!("{}", patinox_error);
        assert!(
            error_message.contains("config.toml"),
            "Error message should contain filename"
        );
    }

    // Test 3: Recovery strategy system
    #[test]
    fn recovery_strategy_validation_errors() {
        let error = PatinoxError::Validation(ValidationError::InvalidInput("test".to_string()));
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Fail);

        let error = PatinoxError::Validation(ValidationError::RateLimited);
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Retry);
    }

    #[test]
    fn recovery_strategy_execution_errors() {
        let error = PatinoxError::Execution(ExecutionError::ToolExecutionFailed(
            "tool".to_string(),
            "reason".to_string(),
        ));
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Fallback);

        let error = PatinoxError::Execution(ExecutionError::AgentStateMismatch(
            "expected".to_string(),
            "actual".to_string(),
        ));
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Fail);
    }

    #[test]
    fn recovery_strategy_network_errors() {
        let error = PatinoxError::Network(NetworkError::Timeout("API timeout".to_string()));
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Retry);

        let error = PatinoxError::Network(NetworkError::RateLimited("API rate limit".to_string()));
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::CircuitBreak);

        let error =
            PatinoxError::Network(NetworkError::Unauthorized("Invalid API key".to_string()));
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Fail);
    }

    #[test]
    fn recovery_strategy_configuration_errors() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = PatinoxError::Configuration(ConfigurationError::FileNotFound(
            "config.toml".to_string(),
            inner,
        ));
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Fail);

        let error =
            PatinoxError::Configuration(ConfigurationError::InvalidFormat("bad TOML".to_string()));
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Fail);
    }

    // Test 4: Recovery strategy exhaustiveness (every error variant must have a strategy)
    #[test]
    fn recovery_strategy_exhaustive() {
        let validation_errors = [
            PatinoxError::Validation(ValidationError::InvalidInput("test".to_string())),
            PatinoxError::Validation(ValidationError::RateLimited),
            PatinoxError::Validation(ValidationError::ValidationTimeout("validator".to_string())),
        ];

        let execution_errors = [
            PatinoxError::Execution(ExecutionError::ToolExecutionFailed(
                "tool".to_string(),
                "reason".to_string(),
            )),
            PatinoxError::Execution(ExecutionError::AgentStateMismatch(
                "expected".to_string(),
                "actual".to_string(),
            )),
            PatinoxError::Execution(ExecutionError::ResourceExhausted("memory".to_string())),
        ];

        let network_errors = [
            PatinoxError::Network(NetworkError::Timeout("test".to_string())),
            PatinoxError::Network(NetworkError::RateLimited("test".to_string())),
            PatinoxError::Network(NetworkError::Unauthorized("test".to_string())),
            PatinoxError::Network(NetworkError::ServiceUnavailable("test".to_string())),
        ];

        let config_errors = [
            PatinoxError::Configuration(ConfigurationError::FileNotFound(
                "test".to_string(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
            )),
            PatinoxError::Configuration(ConfigurationError::InvalidFormat("test".to_string())),
            PatinoxError::Configuration(ConfigurationError::MissingRequired("test".to_string())),
        ];

        // Every error variant must have a valid recovery strategy
        for error in validation_errors
            .iter()
            .chain(execution_errors.iter())
            .chain(network_errors.iter())
            .chain(config_errors.iter())
        {
            let strategy = error.recovery_strategy();
            assert!(
                matches!(
                    strategy,
                    RecoveryStrategy::Retry
                        | RecoveryStrategy::Fallback
                        | RecoveryStrategy::CircuitBreak
                        | RecoveryStrategy::Fail
                ),
                "Error {:?} must have valid recovery strategy, got {:?}",
                error,
                strategy
            );
        }
    }

    // Test 5: Error conversion and integration
    #[test]
    fn error_conversions_from_std() {
        // Test conversion from std::io::Error
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let patinox_error: PatinoxError = io_error.into();
        assert!(matches!(patinox_error, PatinoxError::Configuration(_)));
    }

    #[test]
    fn error_anyhow_integration() {
        let patinox_error =
            PatinoxError::Validation(ValidationError::InvalidInput("test".to_string()));
        // Test that PatinoxError can be converted to anyhow::Error (via blanket impl)
        let anyhow_error = anyhow::Error::from(patinox_error);
        assert!(!format!("{}", anyhow_error).is_empty());

        // Test that we can use ? operator with anyhow
        fn test_function() -> anyhow::Result<()> {
            let _result: Result<(), PatinoxError> = Err(PatinoxError::Validation(
                ValidationError::InvalidInput("test".to_string()),
            ));
            _result?;
            Ok(())
        }

        assert!(test_function().is_err());
    }

    // Test 6: Property-based tests
    proptest! {
        #[test]
        fn recovery_strategy_never_panics(
            error_type in prop::sample::select(vec!["validation", "execution", "network", "configuration"])
        ) {
            let error = match error_type {
                "validation" => PatinoxError::Validation(ValidationError::InvalidInput("test".to_string())),
                "execution" => PatinoxError::Execution(ExecutionError::ToolExecutionFailed("tool".to_string(), "reason".to_string())),
                "network" => PatinoxError::Network(NetworkError::Timeout("test".to_string())),
                "configuration" => PatinoxError::Configuration(ConfigurationError::InvalidFormat("test".to_string())),
                _ => unreachable!(),
            };

            // recovery_strategy() should never panic
            let _strategy = error.recovery_strategy();
        }

        #[test]
        fn error_display_never_empty(
            message in ".*"
        ) {
            let error = PatinoxError::Validation(ValidationError::InvalidInput(message));
            let display_output = format!("{}", error);
            assert!(!display_output.is_empty());
        }

        #[test]
        fn error_debug_never_empty(
            message in ".*"
        ) {
            let error = PatinoxError::Network(NetworkError::Timeout(message));
            let debug_output = format!("{:?}", error);
            assert!(!debug_output.is_empty());
        }
    }

    // Test 7: Recovery strategy enum tests
    #[test]
    fn recovery_strategy_enum_completeness() {
        use RecoveryStrategy::*;

        // Test all variants exist and are accessible
        let strategies = vec![Retry, Fallback, CircuitBreak, Fail];

        for strategy in strategies {
            // Should be able to debug print
            let _debug = format!("{:?}", strategy);

            // Should be able to compare
            assert!(strategy == strategy);
        }
    }

    #[test]
    fn recovery_strategy_copy_clone() {
        let strategy = RecoveryStrategy::Retry;
        let copied = strategy;
        let cloned = strategy;

        assert_eq!(strategy, copied);
        assert_eq!(strategy, cloned);
    }
}

// Implementation to make tests pass (Green phase)

/// Recovery strategy for handling errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation after a delay
    Retry,
    /// Try an alternative approach
    Fallback,
    /// Stop trying and activate circuit breaker
    CircuitBreak,
    /// Fail immediately with no recovery
    Fail,
}

/// Core error type for all Patinox operations
#[derive(thiserror::Error, Debug)]
pub enum PatinoxError {
    /// Validation-related errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Execution-related errors  
    #[error("Execution error: {0}")]
    Execution(#[from] ExecutionError),

    /// Network/API-related errors
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
}

/// Validation error variants
#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    /// Invalid input provided to validator
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Validation was rate limited
    #[error("Validation rate limited")]
    RateLimited,

    /// Validator timed out
    #[error("Validation timeout for validator: {0}")]
    ValidationTimeout(String),
}

/// Execution error variants
#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
    /// Tool execution failed
    #[error("Tool '{0}' execution failed: {1}")]
    ToolExecutionFailed(String, String),

    /// Agent state mismatch
    #[error("Agent state mismatch: expected '{0}', found '{1}'")]
    AgentStateMismatch(String, String),

    /// Resource exhausted
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
}

/// Network error variants  
#[derive(thiserror::Error, Debug)]
pub enum NetworkError {
    /// Request timeout
    #[error("Network timeout: {0}")]
    Timeout(String),

    /// Rate limited by API
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// Authentication/authorization failed
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Configuration error variants
#[derive(thiserror::Error, Debug)]
pub enum ConfigurationError {
    /// Configuration file not found
    #[error("Configuration file '{0}' not found")]
    FileNotFound(String, #[source] std::io::Error),

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),

    /// Required configuration missing
    #[error("Required configuration missing: {0}")]
    MissingRequired(String),
}

impl PatinoxError {
    /// Returns the appropriate recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            PatinoxError::Validation(validation_error) => match validation_error {
                ValidationError::InvalidInput(_) => RecoveryStrategy::Fail,
                ValidationError::RateLimited => RecoveryStrategy::Retry,
                ValidationError::ValidationTimeout(_) => RecoveryStrategy::Retry,
            },

            PatinoxError::Execution(execution_error) => match execution_error {
                ExecutionError::ToolExecutionFailed(_, _) => RecoveryStrategy::Fallback,
                ExecutionError::AgentStateMismatch(_, _) => RecoveryStrategy::Fail,
                ExecutionError::ResourceExhausted(_) => RecoveryStrategy::Retry,
            },

            PatinoxError::Network(network_error) => match network_error {
                NetworkError::Timeout(_) => RecoveryStrategy::Retry,
                NetworkError::RateLimited(_) => RecoveryStrategy::CircuitBreak,
                NetworkError::Unauthorized(_) => RecoveryStrategy::Fail,
                NetworkError::ServiceUnavailable(_) => RecoveryStrategy::Retry,
            },

            PatinoxError::Configuration(config_error) => match config_error {
                ConfigurationError::FileNotFound(_, _) => RecoveryStrategy::Fail,
                ConfigurationError::InvalidFormat(_) => RecoveryStrategy::Fail,
                ConfigurationError::MissingRequired(_) => RecoveryStrategy::Fail,
            },
        }
    }
}

// Standard conversions
impl From<std::io::Error> for PatinoxError {
    fn from(error: std::io::Error) -> Self {
        PatinoxError::Configuration(ConfigurationError::FileNotFound(
            "unknown".to_string(),
            error,
        ))
    }
}

// Note: anyhow::Error already provides From<E> for any Error type
