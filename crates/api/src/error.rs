/*
** EPITECH PROJECT, 2026
** error.rs
** File description:
** application errors
*/

use axum::{Json, http::StatusCode, response::IntoResponse};
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
}
