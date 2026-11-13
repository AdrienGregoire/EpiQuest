/*
 ** EPITECH PROJECT, 2026
 ** campaign.rs
 ** File description:
 ** campaign functions
 */

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub gm_id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Campaign {
    pub async fn create(pool: &PgPool, gm_id: Uuid, name: &str, description: &str) -> Result<Campaign, sqlx::Error> {
        sqlx::query_as::<_, Campaign>(
            r#"
            INSERT INTO campaigns (gm_id, name, description)
            VALUES ($1, $2, $3)
            RETURNING id, gm_id, name, description, created_at, updated_at
            "#,
        ).bind(gm_id).bind(name).bind(description).fetch_one(pool).await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Campaign>, sqlx::Error> {
        sqlx::query_as::<_, Campaign>(
            r#"
            SELECT id, gm_id, name, description, created_at, updated_at
            FROM campaigns
            WHERE id = $1
            "#,
        ).bind(id).fetch_optional(pool).await
    }

    pub async fn list_by_gm(pool: &PgPool, gm_id: Uuid) -> Result<Vec<Campaign>, sqlx::Error> {
        sqlx::query_as::<_, Campaign>(
            r#"
            SELECT id, gm_id, name, description, created_at, updated_at
            FROM campaigns
            WHERE gm_id = $1
            ORDER BY created_at DESC
            "#,
        ).bind(gm_id).fetch_all(pool).await
    }

    pub async fn update(pool: &PgPool, id: Uuid, name: Option<&str>, description: Option<&str>,) -> Result<Option<Campaign>, sqlx::Error> {
        sqlx::query_as::<_, Campaign>(
            r#"
            UPDATE campaigns
            SET name = COALESCE($2::text, name),
                description = COALESCE($3::text, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, gm_id, name, description, created_at, updated_at
            "#,
        ).bind(id).bind(name).bind(description).fetch_optional(pool).await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM campaigns WHERE id = $1").bind(id).execute(pool).await?;
        Ok(())
    }
}

impl From<Campaign> for shared::CampaignResponse {
    fn from(campaign: Campaign) -> Self {
        Self {
            id: campaign.id,
            gm_id: campaign.gm_id,
            name: campaign.name,
            description: campaign.description,
            created_at: campaign.created_at,
            updated_at: campaign.updated_at,
        }
    }
}