//! SSE streaming handler for MiniMax API responses.

use crate::error::SpecGenError;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use reqwest::Response;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A stream of text chunks from the MiniMax API.
pub struct TextStream {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, SpecGenError>> + Send>>,
}

impl Stream for TextStream {
    type Item = Result<String, SpecGenError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                Poll::Ready(Some(Ok(text)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Parse SSE stream from HTTP response.
///
/// Takes a reqwest response body and yields text chunks.
pub async fn parse_sse_stream(response: Response) -> Result<TextStream, SpecGenError> {
    let byte_stream = response.bytes_stream();

    let stream = byte_stream.map(|result| match result {
        Ok(bytes) => parse_sse_chunk(&bytes),
        Err(e) => Err(SpecGenError::StreamError(e.to_string())),
    });

    Ok(TextStream {
        inner: Box::pin(stream),
    })
}

/// Parse a single SSE chunk into text content.
///
/// SSE format: `data: {"id":"...","choices":[...]}\n\n`
fn parse_sse_chunk(bytes: &Bytes) -> Result<Bytes, SpecGenError> {
    let text = String::from_utf8_lossy(bytes);

    // Process each line
    for line in text.lines() {
        let line = line.trim();

        // Skip empty lines and comment lines
        if line.is_empty() || line.starts_with(':') {
            continue;
        }

        // Check if this is a data line
        if let Some(data) = line.strip_prefix("data: ") {
            // Check for done signal
            if data.trim() == "[DONE]" {
                continue;
            }

            // Try to extract content from the JSON
            if let Some(content) = extract_content_from_sse(data) {
                return Ok(Bytes::from(content));
            }
        }
    }

    Ok(Bytes::new())
}

/// Extract content from SSE data payload.
fn extract_content_from_sse(data: &str) -> Option<String> {
    // Simple JSON parsing - look for "content":"..."
    // This is a simplified parser; in production, use proper JSON parsing

    // Try to find "content":"
    let content_start = data.find("\"content\":\"");

    if let Some(start) = content_start {
        let start = start + "\"content\":\"".len();
        let end = data[start..].find('"').map(|i| start + i);

        if let Some(end) = end {
            let content = &data[start..end];
            // Unescape common escape sequences
            let unescaped = content
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\");

            return Some(unescaped);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_content_from_sse() {
        let data = r#"{"id":"cmpl-xxx","choices":[{"delta":{"content":"Hello"},"index":0}]}"#;
        let content = extract_content_from_sse(data);
        assert_eq!(content, Some("Hello".to_string()));
    }

    #[test]
    fn test_extract_content_multiline() {
        let data =
            r#"{"id":"cmpl-xxx","choices":[{"delta":{"content":"Line 1\nLine 2"},"index":0}]}"#;
        let content = extract_content_from_sse(data);
        assert_eq!(content, Some("Line 1\nLine 2".to_string()));
    }

    #[test]
    fn test_extract_content_empty() {
        let data = r#"{"id":"cmpl-xxx","choices":[]}"#;
        let content = extract_content_from_sse(data);
        assert_eq!(content, None);
    }
}
