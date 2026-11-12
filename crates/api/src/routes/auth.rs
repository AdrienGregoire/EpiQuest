/*
** EPITECH PROJECT, 2026
** auth.rs
** File description:
** auth routes and handlers
*/

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use shared::{LoginRequest, RegisterRequest, UserResponse};
use sqlx::PgPool;
use std::sync::LazyLock;

use crate::{
    auth::{
        extractor::AuthUser,
        password::verify_password,
        session::{SESSION_COOKIE, SESSION_TTL_SECS, cookie_secure, generate_token, hash_token},
    },
    error::AppError,
    models::{session::Session, user::User},
};

static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    crate::auth::password::hash_password("dummy-password").expect("failed to hash dummy password")
});

fn session_cookie(token: String) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .secure(cookie_secure())
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::seconds(SESSION_TTL_SECS))
        .build()
}

fn removal_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, "")).path("/").build()
}

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

pub async fn login(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<UserResponse>), AppError> {
    let email = payload.email.trim().to_lowercase();
    let user = User::find_by_email(&pool, &email).await?;
    let hash = user
        .as_ref()
        .map_or_else(|| DUMMY_HASH.to_string(), |u| u.password_hash.clone());
    let password = payload.password;
    let is_valid = tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user = match user {
        Some(user) if is_valid => user,
        _ => {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }
    };
    let token = generate_token();
    Session::create(&pool, user.id, &hash_token(&token), SESSION_TTL_SECS).await?;
    Ok((jar.add(session_cookie(token)), Json(user.into())))
}

pub async fn logout(
    State(pool): State<PgPool>,
    auth: AuthUser,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AppError> {
    Session::revoke(&pool, &auth.token_hash).await?;
    Ok((jar.remove(removal_cookie()), StatusCode::NO_CONTENT))
}

pub async fn logout_all(
    State(pool): State<PgPool>,
    auth: AuthUser,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AppError> {
    Session::revoke_all_for_user(&pool, auth.user.id).await?;
    Ok((jar.remove(removal_cookie()), StatusCode::NO_CONTENT))
}

pub async fn me(auth: AuthUser) -> Json<UserResponse> {
    Json(auth.user.into())
}
