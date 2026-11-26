/*
 ** EPITECH PROJECT, 2026
 ** rolls.rs
 ** File description:
 ** Dice roll routes - server-side RNG
 */

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::extractor::AuthUser, error::AppError, models::roll::Roll};
use shared::{RollRequest, RollResponse};

pub async fn roll_dice(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Path(campaign_id): Path<Uuid>,
    Json(payload): Json<RollRequest>,
) -> Result<(StatusCode, Json<RollResponse>), AppError> {
    let notation = payload.notation.trim();
    if notation.is_empty() {
        return Err(AppError::Validation("Notation cannot be empty".to_string()));
    }
    if notation.len() > 100 {
        return Err(AppError::Validation(
            "Notation too long (max 100 characters)".to_string(),
        ));
    }
    let result = dice::roll(notation).map_err(|e| AppError::Validation(e.to_string()))?;
    let total = result.total;
    let roll = Roll::create(&pool, campaign_id, auth.user.id, notation, &result, total).await?;
    let response = RollResponse {
        id: roll.id,
        campaign_id: roll.campaign_id,
        user_id: roll.user_id,
        notation: roll.notation,
        result: serde_json::to_value(&result).map_err(|e| AppError::Internal(e.to_string()))?,
        total: roll.total as i64,
        created_at: roll.created_at,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_rolls(
    State(pool): State<PgPool>,
    _auth: AuthUser,
    Path(campaign_id): Path<Uuid>,
) -> Result<Json<Vec<RollResponse>>, AppError> {
    let rolls = Roll::list_by_campaign(&pool, campaign_id, 50).await?;

    let responses: Vec<RollResponse> = rolls
        .into_iter()
        .map(|r| RollResponse {
            id: r.id,
            campaign_id: r.campaign_id,
            user_id: r.user_id,
            notation: r.notation,
            result: serde_json::to_value(&r.result.0).unwrap_or_default(),
            total: r.total as i64,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(responses))
}
