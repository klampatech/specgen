//! MiniMax API request/response models.

#[allow(dead_code)]
use serde::{Deserialize, Serialize};

/// Message role in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message.
    System,
    /// User message.
    User,
    /// Assistant message.
    Assistant,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The content of the message.
    pub content: String,
}

#[allow(dead_code)]
impl Message {
    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Message {
            role: Role::System,
            content: content.into(),
        }
    }

    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Message {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Message {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Chat completion request to MiniMax API.
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    /// The model to use.
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<Message>,
    /// Whether to stream the response.
    pub stream: bool,
    /// Temperature for sampling (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[allow(dead_code)]
impl ChatRequest {
    /// Create a new chat request for spec generation.
    pub fn new_spec_request(messages: Vec<Message>) -> Self {
        ChatRequest {
            model: "MiniMax-M2.7".to_string(),
            messages,
            stream: true,
            temperature: Some(0.3),
            max_tokens: Some(8192),
        }
    }

    /// Create a new chat request for interview questions.
    pub fn new_interview_request(messages: Vec<Message>) -> Self {
        ChatRequest {
            model: "MiniMax-M2.7".to_string(),
            messages,
            stream: true,
            temperature: Some(0.7),
            max_tokens: Some(2048),
        }
    }
}

/// Chat completion response from MiniMax API.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ChatResponse {
    /// Unique identifier for this completion.
    pub id: String,
    /// The model used.
    pub model: String,
    /// Generated choices.
    pub choices: Vec<Choice>,
}

/// A single choice in the response.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Choice {
    /// Index of this choice.
    pub index: u32,
    /// The generated message.
    pub message: Message,
    /// Why the generation finished (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Streaming chunk from MiniMax API.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StreamChunk {
    /// Unique identifier for this completion.
    pub id: String,
    /// The model used.
    pub model: String,
    /// Streaming choices.
    pub choices: Vec<StreamChoice>,
}

/// A single streaming choice.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StreamChoice {
    /// Index of this choice.
    pub index: u32,
    /// Delta content (partial message).
    pub delta: StreamDelta,
    /// Why the generation stopped (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Partial message content in a stream.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StreamDelta {
    /// Role of the message (only in first chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// Content delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Error response from MiniMax API.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ApiError {
    /// Error message.
    pub error: ApiErrorDetail,
}

/// Error details.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ApiErrorDetail {
    /// Error message.
    pub message: String,
    /// Error type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
}
