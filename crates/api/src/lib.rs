/*
 ** EPITECH PROJECT, 2026
 ** lib.rs
 ** File description:
 ** lib file
 */

pub mod auth;
pub mod error;
pub mod models;
pub mod routes;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, patch, post},
};
use sqlx::PgPool;

use crate::auth::permissions::{require_auth, require_gm, require_member};

pub fn create_app(pool: PgPool) -> Router {
    let public = Router::new()
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login));
    let authenticated = Router::new()
        .route("/auth/me", get(routes::auth::me))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/logout-all", post(routes::auth::logout_all))
        .route(
            "/campaigns",
            post(routes::campaigns::create_campaign).get(routes::campaigns::list_campaigns),
        )
        .route("/campaigns/join", post(routes::invites::join_campaign))
        .route(
            "/campaigns/joined",
            get(routes::campaigns::list_joined_campaigns),
        )
        .route_layer(from_fn_with_state(pool.clone(), require_auth));
    let members = Router::new()
        .route("/campaigns/{id}", get(routes::campaigns::get_campaign))
        .route(
            "/campaigns/{id}/members",
            get(routes::campaigns::list_members),
        )
        .route(
            "/campaigns/{id}/rolls",
            post(routes::rolls::roll_dice).get(routes::rolls::get_rolls),
        )
        .route_layer(from_fn_with_state(pool.clone(), require_member));
    let game_master = Router::new()
        .route(
            "/campaigns/{id}",
            patch(routes::campaigns::update_campaign).delete(routes::campaigns::delete_campaign),
        )
        .route(
            "/campaigns/{id}/invites",
            post(routes::invites::create_invite).get(routes::invites::list_invites),
        )
        .route(
            "/campaigns/{id}/invites/{invite_id}",
            delete(routes::invites::revoke_invite),
        )
        .route_layer(from_fn_with_state(pool.clone(), require_gm));

    Router::new()
        .merge(public)
        .merge(authenticated)
        .merge(members)
        .merge(game_master)
        .with_state(pool)
}
