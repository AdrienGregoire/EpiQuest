/*
** EPITECH PROJECT, 2026
** main.rs
** File description:
** main file
*/

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
    let host = env::var("HOST").expect("HOST must be defined");
    let port = env::var("PORT").expect("PORT must be defined");
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
    tracing::info!("Server running on http://{host}:{port}");
    Ok(())
}
