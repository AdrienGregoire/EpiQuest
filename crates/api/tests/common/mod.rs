/*
 ** EPITECH PROJECT, 2026
 ** mod.rs
 ** File description:
 ** shared helpers for integration tests
 */

#![allow(dead_code)]

use api::auth::session::SESSION_COOKIE;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;

pub const PASSWORD: &str = "valid_password_123";

pub async fn setup_test_pool() -> PgPool {
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

pub async fn call(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> Response {
    let mut builder = Request::builder().method(method).uri(uri);

    if let Some(token) = token {
        builder = builder.header("Cookie", format!("{SESSION_COOKIE}={token}"));
    }
    let body = match body {
        Some(value) => {
            builder = builder.header("Content-Type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    app.clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap()
}

pub async fn json_body(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();

    serde_json::from_slice(&bytes).unwrap()
}

pub async fn register(app: &Router) -> String {
    let id = uuid::Uuid::new_v4();
    let email = format!("user_{id}@test.com");
    let payload = json!({"username": format!("user_{id}"), "email": email, "password": PASSWORD});
    let response = call(app, "POST", "/auth/register", Some(payload), None).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    email
}

pub async fn login(app: &Router, email: &str, password: &str) -> Response {
    let payload = json!({ "email": email, "password": password });

    call(app, "POST", "/auth/login", Some(payload), None).await
}

pub fn set_cookie_header(response: &Response) -> String {
    response
        .headers()
        .get("set-cookie")
        .expect("missing Set-Cookie header")
        .to_str()
        .unwrap()
        .to_string()
}

pub fn extract_token(response: &Response) -> String {
    set_cookie_header(response)
        .split(';')
        .next()
        .unwrap()
        .strip_prefix(&format!("{SESSION_COOKIE}="))
        .expect("unexpected cookie name")
        .to_string()
}

pub async fn register_and_login(app: &Router) -> String {
    let email = register(app).await;
    let response = login(app, &email, PASSWORD).await;

    assert_eq!(response.status(), StatusCode::OK);
    extract_token(&response)
}
