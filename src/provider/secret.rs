//! Secure string handling for sensitive data like API keys

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A string that contains sensitive data and should be handled securely
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SecretString {
    #[zeroize(skip)]
    inner: String,
}

impl SecretString {
    /// Create a new SecretString from a string
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            inner: value.into(),
        }
    }

    /// Get the raw string value (use sparingly and securely)
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }

    /// Check if the secret is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the length of the secret
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretString")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl Deref for SecretString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.inner.as_bytes().ct_eq(other.inner.as_bytes()).into()
    }
}

impl Eq for SecretString {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_creation() {
        let secret = SecretString::new("my-secret-key");
        assert_eq!(secret.len(), 13);
        assert!(!secret.is_empty());
        assert_eq!(secret.expose_secret(), "my-secret-key");
    }

    #[test]
    fn test_secret_string_display() {
        let secret = SecretString::new("my-secret-key");
        assert_eq!(format!("{}", secret), "[REDACTED]");
        assert_eq!(format!("{:?}", secret), "SecretString { inner: \"[REDACTED]\" }");
    }

    #[test]
    fn test_secret_string_empty() {
        let secret = SecretString::new("");
        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
    }

    #[test]
    fn test_secret_string_from_string() {
        let secret: SecretString = "test-key".to_string().into();
        assert_eq!(secret.expose_secret(), "test-key");
    }

    #[test]
    fn test_secret_string_from_str() {
        let secret: SecretString = "test-key".into();
        assert_eq!(secret.expose_secret(), "test-key");
    }

    #[test]
    fn test_secret_string_equality() {
        let secret1 = SecretString::new("same-key");
        let secret2 = SecretString::new("same-key");
        let secret3 = SecretString::new("different-key");

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }
}