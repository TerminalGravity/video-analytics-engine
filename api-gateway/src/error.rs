use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, error_code) = match self {
            AppError::Database(_) => {
                tracing::error!("Database error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                    "DATABASE_ERROR",
                )
            }
            AppError::Authentication(msg) => {
                tracing::warn!("Authentication error: {}", msg);
                (StatusCode::UNAUTHORIZED, msg, "AUTHENTICATION_ERROR")
            }
            AppError::Authorization(msg) => {
                tracing::warn!("Authorization error: {}", msg);
                (StatusCode::FORBIDDEN, msg, "AUTHORIZATION_ERROR")
            }
            AppError::Validation(msg) => {
                tracing::warn!("Validation error: {}", msg);
                (StatusCode::BAD_REQUEST, msg, "VALIDATION_ERROR")
            }
            AppError::NotFound(msg) => {
                tracing::info!("Not found: {}", msg);
                (StatusCode::NOT_FOUND, msg, "NOT_FOUND")
            }
            AppError::Conflict(msg) => {
                tracing::warn!("Conflict: {}", msg);
                (StatusCode::CONFLICT, msg, "CONFLICT")
            }
            AppError::RateLimited => {
                tracing::warn!("Rate limit exceeded");
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Rate limit exceeded".to_string(),
                    "RATE_LIMITED",
                )
            }
            AppError::BadRequest(msg) => {
                tracing::warn!("Bad request: {}", msg);
                (StatusCode::BAD_REQUEST, msg, "BAD_REQUEST")
            }
            AppError::ServiceUnavailable(msg) => {
                tracing::error!("Service unavailable: {}", msg);
                (StatusCode::SERVICE_UNAVAILABLE, msg, "SERVICE_UNAVAILABLE")
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    "INTERNAL_ERROR",
                )
            }
            AppError::Jwt(_) => {
                tracing::warn!("JWT error: {}", self);
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid token".to_string(),
                    "INVALID_TOKEN",
                )
            }
            AppError::Redis(_) => {
                tracing::error!("Redis error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Cache error occurred".to_string(),
                    "CACHE_ERROR",
                )
            }
            AppError::HttpClient(_) => {
                tracing::error!("HTTP client error: {}", self);
                (
                    StatusCode::BAD_GATEWAY,
                    "External service error".to_string(),
                    "EXTERNAL_SERVICE_ERROR",
                )
            }
            AppError::Serialization(_) => {
                tracing::error!("Serialization error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Data processing error".to_string(),
                    "DATA_PROCESSING_ERROR",
                )
            }
            AppError::Config(msg) => {
                tracing::error!("Configuration error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Configuration error".to_string(),
                    "CONFIG_ERROR",
                )
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": error_message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }));

        (status, body).into_response()
    }
}

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>; 