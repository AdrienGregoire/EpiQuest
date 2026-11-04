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

use axum::Router;
use sqlx::PgPool;

pub fn create_app(pool: PgPool) -> Router {
    Router::new()
        .route(
            "/auth/register",
            axum::routing::post(routes::auth::register),
        )
        .with_state(pool)
}
