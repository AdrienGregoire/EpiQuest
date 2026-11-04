/*
** EPITECH PROJECT, 2026
** register.rs
** File description:
** integration tests for user registration
*/

use api::create_app;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

async fn setup_test_pool() -> sqlx::PgPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://app:app@localhost:5432/app".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

#[tokio::test]
async fn test_register_flow() {
    let pool = setup_test_pool().await;
    let app = create_app(pool);

    let random_id = uuid::Uuid::new_v4();
    let payload = json!({
        "username": format!("user_{random_id}"),
        "email": format!("user_{random_id}@test.com"),
        "password": "valid_password_123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user_resp: Value = serde_json::from_slice(&body).unwrap();

    assert!(user_resp.get("password_hash").is_none());
    assert!(user_resp.get("id").is_some());

    let duplicate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(duplicate_response.status(), StatusCode::CONFLICT);
    let short_password_payload = json!({
        "username": "short_pwd_user",
        "email": "short@test.com",
        "password": "123"
    });

    let invalid_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(short_password_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(invalid_response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
