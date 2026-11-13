/*
 ** EPITECH PROJECT, 2026
 ** campaigns.rs
 ** File description:
 ** campaign routes and handlers
 */

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::{CampaignResponse, CreateCampaignRequest, UpdateCampaignRequest};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::extractor::AuthUser, error::AppError, models::campaign::Campaign};

const MAX_NAME_LEN: usize = 100;
const MAX_DESCRITPION_LEN: usize = 2000;

fn validate_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();

    if name.is_empty() {
        return Err(AppError::Validation("Name cannot be empty".to_string()));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Validation(format!(
            "Name cannot exceed {MAX_NAME_LEN} characters"
        )));
    }
    Ok(name.to_string())
}

fn validate_description(description: &str) -> Result<String, AppError> {
    let description = description.trim();

    if description.chars().count() > MAX_DESCRITPION_LEN {
        return Err(AppError::Validation(format!(
            "Description cannot exceed {MAX_DESCRITPION_LEN} characters"
        )));
    }
    Ok(description.to_string())
}

async fn find_owned(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Campaign, AppError> {
    let campaign = Campaign::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

    if campaign.gm_id != user_id {
        return Err(AppError::Forbidden(
            "Only the game master can access this campaign".to_string(),
        ));
    }
    Ok(campaign)
}

pub async fn creat_campaign(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Json(payload): Json<CreateCampaignRequest>,
) -> Result<(StatusCode, Json<CampaignResponse>), AppError> {
    let name = validate_name(&payload.name)?;
    let description = validate_description(&payload.description)?;
    let campaign = Campaign::create(&pool, auth.user.id, &name, &description).await?;

    Ok((StatusCode::CREATED, Json(campaign.into())))
}

pub async fn list_campaigns(
    State(pool): State<PgPool>,
    auth: AuthUser,
) -> Result<Json<Vec<CampaignResponse>>, AppError> {
    let campaigns = Campaign::list_by_gm(&pool, auth.user.id).await?;

    Ok(Json(
        campaigns.into_iter().map(CampaignResponse::from).collect(),
    ))
}

pub async fn get_campaign(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CampaignResponse>, AppError> {
    let campaign = find_owned(&pool, id, auth.user.id).await?;

    Ok(Json(campaign.into()))
}

pub async fn update_campaign(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCampaignRequest>,
) -> Result<Json<CampaignResponse>, AppError> {
    let campaign = find_owned(&pool, id, auth.user.id).await?;

    if payload.name.is_none() && payload.description.is_none() {
        return Err(AppError::Validation("Nothing to update".to_string()));
    }
    let name = payload.name.as_deref().map(validate_name).transpose()?;
    let description = payload
        .description
        .as_deref()
        .map(validate_description)
        .transpose()?;
    let updated = Campaign::update(&pool, campaign.id, name.as_deref(), description.as_deref())
        .await?
        .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

    Ok(Json(updated.into()))
}

pub async fn delete_campaign(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let campaign = find_owned(&pool, id, auth.user.id).await?;

    Campaign::delete(&pool, campaign.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
