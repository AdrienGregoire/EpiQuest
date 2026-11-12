/*
** EPITECH PROJECT, 2026
** error.rs
** File description:
** application errors
*/


use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation: {0}")]
    Validation(String),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db_err) = err
            && db_err.code().as_deref() == Some("23505")
        {
            return AppError::Conflict("Username or email already exists".to_string());
        }
        AppError::Internal(err.to_string())
    }
}
