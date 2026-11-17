/*
 ** EPITECH PROJECT, 2026
 ** permissions.rs
 ** File description:
 ** role-based access middlewares
 */

use std::collections::HashMap;

use axum::{
    extract::{FromRequestParts, Path, Request, State},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::extractor::AuthUser,
    error::AppError,
    models::{campaign::Campaign, member::Member},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Gm,
    Player,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Member,
    Gm,
}

impl Requirement {
    pub fn is_met_by(self, role: Role) -> bool {
        match self {
            Requirement::Member => true,
            Requirement::Gm => role == Role::Gm,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CampaignContext {
    pub campaign: Campaign,
    pub role: Role,
}

pub async fn require_auth(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();
    let auth = AuthUser::from_request_parts(&mut parts, &pool).await?;

    parts.extensions.insert(auth);
    Ok(next.run(Request::from_parts(parts, body)).await)
}

pub async fn require_member(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    authorize_campaign(&pool, Requirement::Member, request, next).await
}

pub async fn require_gm(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    authorize_campaign(&pool, Requirement::Gm, request, next).await
}

async fn authorize_campaign(
    pool: &PgPool,
    required: Requirement,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();
    let auth = AuthUser::from_request_parts(&mut parts, pool).await?;
    let Path(params) = Path::<HashMap<String, String>>::from_request_parts(&mut parts, pool)
        .await
        .map_err(|_| AppError::NotFound("Campaign not found".to_string()))?;
    let id = params
        .get("id")
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;
    let campaign = Campaign::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;
    let role = if campaign.gm_id == auth.user.id {
        Role::Gm
    } else if Member::is_member(pool, campaign.id, auth.user.id).await? {
        Role::Player
    } else {
        return Err(AppError::Forbidden(
            "You are not part of this campaign".to_string(),
        ));
    };

    if !required.is_met_by(role) {
        return Err(AppError::Forbidden(
            "Only the game master can do this".to_string(),
        ));
    }
    parts.extensions.insert(auth);
    parts.extensions.insert(CampaignContext { campaign, role });
    Ok(next.run(Request::from_parts(parts, body)).await)
}
