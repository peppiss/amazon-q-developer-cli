use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use cli::api_client::StreamingClient;
use cli::database::Database;
use eyre::Result;

use crate::models::ConversationSession;

/// Server state shared across all API requests
#[derive(Clone)]
pub struct ServerState {
    /// The streaming client for Amazon Q
    pub streaming_client: StreamingClient,
    
    /// Active conversations
    pub conversations: Arc<RwLock<HashMap<String, ConversationSession>>>,
}

impl ServerState {
    /// Create a new server state
    pub async fn new() -> Result<Self> {
        // Initialize the database
        let mut database = Database::new().await?;
        
        // Initialize the streaming client
        let streaming_client = StreamingClient::new(&mut database).await
            .map_err(|e| eyre::eyre!("Failed to initialize streaming client: {}", e))?;
        
        Ok(Self {
            streaming_client,
            conversations: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Get a conversation by ID, or create a new one if it doesn't exist
    pub fn get_or_create_conversation(&self, id: Option<String>) -> ConversationSession {
        let mut conversations = self.conversations.write().unwrap();
        
        match id {
            Some(id) if conversations.contains_key(&id) => {
                conversations.get(&id).unwrap().clone()
            },
            _ => {
                let session = ConversationSession::new();
                let id = session.id.clone();
                conversations.insert(id, session.clone());
                session
            }
        }
    }
    
    /// Update a conversation in the server state
    pub fn update_conversation(&self, session: ConversationSession) {
        let mut conversations = self.conversations.write().unwrap();
        conversations.insert(session.id.clone(), session);
    }
}
