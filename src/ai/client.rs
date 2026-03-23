//! MiniMax API client implementation.

use crate::ai::models::{ChatRequest, Message};
use crate::ai::streaming::parse_sse_stream;
use crate::api_key::ApiKey;
use crate::error::SpecGenError;
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

/// Base URL for MiniMax API.
#[allow(dead_code)]
const MINIMAX_BASE_URL: &str = "https://api.minimax.io";

/// Maximum retries for retryable errors.
const MAX_RETRIES: u32 = 3;

/// Initial backoff duration for rate limit (429).
const RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(1);

/// Initial backoff duration for server errors (5xx).
const SERVER_ERROR_BACKOFF: Duration = Duration::from_millis(500);

/// Request timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Connection timeout.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// AI Client trait for abstraction and testing.
#[async_trait]
#[allow(dead_code)]
pub trait AiClient: Send + Sync {
    /// Send a chat request and get a streaming response.
    async fn chat(&self, request: ChatRequest) -> Result<String, SpecGenError>;

    /// Validate the API key by making a test request.
    async fn validate_api_key(&self, api_key: &ApiKey) -> Result<(), SpecGenError>;
}

/// MiniMax API client.
pub struct MinimaxClient {
    /// HTTP client.
    client: Client,
    /// API key (stored as SecretString internally).
    api_key: ApiKey,
    /// Base URL for API.
    base_url: String,
}

impl MinimaxClient {
    /// Create a new MiniMax client.
    #[allow(dead_code)]
    pub fn new(api_key: ApiKey) -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .expect("Failed to build HTTP client");

        MinimaxClient {
            client,
            api_key,
            base_url: MINIMAX_BASE_URL.to_string(),
        }
    }

    /// Create a new client with custom base URL (for testing).
    #[allow(dead_code)]
    pub fn with_base_url(api_key: ApiKey, base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .expect("Failed to build HTTP client");

        MinimaxClient {
            client,
            api_key,
            base_url: base_url.into(),
        }
    }

    /// Build the chat completion URL.
    fn chat_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    /// Execute the HTTP request with retry logic.
    async fn execute_with_retry(&self, request: ChatRequest) -> Result<String, SpecGenError> {
        let mut last_error = None;
        let mut backoff = RATE_LIMIT_BACKOFF;

        for attempt in 0..MAX_RETRIES {
            let result = self.execute_request(request.clone()).await;

            match result {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Check if error is retryable
                    match &e {
                        SpecGenError::RateLimited(_) => {
                            // Rate limited - wait and retry with exponential backoff
                            tokio::time::sleep(backoff).await;
                            backoff *= 2;
                            last_error = Some(e);
                        }
                        SpecGenError::HttpError(status, _) if *status >= 500 => {
                            // Server error - wait and retry
                            let server_backoff = SERVER_ERROR_BACKOFF * (2_u32.pow(attempt)).max(1);
                            tokio::time::sleep(server_backoff).await;
                            last_error = Some(e);
                        }
                        SpecGenError::NetworkError(_) => {
                            // Network error - wait and retry
                            tokio::time::sleep(backoff).await;
                            backoff *= 2;
                            last_error = Some(e);
                        }
                        _ => {
                            // Non-retryable error - return immediately
                            return Err(e);
                        }
                    }
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap_or(SpecGenError::Unexpected(
            "Unknown error after retries".to_string(),
        )))
    }

    /// Execute a single HTTP request.
    async fn execute_request(&self, request: ChatRequest) -> Result<String, SpecGenError> {
        let url = self.chat_url();

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.as_str()))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        // Handle different status codes
        if status == 429 {
            // Rate limited - extract retry-after if available
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs);

            return Err(SpecGenError::RateLimited(
                retry_after.unwrap_or(RATE_LIMIT_BACKOFF),
            ));
        }

        if status.is_server_error() {
            return Err(SpecGenError::HttpError(
                status.as_u16(),
                "Server error".to_string(),
            ));
        }

        if status.is_client_error() && status != 429 {
            let body = response.text().await.unwrap_or_default();
            return Err(SpecGenError::HttpError(status.as_u16(), body));
        }

        // Success - parse streaming response
        let stream = parse_sse_stream(response).await?;

        // Collect all chunks into a single response
        let mut full_response = String::new();
        let mut stream = Box::pin(stream);

        while let Some(chunk_result) = futures::StreamExt::next(&mut stream).await {
            match chunk_result {
                Ok(chunk) => full_response.push_str(&chunk),
                Err(e) => return Err(e),
            }
        }

        Ok(full_response)
    }
}

#[async_trait]
impl AiClient for MinimaxClient {
    /// Send a chat request and get a streaming response.
    async fn chat(&self, request: ChatRequest) -> Result<String, SpecGenError> {
        self.execute_with_retry(request).await
    }

    /// Validate the API key by making a test request.
    async fn validate_api_key(&self, api_key: &ApiKey) -> Result<(), SpecGenError> {
        // Create a simple test request
        let request =
            ChatRequest::new_spec_request(vec![Message::system("You are a helpful assistant.")]);

        let url = self.chat_url();

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key.as_str()))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            Ok(())
        } else if status == 401 {
            Err(SpecGenError::InvalidApiKey("Invalid API key".to_string()))
        } else if status == 429 {
            // Rate limited but key is valid
            Ok(())
        } else {
            Err(SpecGenError::HttpError(
                status.as_u16(),
                "API validation failed".to_string(),
            ))
        }
    }
}

/// Create a new MiniMax client from an API key.
#[allow(dead_code)]
pub fn create_client(api_key: ApiKey) -> Arc<dyn AiClient> {
    Arc::new(MinimaxClient::new(api_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_url() {
        let client = MinimaxClient::new(ApiKey::new("test_key_12345".to_string()));
        assert_eq!(
            client.chat_url(),
            "https://api.minimax.io/v1/chat/completions"
        );
    }

    #[test]
    fn test_chat_url_custom_base() {
        let client = MinimaxClient::with_base_url(
            ApiKey::new("test_key_12345".to_string()),
            "https://test.minimax.io",
        );
        assert_eq!(
            client.chat_url(),
            "https://test.minimax.io/v1/chat/completions"
        );
    }
}
