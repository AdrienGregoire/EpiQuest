/*
 ** EPITECH PROJECT, 2026
 ** roll.rs
 ** File description:
 ** Roll model
 */

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use dice::RollResult;

#[derive(Debug, Clone, FromRow)]
pub struct Roll {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub user_id: Uuid,
    pub notation: String,
    pub result: Json<RollResult>,
    pub total: i32,
    pub created_at: DateTime<Utc>,
}

impl Roll {
    pub async fn create(
        pool: &PgPool,
        campaign_id: Uuid,
        user_id: Uuid,
        notation: &str,
        result: &RollResult,
        total: i64,
    ) -> Result<Roll, sqlx::Error> {
        sqlx::query_as::<_, Roll>(
            r#"
            INSERT INTO rolls (campaign_id, user_id, formula, total, roll_type, data)
            VALUES ($1, $2, $3, $4, 'manual', $5)
            RETURNING id, campaign_id, user_id, formula AS notation, data AS result, total, created_at
            "#,
        )
        .bind(campaign_id)
        .bind(user_id)
        .bind(notation)
        .bind(total as i32)
        .bind(Json(result))
        .fetch_one(pool)
        .await
    }

    pub async fn list_by_campaign(
        pool: &PgPool,
        campaign_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Roll>, sqlx::Error> {
        sqlx::query_as::<_, Roll>(
            r#"
            SELECT id, campaign_id, user_id, formula AS notation, data AS result, total, created_at
            FROM rolls
            WHERE campaign_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(campaign_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}
