/*
 ** EPITECH PROJECT, 2026
 ** member.rs
 ** File description:
 ** campaign member functions
 */

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Member {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

impl Member {
    pub async fn add_player(
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO campaign_members (campaign_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(campaign_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn is_member(
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let (exists,): (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM campaign_members
                WHERE campaign_id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(campaign_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }

    pub async fn list_for_campaign(
        pool: &PgPool,
        campaign_id: Uuid,
    ) -> Result<Vec<Member>, sqlx::Error> {
        sqlx::query_as::<_, Member>(
            r#"
            SELECT u.id AS user_id, u.username, 'gm'::text AS role, c.created_at AS joined_at
            FROM campaigns c
            JOIN users u ON u.id = c.gm_id
            WHERE c.id = $1
            UNION ALL
            SELECT u.id AS user_id, u.username, 'player'::text AS role, m.joined_at
            FROM campaign_members m
            JOIN users u ON u.id = m.user_id
            WHERE m.campaign_id = $1
            ORDER BY joined_at
            "#,
        )
        .bind(campaign_id)
        .fetch_all(pool)
        .await
    }
}

impl From<Member> for shared::MemberResponse {
    fn from(member: Member) -> Self {
        Self {
            user_id: member.user_id,
            username: member.username,
            role: member.role,
            joined_at: member.joined_at,
        }
    }
}
