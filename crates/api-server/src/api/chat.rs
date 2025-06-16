use actix_web::{web, HttpResponse, Responder, get, post};
use async_stream::stream;
use cli::api_client::model::{
    ChatMessage, ChatTriggerType, ConversationState, 
    EditorState, Origin, UserMessage
};
use futures::StreamExt;
use std::time::Duration;
use tracing::{error, info};

use crate::error::ApiError;
use crate::models::{ChatRequest, ChatResponse, ChatStreamChunk, ChunkType, MessageRole};
use crate::server::ServerState;

/// Configure chat-related routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/chat")
            .service(chat)
            .service(chat_stream)
            .service(get_conversations)
    );
}

/// Endpoint for non-streaming chat requests
#[post("")]
async fn chat(
    req: web::Json<ChatRequest>,
    state: web::Data<ServerState>,
) -> Result<impl Responder, ApiError> {
    info!("Received chat request: {:?}", req);
    
    // Get or create a conversation session
    let mut session = state.get_or_create_conversation(req.conversation_id.clone());
    
    // Add the user message to the conversation
    session.add_message(MessageRole::User, req.message.clone());
    
    // Create the conversation state for Amazon Q
    let conversation_state = create_conversation_state(&session);
    
    // Send the message to Amazon Q
    let result = state.streaming_client.send_message(conversation_state).await;
    
    match result {
        Ok(response) => {
            // Collect all response chunks into a single response
            let mut full_response = String::new();
            let mut code_snippets = Vec::new();
            
            // Process the response stream
            let mut stream = response.into_stream();
            while let Some(event) = stream.next().await {
                match event {
                    Ok(event) => {
                        // Process different event types
                        if let Some(text_event) = event.text_event() {
                            full_response.push_str(&text_event.text);
                        } else if let Some(code_event) = event.code_event() {
                            code_snippets.push(crate::models::CodeSnippet {
                                language: code_event.language.clone().unwrap_or_default(),
                                code: code_event.code.clone(),
                            });
                        }
                        // Other event types can be handled here
                    },
                    Err(e) => {
                        error!("Error processing response stream: {:?}", e);
                        return Err(ApiError::AmazonQApiError(format!("Error processing response: {}", e)));
                    }
                }
            }
            
            // Add the assistant's response to the conversation
            session.add_message(MessageRole::Assistant, full_response.clone());
            
            // Update the conversation in the server state
            state.update_conversation(session.clone());
            
            // Create the response
            let response = ChatResponse {
                conversation_id: session.id,
                message: full_response,
                code_snippets: if code_snippets.is_empty() { None } else { Some(code_snippets) },
                citations: None, // Citations could be extracted from citation events
            };
            
            Ok(HttpResponse::Ok().json(response))
        },
        Err(e) => {
            error!("Error sending message to Amazon Q: {:?}", e);
            Err(ApiError::AmazonQApiError(format!("Error sending message: {}", e)))
        }
    }
}

/// Endpoint for streaming chat requests
#[post("/stream")]
async fn chat_stream(
    req: web::Json<ChatRequest>,
    state: web::Data<ServerState>,
) -> Result<impl Responder, ApiError> {
    info!("Received streaming chat request: {:?}", req);
    
    // Get or create a conversation session
    let mut session = state.get_or_create_conversation(req.conversation_id.clone());
    
    // Add the user message to the conversation
    session.add_message(MessageRole::User, req.message.clone());
    
    // Create the conversation state for Amazon Q
    let conversation_state = create_conversation_state(&session);
    
    // Send the message to Amazon Q
    let result = state.streaming_client.send_message(conversation_state).await;
    
    match result {
        Ok(response) => {
            // Create a stream of response chunks
            let conversation_id = session.id.clone();
            let mut full_response = String::new();
            
            // Clone state for use in the stream
            let state_clone = state.clone();
            let session_clone = session.clone();
            
            // Create a stream that will send chunks as they arrive
            let stream = stream! {
                let mut stream = response.into_stream();
                while let Some(event) = stream.next().await {
                    match event {
                        Ok(event) => {
                            // Process different event types
                            if let Some(text_event) = event.text_event() {
                                full_response.push_str(&text_event.text);
                                
                                yield Ok::<_, actix_web::Error>(web::Bytes::from(
                                    serde_json::to_string(&ChatStreamChunk {
                                        conversation_id: conversation_id.clone(),
                                        chunk_type: ChunkType::Text,
                                        content: text_event.text.clone(),
                                        is_final: false,
                                    }).unwrap()
                                ));
                            } else if let Some(code_event) = event.code_event() {
                                yield Ok::<_, actix_web::Error>(web::Bytes::from(
                                    serde_json::to_string(&ChatStreamChunk {
                                        conversation_id: conversation_id.clone(),
                                        chunk_type: ChunkType::Code,
                                        content: serde_json::to_string(&crate::models::CodeSnippet {
                                            language: code_event.language.clone().unwrap_or_default(),
                                            code: code_event.code.clone(),
                                        }).unwrap(),
                                        is_final: false,
                                    }).unwrap()
                                ));
                            }
                            // Other event types can be handled here
                        },
                        Err(e) => {
                            error!("Error processing response stream: {:?}", e);
                            yield Ok::<_, actix_web::Error>(web::Bytes::from(
                                serde_json::to_string(&ChatStreamChunk {
                                    conversation_id: conversation_id.clone(),
                                    chunk_type: ChunkType::Error,
                                    content: format!("Error: {}", e),
                                    is_final: true,
                                }).unwrap()
                            ));
                            break;
                        }
                    }
                }
                
                // Send final chunk
                yield Ok::<_, actix_web::Error>(web::Bytes::from(
                    serde_json::to_string(&ChatStreamChunk {
                        conversation_id: conversation_id.clone(),
                        chunk_type: ChunkType::Text,
                        content: String::new(),
                        is_final: true,
                    }).unwrap()
                ));
                
                // Update the conversation with the full response
                let mut session = session_clone;
                session.add_message(MessageRole::Assistant, full_response);
                state_clone.update_conversation(session);
            };
            
            // Return the stream as the response
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .streaming(stream))
        },
        Err(e) => {
            error!("Error sending message to Amazon Q: {:?}", e);
            Err(ApiError::AmazonQApiError(format!("Error sending message: {}", e)))
        }
    }
}

/// Endpoint to get all conversations
#[get("/conversations")]
async fn get_conversations(state: web::Data<ServerState>) -> Result<impl Responder, ApiError> {
    let conversations = state.conversations.read().unwrap();
    let conversations: Vec<_> = conversations.values().cloned().collect();
    
    Ok(HttpResponse::Ok().json(conversations))
}

/// Create a conversation state from a session
fn create_conversation_state(session: &crate::models::ConversationSession) -> ConversationState {
    // Convert the session messages to ChatMessage format
    let messages: Vec<ChatMessage> = session.messages.iter().map(|msg| {
        match msg.role {
            MessageRole::User => ChatMessage::User(UserMessage {
                content: msg.content.clone(),
                trigger_type: Some(ChatTriggerType::Manual),
            }),
            MessageRole::Assistant => ChatMessage::Assistant(cli::api_client::model::AssistantMessage {
                content: msg.content.clone(),
            }),
        }
    }).collect();
    
    // Create the conversation state
    ConversationState {
        messages,
        editor_state: Some(EditorState::default()),
        source: Some(Origin::Cli),
        ..Default::default()
    }
}
