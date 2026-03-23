//! API key management for SpecGen CLI.
//!
//! This module handles reading, validation, and secure storage of the MiniMax API key.

use crate::error::SpecGenError;
use secrecy::{ExposeSecret, SecretString};
use std::env;

/// Minimum length for a valid API key.
const MIN_API_KEY_LENGTH: usize = 10;

/// Wrapper type for the MiniMax API key.
///
/// Uses the `secrecy` crate to prevent the key from being logged or printed.
#[derive(Debug, Clone)]
pub struct ApiKey(SecretString);

impl ApiKey {
    /// Create a new ApiKey from a raw string.
    pub fn new(key: String) -> Self {
        ApiKey(SecretString::from(key))
    }

    /// Get the API key as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.expose_secret()
    }

    /// Validate the API key format.
    ///
    /// Returns `Ok(())` if the key appears valid, or an error with details.
    pub fn validate(&self) -> Result<(), SpecGenError> {
        let key = self.as_str();

        if key.len() < MIN_API_KEY_LENGTH {
            return Err(SpecGenError::InvalidApiKey(format!(
                "API key is too short (minimum {MIN_API_KEY_LENGTH} characters)",
            )));
        }

        // Basic format check - MiniMax keys are typically alphanumeric with some special chars
        if !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(SpecGenError::InvalidApiKey(
                "API key contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }
}

impl TryFrom<String> for ApiKey {
    type Error = SpecGenError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let key = ApiKey::new(value);
        key.validate()?;
        Ok(key)
    }
}

impl TryFrom<&str> for ApiKey {
    type Error = SpecGenError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ApiKey::try_from(value.to_string())
    }
}

/// Read the API key from environment variables.
///
/// This is the primary entry point for obtaining the API key.
/// Returns an error if the environment variable is not set or is empty.
pub fn read_api_key_from_env() -> Result<ApiKey, SpecGenError> {
    let key = env::var("MINIMAX_API_KEY").map_err(|_| SpecGenError::MissingApiKey)?;

    if key.is_empty() {
        return Err(SpecGenError::InvalidApiKey("API key is empty".to_string()));
    }

    key.try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_validate_empty() {
        let key = ApiKey::new(String::new());
        assert!(key.validate().is_err());
    }

    #[test]
    fn test_api_key_validate_too_short() {
        let key = ApiKey::new("short".to_string());
        assert!(key.validate().is_err());
    }

    #[test]
    fn test_api_key_validate_valid() {
        let key = ApiKey::new("valid_api_key_12345".to_string());
        assert!(key.validate().is_ok());
    }

    #[test]
    fn test_api_key_validate_invalid_chars() {
        let key = ApiKey::new("key_with_$ymbols!".to_string());
        assert!(key.validate().is_err());
    }

    #[test]
    fn test_read_api_key_missing() {
        // Ensure MINIMAX_API_KEY is not set for this test
        env::remove_var("MINIMAX_API_KEY");
        let result = read_api_key_from_env();
        assert!(matches!(result, Err(SpecGenError::MissingApiKey)));
    }

    #[test]
    fn test_read_api_key_empty() {
        env::set_var("MINIMAX_API_KEY", "");
        let result = read_api_key_from_env();
        assert!(matches!(result, Err(SpecGenError::InvalidApiKey(_))));
        env::remove_var("MINIMAX_API_KEY");
    }

    #[test]
    fn test_read_api_key_success() {
        env::set_var("MINIMAX_API_KEY", "test_api_key_12345");
        let result = read_api_key_from_env();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "test_api_key_12345");
        env::remove_var("MINIMAX_API_KEY");
    }
}
