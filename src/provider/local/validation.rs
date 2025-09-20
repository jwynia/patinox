//! Validation utilities for local providers
//!
//! This module provides shared validation functions to ensure consistent
//! security and safety checks across all local providers. The validation
//! functions help prevent memory exhaustion attacks and maintain data integrity.

use crate::provider::{ProviderError, ProviderResult};

/// Maximum allowed size for a single streaming chunk (in characters)
/// This prevents memory exhaustion from extremely large responses
pub const MAX_CHUNK_SIZE: usize = 1024 * 1024; // 1MB in characters

/// Validates that content doesn't exceed maximum chunk size to prevent memory exhaustion
///
/// This function performs a critical security check to prevent memory exhaustion attacks
/// by limiting the size of individual chunks processed during streaming operations.
///
/// # Arguments
///
/// * `content` - The content string to validate
/// * `max_size` - Maximum allowed size in characters
///
/// # Returns
///
/// * `Ok(())` if content size is within limits
/// * `Err(ProviderError::ApiError)` if content exceeds the limit
///
/// # Example
///
/// ```rust
/// use patinox::provider::local::validation::{validate_chunk_size, MAX_CHUNK_SIZE};
///
/// let content = "Hello, world!";
/// validate_chunk_size(content, MAX_CHUNK_SIZE).unwrap();
///
/// let large_content = "x".repeat(MAX_CHUNK_SIZE + 1);
/// assert!(validate_chunk_size(&large_content, MAX_CHUNK_SIZE).is_err());
/// ```
pub fn validate_chunk_size(content: &str, max_size: usize) -> ProviderResult<()> {
    if content.len() > max_size {
        Err(ProviderError::ApiError(format!(
            "Chunk size ({} chars) exceeds limit ({} chars)",
            content.len(),
            max_size
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_chunk_size_within_limit() {
        let content = "Hello, world!";
        let result = validate_chunk_size(content, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chunk_size_exactly_at_limit() {
        let content = "x".repeat(100);
        let result = validate_chunk_size(&content, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chunk_size_exceeds_limit() {
        let content = "x".repeat(101);
        let result = validate_chunk_size(&content, 100);
        assert!(result.is_err());

        if let Err(ProviderError::ApiError(msg)) = result {
            assert!(msg.contains("Chunk size (101 chars) exceeds limit (100 chars)"));
        } else {
            panic!("Expected ApiError with specific message");
        }
    }

    #[test]
    fn test_validate_chunk_size_empty_content() {
        let content = "";
        let result = validate_chunk_size(content, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chunk_size_zero_limit() {
        let content = "x";
        let result = validate_chunk_size(content, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chunk_size_with_default_max() {
        let content = "x".repeat(MAX_CHUNK_SIZE);
        let result = validate_chunk_size(&content, MAX_CHUNK_SIZE);
        assert!(result.is_ok());

        let large_content = "x".repeat(MAX_CHUNK_SIZE + 1);
        let result = validate_chunk_size(&large_content, MAX_CHUNK_SIZE);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_message_format() {
        let content = "x".repeat(150);
        let result = validate_chunk_size(&content, 100);

        if let Err(ProviderError::ApiError(msg)) = result {
            assert_eq!(msg, "Chunk size (150 chars) exceeds limit (100 chars)");
        } else {
            panic!("Expected specific error message format");
        }
    }
}