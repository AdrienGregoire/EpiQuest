/*
 ** EPITECH PROJECT, 2026
 ** campaigns.rs
 ** File description:
 ** campaign routes and handlers
 */

use axum::{Extension, Json, extract::State, http::StatusCode};
use shared::{CampaignResponse, CreateCampaignRequest, MemberResponse, UpdateCampaignRequest};
use sqlx::PgPool;

use crate::{
    auth::{extractor::AuthUser, permissions::CampaignContext},
    error::AppError,
    models::{campaign::Campaign, member::Member},
};

const MAX_NAME_LEN: usize = 100;
const MAX_DESCRIPTION_LEN: usize = 2000;

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

    if description.chars().count() > MAX_DESCRIPTION_LEN {
        return Err(AppError::Validation(format!(
            "Description cannot exceed {MAX_DESCRIPTION_LEN} characters"
        )));
    }
    Ok(description.to_string())
}

pub async fn create_campaign(
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

pub async fn list_joined_campaigns(
    State(pool): State<PgPool>,
    auth: AuthUser,
) -> Result<Json<Vec<CampaignResponse>>, AppError> {
    let campaigns = Campaign::list_joined_by(&pool, auth.user.id).await?;

    Ok(Json(
        campaigns.into_iter().map(CampaignResponse::from).collect(),
    ))
}

pub async fn get_campaign(Extension(ctx): Extension<CampaignContext>) -> Json<CampaignResponse> {
    Json(ctx.campaign.into())
}

pub async fn list_members(
    State(pool): State<PgPool>,
    Extension(ctx): Extension<CampaignContext>,
) -> Result<Json<Vec<MemberResponse>>, AppError> {
    let members = Member::list_for_campaign(&pool, ctx.campaign.id).await?;

    Ok(Json(
        members.into_iter().map(MemberResponse::from).collect(),
    ))
}

pub async fn update_campaign(
    State(pool): State<PgPool>,
    Extension(ctx): Extension<CampaignContext>,
    Json(payload): Json<UpdateCampaignRequest>,
) -> Result<Json<CampaignResponse>, AppError> {
    if payload.name.is_none() && payload.description.is_none() {
        return Err(AppError::Validation("Nothing to update".to_string()));
    }
    let name = payload.name.as_deref().map(validate_name).transpose()?;
    let description = payload
        .description
        .as_deref()
        .map(validate_description)
        .transpose()?;
    let updated = Campaign::update(
        &pool,
        ctx.campaign.id,
        name.as_deref(),
        description.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

    Ok(Json(updated.into()))
}

pub async fn delete_campaign(
    State(pool): State<PgPool>,
    Extension(ctx): Extension<CampaignContext>,
) -> Result<StatusCode, AppError> {
    Campaign::delete(&pool, ctx.campaign.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
