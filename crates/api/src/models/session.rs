/*
 ** EPITECH PROJECT, 2026
 ** session.rs
 ** File description:
 ** session functions
 */

use chrono::{DateTime, TimeDelta, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::models::user::User;

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl Session {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        token_hash: &str,
        ttl_secs: i64,
    ) -> Result<Session, sqlx::Error> {
        let expires_at = Utc::now() + TimeDelta::seconds(ttl_secs);

        sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, token_hash, created_at, expires_at, revoked_at
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(pool)
        .await
    }

    pub async fn find_user_by_token_hash(
        pool: &PgPool,
        token_hash: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT u.id, u.username, u.email, u.password_hash, u.created_at
            FROM sessions s
            JOIN users u ON u.id = s.user_id
            WHERE s.token_hash = $1
              AND s.revoked_at IS NULL
              AND s.expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
    }

    pub async fn revoke(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE sessions SET revoked_at = NOW() WHERE token_hash = $1 AND revoked_at IS NULL",
        )
        .bind(token_hash)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
        )
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
