/*
 ** EPITECH PROJECT, 2026
 ** invite.rs
 ** File description:
 ** campaign invite functions
 */

use chrono::{DateTime, TimeDelta, Utc};
use rand_core::{OsRng, RngCore};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub const CODE_LEN: usize = 8;
pub const INVITE_TTL_HOURS: i64 = 7 * 24;
const CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn generate_code() -> String {
    let mut bytes = [0u8; CODE_LEN];

    OsRng.fill_bytes(&mut bytes);
    bytes
        .iter()
        .map(|byte| CODE_ALPHABET[(byte & 31) as usize] as char)
        .collect()
}

#[derive(Debug, Clone, FromRow)]
pub struct Invite {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl Invite {
    pub async fn create(
        pool: &PgPool,
        campaign_id: Uuid,
        code: &str,
        ttl_hours: i64,
    ) -> Result<Invite, sqlx::Error> {
        let expires_at = Utc::now() + TimeDelta::hours(ttl_hours);

        sqlx::query_as::<_, Invite>(
            r#"
            INSERT INTO campaign_invites (campaign_id, code, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, campaign_id, code, created_at, expires_at, revoked_at
            "#,
        )
        .bind(campaign_id)
        .bind(code)
        .bind(expires_at)
        .fetch_one(pool)
        .await
    }

    pub async fn list_active(pool: &PgPool, campaign_id: Uuid) -> Result<Vec<Invite>, sqlx::Error> {
        sqlx::query_as::<_, Invite>(
            r#"
            SELECT id, campaign_id, code, created_at, expires_at, revoked_at
            FROM campaign_invites
            WHERE campaign_id = $1
              AND revoked_at IS NULL
              AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(campaign_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_active_by_code(
        pool: &PgPool,
        code: &str,
    ) -> Result<Option<Invite>, sqlx::Error> {
        sqlx::query_as::<_, Invite>(
            r#"
            SELECT id, campaign_id, code, created_at, expires_at, revoked_at
            FROM campaign_invites
            WHERE code = $1
              AND revoked_at IS NULL
              AND expires_at > NOW()
            "#,
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    pub async fn revoke(
        pool: &PgPool,
        campaign_id: Uuid,
        invite_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE campaign_invites
            SET revoked_at = NOW()
            WHERE id = $1 AND campaign_id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(invite_id)
        .bind(campaign_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

impl From<Invite> for shared::InviteResponse {
    fn from(invite: Invite) -> Self {
        Self {
            id: invite.id,
            campaign_id: invite.campaign_id,
            code: invite.code,
            created_at: invite.created_at,
            expires_at: invite.expires_at,
        }
    }
}
