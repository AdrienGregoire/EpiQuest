/*
 ** EPITECH PROJECT, 2026
 ** extractor.rs
 ** File description:
 ** authenticated user extractor
 */

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::cookie::CookieJar;
use sqlx::PgPool;

use crate::{
    auth::session::{SESSION_COOKIE, hash_token},
    error::AppError,
    models::{session::Session, user::User},
};

pub struct AuthUser {
    pub user: User,
    pub token_hash: String,
}

impl FromRequestParts<PgPool> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, pool: &PgPool) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar
            .get(SESSION_COOKIE)
            .map(|c| c.value().to_owned())
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))?;
        let token_hash = hash_token(&token);
        let user = Session::find_user_by_token_hash(pool, &token_hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid or expired session".to_string()))?;

        Ok(AuthUser { user, token_hash })
    }
}
