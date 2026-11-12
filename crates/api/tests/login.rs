/*
** EPITECH PROJECT, 2026
** login.rs
** File description:
** integration tests for login and sessions
*/

use api::{
    auth::session::{SESSION_COOKIE, hash_token},
    create_app,
};
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

const PASSWORD: &str = "valid_password_123";

async fn setup_test_pool() -> PgPool {
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

async fn call(
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

async fn register(app: &Router) -> String {
    let id = uuid::Uuid::new_v4();
    let email = format!("user_{id}@test.com");
    let payload = json!({
        "username": format!("user_{id}"),
        "email": email,
        "password": PASSWORD,
    });
    let response = call(app, "POST", "/auth/register", Some(payload), None).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    email
}

async fn login(app: &Router, email: &str, password: &str) -> Response {
    let payload = json!({ "email": email, "password": password });

    call(app, "POST", "/auth/login", Some(payload), None).await
}

fn set_cookie_header(response: &Response) -> String {
    response
        .headers()
        .get("set-cookie")
        .expect("missing Set-Cookie header")
        .to_str()
        .unwrap()
        .to_string()
}

fn extract_token(response: &Response) -> String {
    set_cookie_header(response)
        .split(';')
        .next()
        .unwrap()
        .strip_prefix(&format!("{SESSION_COOKIE}="))
        .expect("unexpected cookie name")
        .to_string()
}

#[tokio::test]
async fn test_login_success_sets_hardened_cookie() {
    let app = create_app(setup_test_pool().await);
    let email = register(&app).await;

    let response = login(&app, &email, PASSWORD).await;
    assert_eq!(response.status(), StatusCode::OK);

    let cookie = set_cookie_header(&response);
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    assert!(cookie.contains("Path=/"));
    assert!(cookie.contains("Max-Age="));

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: Value = serde_json::from_slice(&body).unwrap();
    assert!(user.get("password_hash").is_none());
    assert_eq!(user["email"], email);
}

#[tokio::test]
async fn test_login_rejects_bad_credentials() {
    let app = create_app(setup_test_pool().await);
    let email = register(&app).await;

    let wrong_password = login(&app, &email, "wrong_password_123").await;
    assert_eq!(wrong_password.status(), StatusCode::UNAUTHORIZED);
    assert!(wrong_password.headers().get("set-cookie").is_none());

    let unknown_email = login(&app, "nobody@test.com", PASSWORD).await;
    assert_eq!(unknown_email.status(), StatusCode::UNAUTHORIZED);
    assert!(unknown_email.headers().get("set-cookie").is_none());
}

#[tokio::test]
async fn test_me_requires_valid_session() {
    let app = create_app(setup_test_pool().await);
    let email = register(&app).await;

    let anonymous = call(&app, "GET", "/auth/me", None, None).await;
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);

    let forged = call(&app, "GET", "/auth/me", None, Some("not_a_real_token")).await;
    assert_eq!(forged.status(), StatusCode::UNAUTHORIZED);

    let token = extract_token(&login(&app, &email, PASSWORD).await);
    let authenticated = call(&app, "GET", "/auth/me", None, Some(&token)).await;
    assert_eq!(authenticated.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_logout_revokes_session() {
    let app = create_app(setup_test_pool().await);
    let email = register(&app).await;
    let token = extract_token(&login(&app, &email, PASSWORD).await);

    let logout = call(&app, "POST", "/auth/logout", None, Some(&token)).await;
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);

    let after = call(&app, "GET", "/auth/me", None, Some(&token)).await;
    assert_eq!(after.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_all_revokes_every_session() {
    let app = create_app(setup_test_pool().await);
    let email = register(&app).await;
    let first = extract_token(&login(&app, &email, PASSWORD).await);
    let second = extract_token(&login(&app, &email, PASSWORD).await);
    assert_ne!(first, second);

    let response = call(&app, "POST", "/auth/logout-all", None, Some(&first)).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let other = call(&app, "GET", "/auth/me", None, Some(&second)).await;
    assert_eq!(other.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_expired_session_is_rejected() {
    let pool = setup_test_pool().await;
    let app = create_app(pool.clone());
    let email = register(&app).await;
    let token = extract_token(&login(&app, &email, PASSWORD).await);

    sqlx::query(
        "UPDATE sessions SET expires_at = NOW() - INTERVAL '1 second' WHERE token_hash = $1",
    )
    .bind(hash_token(&token))
    .execute(&pool)
    .await
    .unwrap();

    let response = call(&app, "GET", "/auth/me", None, Some(&token)).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
