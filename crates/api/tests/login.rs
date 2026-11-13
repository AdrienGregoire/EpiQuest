/*
 ** EPITECH PROJECT, 2026
 ** login.rs
 ** File description:
 ** integration tests for login and sessions
 */

mod common;

use api::{auth::session::hash_token, create_app};
use axum::http::StatusCode;
use common::{
    PASSWORD, call, extract_token, json_body, login, register, set_cookie_header, setup_test_pool,
};

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

    let user = json_body(response).await;
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
