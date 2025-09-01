use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
            let (status, error_message) = match &self {
        AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, Cow::Borrowed("Database error")),
        AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, Cow::Borrowed(msg.as_str())),
        AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, Cow::Borrowed(msg.as_str())),
        AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, Cow::Borrowed(msg.as_str())),
        AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, Cow::Borrowed(msg.as_str())),
    };

        let body = axum::Json(serde_json::json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
