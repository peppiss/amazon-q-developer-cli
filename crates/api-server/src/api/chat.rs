use actix_web::{web, HttpResponse, Responder, get, post};
use async_stream::stream;
use cli::api_client::model::{
    ChatMessage, ChatResponseStream, ConversationState, UserInputMessage, UserInputMessageContext
};


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
            let mut response_clone = response;
            while let Ok(Some(event)) = response_clone.recv().await {
                {
                        // Process different event types
                        if let ChatResponseStream::AssistantResponseEvent { content } = event {
                            full_response.push_str(&content);
                        } else if let ChatResponseStream::CodeEvent { content } = event {
                            code_snippets.push(crate::models::CodeSnippet {
                                language: "".to_string(), // Default language
                                code: content,
                            });
                        }
                        // Other event types can be handled here
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
                let mut response_clone = response;
                while let Ok(Some(event)) = response_clone.recv().await {
                    // Process different event types
                    if let ChatResponseStream::AssistantResponseEvent { content } = event {
                        full_response.push_str(&content);
                        
                        yield Ok::<_, actix_web::Error>(web::Bytes::from(
                            serde_json::to_string(&ChatStreamChunk {
                                conversation_id: conversation_id.clone(),
                                chunk_type: ChunkType::Text,
                                content: content.clone(),
                                is_final: false,
                            }).unwrap()
                        ));
                    } else if let ChatResponseStream::CodeEvent { content } = event {
                        yield Ok::<_, actix_web::Error>(web::Bytes::from(
                            serde_json::to_string(&ChatStreamChunk {
                                conversation_id: conversation_id.clone(),
                                chunk_type: ChunkType::Code,
                                content: serde_json::to_string(&crate::models::CodeSnippet {
                                    language: "".to_string(), // Default language
                                    code: content,
                                }).unwrap(),
                                is_final: false,
                            }).unwrap()
                        ));
                    } else if let ChatResponseStream::InvalidStateEvent { reason, message } = event {
                        error!("Invalid state: {} - {}", reason, message);
                        yield Ok::<_, actix_web::Error>(web::Bytes::from(
                            serde_json::to_string(&ChatStreamChunk {
                                conversation_id: conversation_id.clone(),
                                chunk_type: ChunkType::Error,
                                content: format!("Error: {}", message),
                                is_final: true,
                            }).unwrap()
                        ));
                        break;
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
    let mut result = Vec::new();
    
    for (_id, session) in conversations.iter() {
        result.push(session.clone());
    }
    
    Ok(HttpResponse::Ok().json(result))
}

/// Create a conversation state from a session
fn create_conversation_state(session: &crate::models::ConversationSession) -> ConversationState {
    // Get the last user message if available
    let last_user_message = session.messages.iter()
        .filter(|msg| msg.role == MessageRole::User)
        .last();
    
    // Create history from previous messages
    let mut history = Vec::new();
    
    // Add message pairs to history
    let mut user_msg = None;
    for msg in &session.messages {
        match msg.role {
            MessageRole::User => {
                user_msg = Some(msg);
            },
            MessageRole::Assistant => {
                if let Some(user) = user_msg {
                    // Add user message
                    history.push(ChatMessage::UserInputMessage(UserInputMessage {
                        content: user.content.clone(),
                        user_input_message_context: None,
                        user_intent: None,
                        images: None,
                    }));
                    
                    // Add assistant message
                    history.push(ChatMessage::AssistantResponseMessage(
                        cli::api_client::model::AssistantResponseMessage {
                            content: msg.content.clone(),
                            message_id: None,
                            tool_uses: None,
                        }
                    ));
                    
                    user_msg = None;
                }
            }
        }
    }
    
    // Create user input message from the last user message
    let user_input_message = if let Some(last_msg) = last_user_message {
        UserInputMessage {
            content: last_msg.content.clone(),
            user_input_message_context: Some(UserInputMessageContext::default()),
            user_intent: None,
            images: None,
        }
    } else {
        UserInputMessage {
            content: String::new(),
            user_input_message_context: Some(UserInputMessageContext::default()),
            user_intent: None,
            images: None,
        }
    };
    
    // Create the conversation state
    ConversationState {
        conversation_id: Some(session.id.clone()),
        user_input_message,
        history: Some(history),
    }
}
