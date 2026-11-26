/*
 ** EPITECH PROJECT, 2026
 ** rolls.rs
 ** File description:
 ** Integration tests for server-side dice rolling
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
        .unwrap_or("postgres://app:app@localhost:5432/app".to_string());
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

async fn register_and_login(
    app: axum::Router,
    username: &str,
    email: &str,
) -> (axum::Router, String) {
    let register_payload = json!({
        "username": username,
        "email": email,
        "password": "test_password_123"
    });
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let login_payload = json!({
        "email": email,
        "password": "test_password_123"
    });
    let login_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let cookie = login_resp
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    (app, cookie)
}

#[tokio::test]
async fn test_roll_requires_valid_notation() {
    let pool = setup_test_pool().await;
    let app = create_app(pool);
    let random_id = uuid::Uuid::new_v4();
    let email = format!("roll_test_{random_id}@test.com");
    let (app, cookie) = register_and_login(app, &format!("roll_{random_id}"), &email).await;
    let campaign_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/campaigns")
                .header("Content-Type", "application/json")
                .header("Cookie", &cookie)
                .body(Body::from(
                    json!({"name": "Test Campaign", "description": "test"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let campaign_body = campaign_resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let campaign: Value = serde_json::from_slice(&campaign_body).unwrap();
    let campaign_id = campaign["id"].as_str().unwrap();
    let empty_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/campaigns/{campaign_id}/rolls"))
                .header("Content-Type", "application/json")
                .header("Cookie", &cookie)
                .body(Body::from(json!({"notation": ""}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(empty_resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let invalid_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/campaigns/{campaign_id}/rolls"))
                .header("Content-Type", "application/json")
                .header("Cookie", &cookie)
                .body(Body::from(json!({"notation": "xyz"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let valid_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/campaigns/{campaign_id}/rolls"))
                .header("Content-Type", "application/json")
                .header("Cookie", &cookie)
                .body(Body::from(json!({"notation": "2d6+3"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(valid_resp.status(), StatusCode::CREATED);

    let body = valid_resp.into_body().collect().await.unwrap().to_bytes();
    let roll: Value = serde_json::from_slice(&body).unwrap();

    assert!(roll.get("id").is_some());
    assert!(roll.get("result").is_some());
    assert!(roll.get("total").is_some());
    let total = roll["total"].as_i64().unwrap();
    assert!((5..=15).contains(&total)); // 2d6+3 range

    let history_resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/campaigns/{campaign_id}/rolls"))
                .header("Cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(history_resp.status(), StatusCode::OK);

    let history_body = history_resp.into_body().collect().await.unwrap().to_bytes();
    let rolls: Vec<Value> = serde_json::from_slice(&history_body).unwrap();
    assert!(!rolls.is_empty());
}
