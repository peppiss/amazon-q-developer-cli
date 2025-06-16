use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request model for chat conversations
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    /// The message to send to Amazon Q
    pub message: String,
    
    /// Optional conversation ID to continue an existing conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    
    /// Optional context files to include with the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_files: Option<Vec<ContextFile>>,
}

/// Context file to provide additional information to Amazon Q
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextFile {
    /// The name of the file
    pub name: String,
    
    /// The content of the file
    pub content: String,
}

/// Response model for chat conversations
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Unique identifier for the conversation
    pub conversation_id: String,
    
    /// The response message from Amazon Q
    pub message: String,
    
    /// Optional code snippets included in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_snippets: Option<Vec<CodeSnippet>>,
    
    /// Optional citations for the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<Citation>>,
}

/// Code snippet included in a response
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSnippet {
    /// The programming language of the code
    pub language: String,
    
    /// The code content
    pub code: String,
}

/// Citation for information in a response
#[derive(Debug, Serialize, Deserialize)]
pub struct Citation {
    /// The source of the citation
    pub source: String,
    
    /// The URL of the citation, if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Streaming response chunk for chat conversations
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatStreamChunk {
    /// Unique identifier for the conversation
    pub conversation_id: String,
    
    /// The type of chunk (text, code, citation, etc.)
    pub chunk_type: ChunkType,
    
    /// The content of the chunk
    pub content: String,
    
    /// Whether this is the final chunk in the response
    pub is_final: bool,
}

/// Types of chunks in a streaming response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkType {
    Text,
    Code,
    Citation,
    Error,
}

/// Session information for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    /// Unique identifier for the conversation
    pub id: String,
    
    /// The history of messages in the conversation
    pub messages: Vec<Message>,
    
    /// When the conversation was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// When the conversation was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ConversationSession {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            messages: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
    
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(Message {
            role,
            content,
            timestamp: chrono::Utc::now(),
        });
        self.updated_at = chrono::Utc::now();
    }
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender (user or assistant)
    pub role: MessageRole,
    
    /// The content of the message
    pub content: String,
    
    /// When the message was sent
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// The role of a message sender
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
}
