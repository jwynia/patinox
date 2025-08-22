//! Error types for local model providers

use crate::provider::ProviderError;
use std::time::Duration;
use thiserror::Error;

/// Result type for local provider operations
pub type LocalProviderResult<T> = Result<T, LocalProviderError>;

/// Errors specific to local model providers
#[derive(Debug, Error)]
pub enum LocalProviderError {
    /// Service is not available or reachable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    /// Service was not found during discovery
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    /// Model not found on any available service
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Model exists but is not loaded
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),
    
    /// Service operation timed out
    #[error("Service timeout after {0:?}")]
    ServiceTimeout(Duration),
    
    /// Invalid configuration provided
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Network error occurred
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    /// Service returned an error
    #[error("Service error: {0}")]
    ServiceError(String),
    
    /// Failed to parse response
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<LocalProviderError> for ProviderError {
    fn from(err: LocalProviderError) -> Self {
        match err {
            LocalProviderError::ServiceUnavailable(msg) => 
                ProviderError::NetworkError(format!("Local service unavailable: {}", msg)),
            LocalProviderError::ServiceNotFound(msg) => 
                ProviderError::ConfigurationError(format!("Service not found: {}", msg)),
            LocalProviderError::ModelNotFound(msg) => 
                ProviderError::ModelNotAvailable { model: msg },
            LocalProviderError::ModelNotLoaded(msg) => 
                ProviderError::ApiError(format!("Model not loaded: {}", msg)),
            LocalProviderError::ServiceTimeout(duration) => 
                ProviderError::NetworkError(format!("Service timeout after {:?}", duration)),
            LocalProviderError::InvalidConfiguration(msg) => 
                ProviderError::ConfigurationError(msg),
            LocalProviderError::NetworkError(err) => 
                ProviderError::NetworkError(err.to_string()),
            LocalProviderError::ServiceError(msg) => 
                ProviderError::ApiError(format!("Service error: {}", msg)),
            LocalProviderError::ParseError(msg) => 
                ProviderError::SerializationError(format!("Parse error: {}", msg)),
        }
    }
}