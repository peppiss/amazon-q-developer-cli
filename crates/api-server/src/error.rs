use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    
    #[error("Amazon Q API error: {0}")]
    AmazonQApiError(String),
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, error_type) = match self {
            ApiError::Authentication(_) => (actix_web::http::StatusCode::UNAUTHORIZED, "authentication_error"),
            ApiError::Authorization(_) => (actix_web::http::StatusCode::FORBIDDEN, "authorization_error"),
            ApiError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "not_found"),
            ApiError::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::Database(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            ApiError::InternalServerError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "internal_server_error"),
            ApiError::AmazonQApiError(_) => (actix_web::http::StatusCode::BAD_GATEWAY, "amazon_q_api_error"),
        };

        HttpResponse::build(status_code).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        })
    }
}
