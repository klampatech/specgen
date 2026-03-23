//! Error types for SpecGen CLI.
//!
//! This module provides a comprehensive error type hierarchy using `thiserror`
//! for typed error handling throughout the application.

use thiserror::Error;

/// Main error type for SpecGen CLI operations.
///
/// All errors in the application are typed and structured, providing clear
/// error codes and messages for debugging and user feedback.
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum SpecGenError {
    /// API key is missing from environment.
    #[error("MINIMAX_API_KEY environment variable is not set")]
    MissingApiKey,

    /// API key format is invalid.
    #[error("API key format is invalid: {0}")]
    InvalidApiKey(String),

    /// Network request failed.
    #[error("Network request failed: {0}")]
    NetworkError(String),

    /// HTTP request returned an error status.
    #[error("HTTP error: {0} - {1}")]
    HttpError(u16, String),

    /// Rate limit exceeded, retry after specified duration.
    #[error("Rate limit exceeded, retry after {0:?}")]
    RateLimited(std::time::Duration),

    /// Invalid response from API.
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    /// Stream parsing error.
    #[error("Stream parsing error: {0}")]
    StreamError(String),

    /// Interview validation error.
    #[error("Interview validation error: {0}")]
    InterviewError(String),

    /// Domain detection error.
    #[error("Domain detection error: {0}")]
    DomainError(String),

    /// File I/O error.
    #[error("File I/O error: {0}")]
    IoError(String),

    /// Session error.
    #[error("Session error: {0}")]
    SessionError(String),

    /// Spec generation error.
    #[error("Spec generation error: {0}")]
    SpecError(String),

    /// Diff error.
    #[error("Diff error: {0}")]
    DiffError(String),

    /// Merge error.
    #[error("Merge error: {0}")]
    MergeError(String),

    /// UI error.
    #[error("UI error: {0}")]
    UiError(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Unexpected error.
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl From<std::io::Error> for SpecGenError {
    fn from(err: std::io::Error) -> Self {
        SpecGenError::IoError(err.to_string())
    }
}

impl From<reqwest::Error> for SpecGenError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_connect() {
            SpecGenError::NetworkError(format!("Connection failed: {err}"))
        } else if err.is_timeout() {
            SpecGenError::NetworkError(format!("Request timed out: {err}"))
        } else {
            SpecGenError::NetworkError(err.to_string())
        }
    }
}
