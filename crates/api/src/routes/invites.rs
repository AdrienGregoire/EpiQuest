/*
 ** EPITECH PROJECT, 2026
 ** invites.rs
 ** File description:
 ** invite and join routes and handlers
 */

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use shared::{CampaignResponse, InviteResponse, JoinCampaignRequest};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::{extractor::AuthUser, permissions::CampaignContext},
    error::AppError,
    models::{
        campaign::Campaign,
        invite::{INVITE_TTL_HOURS, Invite, generate_code},
        member::Member,
    },
};

pub async fn create_invite(
    State(pool): State<PgPool>,
    Extension(ctx): Extension<CampaignContext>,
) -> Result<(StatusCode, Json<InviteResponse>), AppError> {
    let invite = Invite::create(&pool, ctx.campaign.id, &generate_code(), INVITE_TTL_HOURS).await?;

    Ok((StatusCode::CREATED, Json(invite.into())))
}

pub async fn list_invites(
    State(pool): State<PgPool>,
    Extension(ctx): Extension<CampaignContext>,
) -> Result<Json<Vec<InviteResponse>>, AppError> {
    let invites = Invite::list_active(&pool, ctx.campaign.id).await?;

    Ok(Json(
        invites.into_iter().map(InviteResponse::from).collect(),
    ))
}

pub async fn revoke_invite(
    State(pool): State<PgPool>,
    Extension(ctx): Extension<CampaignContext>,
    Path((_, invite_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    if !Invite::revoke(&pool, ctx.campaign.id, invite_id).await? {
        return Err(AppError::NotFound("Invite not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn join_campaign(
    State(pool): State<PgPool>,
    auth: AuthUser,
    Json(payload): Json<JoinCampaignRequest>,
) -> Result<(StatusCode, Json<CampaignResponse>), AppError> {
    let code = payload.code.trim().to_uppercase();
    let invite = Invite::find_active_by_code(&pool, &code)
        .await?
        .ok_or_else(|| AppError::NotFound("Invalid or expired invite code".to_string()))?;
    let campaign = Campaign::find_by_id(&pool, invite.campaign_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invalid or expired invite code".to_string()))?;

    if campaign.gm_id == auth.user.id {
        return Err(AppError::Conflict(
            "You are the game master of this campaign".to_string(),
        ));
    }
    if !Member::add_player(&pool, campaign.id, auth.user.id).await? {
        return Err(AppError::Conflict(
            "You already joined this campaign".to_string(),
        ));
    }
    Ok((StatusCode::CREATED, Json(campaign.into())))
}
