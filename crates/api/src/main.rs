/*
** EPITECH PROJECT, 2026
** main.rs
** File description:
** main file
*/

use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod auth;
mod error;
mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Unable to load the .env file");
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let app: Router = axum::Router::new()
        .route(
            "/auth/register",
            axum::routing::post(routes::auth::register),
        )
        .with_state(pool);
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
    tracing::info!("Server running on http://{host}:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}
