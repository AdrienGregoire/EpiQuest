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
    routing::{get, post},
};
use sqlx::PgPool;

pub fn create_app(pool: PgPool) -> Router {
    Router::new()
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/logout-all", post(routes::auth::logout_all))
        .route("/auth/me", get(routes::auth::me))
        .with_state(pool)
}
