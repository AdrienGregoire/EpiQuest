/*
** EPITECH PROJECT, 2026
** auth.rs
** File description:
** auth routes and handlers
*/

use axum::{Json, extract::State, http::StatusCode};
use shared::{RegisterRequest, UserResponse};
use sqlx::PgPool;

use crate::{error::AppError, models::user::User};

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let username = payload.username.trim();

    if username.is_empty() {
        return Err(AppError::Validation("Username cannot be empty".to_string()));
    }
    if username.len() > 50 {
        return Err(AppError::Validation(
            "Username cannot exceed 50 characters".to_string(),
        ));
    }
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') || !email.contains('.') {
        return Err(AppError::Validation("Invalid email address".to_string()));
    }
    if payload.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    let password = payload.password;
    let password_hash =
        tokio::task::spawn_blocking(move || crate::auth::password::hash_password(&password))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .map_err(|e| AppError::Internal(e.to_string()))?;
    let user = User::create(&pool, username, &email, &password_hash).await?;
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: user.created_at,
    };
    Ok((StatusCode::CREATED, Json(response)))
}
